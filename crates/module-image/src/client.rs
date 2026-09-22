use anyhow::{Result, anyhow, ensure};
use reqwest::{Client, StatusCode, header};
use serde::Deserialize;
use std::{
    collections::HashMap,
    sync::Mutex,
    time::{Duration, Instant},
};
use tokio::sync::Semaphore;
use url::Url;

const SEARCH_URL: &str = "https://duckduckgo.com/";
const IMAGES_URL: &str = "https://duckduckgo.com/i.js";
const USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/140.0.0.0 Safari/537.36";
const MAX_SEARCH_PAGE_BYTES: usize = 1024 * 1024;
const MAX_RESPONSE_BYTES: usize = 2 * 1024 * 1024;
const CACHE_TTL: Duration = Duration::from_secs(10 * 60);
const MAX_CACHE_ENTRIES: usize = 128;

pub struct DuckDuckGoImageService {
    client: Client,
    cache: Mutex<HashMap<String, CacheEntry>>,
    requests: Semaphore,
}

#[derive(Clone)]
struct CacheEntry {
    stored_at: Instant,
    result: Option<ImageResult>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ImageResult {
    pub title: String,
    pub image_url: String,
    pub page_url: Option<String>,
    pub source: Option<String>,
}

#[derive(Deserialize)]
struct SearchResponse {
    #[serde(default)]
    results: Vec<ApiImageResult>,
}

#[derive(Deserialize)]
struct ApiImageResult {
    #[serde(default)]
    title: Option<String>,
    #[serde(default)]
    url: Option<String>,
    #[serde(default)]
    source: Option<String>,
    #[serde(default)]
    thumbnail: Option<String>,
    #[serde(default)]
    image: Option<String>,
}

impl DuckDuckGoImageService {
    pub fn new() -> Result<Self> {
        let client = Client::builder()
            .connect_timeout(Duration::from_secs(5))
            .timeout(Duration::from_secs(15))
            .redirect(reqwest::redirect::Policy::none())
            .no_proxy()
            // DuckDuckGo rejects its image endpoint unless the request matches its browser client.
            .user_agent(USER_AGENT)
            .build()?;
        Ok(Self {
            client,
            cache: Mutex::new(HashMap::new()),
            requests: Semaphore::new(4),
        })
    }

    pub async fn search(&self, query: &str) -> Result<Option<ImageResult>> {
        let query = validate_query(query)?;
        let cache_key = query.to_lowercase();
        if let Some(result) = self.cached(&cache_key)? {
            return Ok(result);
        }

        let _permit = self
            .requests
            .acquire()
            .await
            .map_err(|_| anyhow!("Image Search is shutting down"))?;
        if let Some(result) = self.cached(&cache_key)? {
            return Ok(result);
        }

        let search_page = self
            .client
            .get(SEARCH_URL)
            .query(&[("q", query)])
            .send()
            .await
            .map_err(|_| anyhow!("DuckDuckGo Image Search request failed"))?;
        let search_page = read_body(search_page, MAX_SEARCH_PAGE_BYTES).await?;
        let search_page = std::str::from_utf8(&search_page)
            .map_err(|_| anyhow!("DuckDuckGo returned an invalid search page"))?;
        let vqd = extract_vqd(search_page).ok_or_else(|| {
            anyhow!(
                "DuckDuckGo Image Search is temporarily unavailable; its endpoint may have changed"
            )
        })?;

        let response = self
            .client
            .get(IMAGES_URL)
            .header(header::ACCEPT, "*/*")
            .header(header::ACCEPT_LANGUAGE, "en-US,en;q=0.5")
            .header(header::REFERER, SEARCH_URL)
            .header("Sec-GPC", "1")
            .header("Sec-Fetch-Dest", "empty")
            .header("Sec-Fetch-Mode", "cors")
            .header("Sec-Fetch-Site", "same-origin")
            .header("Priority", "u=4")
            .query(&[
                ("o", "json"),
                ("q", query),
                ("l", "us-en"),
                ("vqd", vqd),
                ("p", "1"),
                ("ct", "AT"),
            ])
            .send()
            .await
            .map_err(|_| anyhow!("DuckDuckGo Image Search request failed"))?;
        let response: SearchResponse = read_json(response).await?;
        let result = select_result(response.results);
        self.store(cache_key, result.clone())?;
        Ok(result)
    }

    fn cached(&self, key: &str) -> Result<Option<Option<ImageResult>>> {
        let cache = self
            .cache
            .lock()
            .map_err(|_| anyhow!("Image Search cache is unavailable"))?;
        Ok(cache
            .get(key)
            .filter(|entry| entry.stored_at.elapsed() < CACHE_TTL)
            .map(|entry| entry.result.clone()))
    }

    fn store(&self, key: String, result: Option<ImageResult>) -> Result<()> {
        let mut cache = self
            .cache
            .lock()
            .map_err(|_| anyhow!("Image Search cache is unavailable"))?;
        cache.retain(|_, entry| entry.stored_at.elapsed() < CACHE_TTL);
        if cache.len() >= MAX_CACHE_ENTRIES {
            cache.clear();
        }
        cache.insert(
            key,
            CacheEntry {
                stored_at: Instant::now(),
                result,
            },
        );
        Ok(())
    }
}

fn validate_query(query: &str) -> Result<&str> {
    let query = query.trim();
    ensure!(!query.is_empty(), "Enter something to search for");
    ensure!(
        query.chars().count() <= 400 && query.split_whitespace().count() <= 50,
        "Search queries may contain at most 400 characters and 50 words"
    );
    Ok(query)
}

fn extract_vqd(page: &str) -> Option<&str> {
    ["vqd='", "vqd=\"", "\"vqd\":\"", "\"vqd\": \""]
        .into_iter()
        .find_map(|marker| {
            let value = page.split_once(marker)?.1.split(['\'', '\"']).next()?;
            (!value.is_empty()
                && value.len() <= 128
                && value
                    .bytes()
                    .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_')))
            .then_some(value)
        })
}

fn select_result(results: Vec<ApiImageResult>) -> Option<ImageResult> {
    results.into_iter().find_map(|result| {
        let image_url = result.thumbnail.or(result.image)?;
        let image_url = safe_url(image_url, true)?;
        let page_url = result.url.and_then(|url| safe_url(url, false));
        Some(ImageResult {
            title: result
                .title
                .filter(|title| !title.trim().is_empty())
                .unwrap_or_else(|| "Image result".into()),
            image_url,
            page_url,
            source: result.source.filter(|source| !source.trim().is_empty()),
        })
    })
}

fn safe_url(value: String, https_only: bool) -> Option<String> {
    if value.len() > 2_048 {
        return None;
    }
    let url = Url::parse(&value).ok()?;
    if (https_only && url.scheme() != "https")
        || (!https_only && !matches!(url.scheme(), "http" | "https"))
        || url.host_str().is_none()
        || !url.username().is_empty()
        || url.password().is_some()
    {
        return None;
    }
    Some(value)
}

async fn read_body(mut response: reqwest::Response, limit: usize) -> Result<Vec<u8>> {
    let status = response.status();
    if status == StatusCode::TOO_MANY_REQUESTS {
        return Err(anyhow!(
            "DuckDuckGo Image Search is rate limited; try again later"
        ));
    }
    if !status.is_success() {
        return Err(anyhow!(
            "DuckDuckGo Image Search failed with status {}",
            status.as_u16()
        ));
    }

    let mut body = Vec::new();
    while let Some(chunk) = response
        .chunk()
        .await
        .map_err(|_| anyhow!("DuckDuckGo response could not be read"))?
    {
        ensure!(
            body.len().saturating_add(chunk.len()) <= limit,
            "DuckDuckGo response was too large"
        );
        body.extend_from_slice(&chunk);
    }
    Ok(body)
}

async fn read_json<T: for<'de> Deserialize<'de>>(response: reqwest::Response) -> Result<T> {
    let body = read_body(response, MAX_RESPONSE_BYTES).await?;
    serde_json::from_slice(&body).map_err(|_| {
        anyhow!("DuckDuckGo returned an invalid response; its image endpoint may have changed")
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_current_vqd_shapes() {
        assert_eq!(extract_vqd("x vqd='4-123-456' y"), Some("4-123-456"));
        assert_eq!(extract_vqd(r#"{"vqd":"4-token_1"}"#), Some("4-token_1"));
        assert_eq!(extract_vqd("vqd='<script>'"), None);
    }

    #[test]
    fn selects_a_safe_renderable_result() {
        let response: SearchResponse = serde_json::from_value(serde_json::json!({
            "results": [
                {"title": "Unsafe", "thumbnail": "http://example.com/a.jpg"},
                {
                    "title": "Cat",
                    "url": "https://example.com/cat",
                    "source": "example.com",
                    "thumbnail": "https://external-content.duckduckgo.com/cat.jpg",
                    "image": "https://example.com/cat.jpg"
                }
            ]
        }))
        .unwrap();
        assert_eq!(
            select_result(response.results),
            Some(ImageResult {
                title: "Cat".into(),
                image_url: "https://external-content.duckduckgo.com/cat.jpg".into(),
                page_url: Some("https://example.com/cat".into()),
                source: Some("example.com".into()),
            })
        );
    }

    #[test]
    fn enforces_query_limits() {
        assert!(validate_query("cats").is_ok());
        assert!(validate_query("   ").is_err());
        assert!(validate_query(&"word ".repeat(51)).is_err());
        assert!(validate_query(&"x".repeat(401)).is_err());
    }

    #[tokio::test]
    #[ignore = "uses the live unofficial DuckDuckGo image endpoint"]
    async fn live_duckduckgo_smoke_test() {
        let result = DuckDuckGoImageService::new()
            .unwrap()
            .search("orange cat")
            .await
            .unwrap();
        assert!(result.is_some());
    }
}
