use anyhow::{Result, anyhow, ensure};
use reqwest::{Client, StatusCode, header};
use serde::Deserialize;
use std::{
    collections::HashMap,
    error::Error,
    fmt,
    sync::Mutex,
    time::{Duration, Instant},
};
use tokio::sync::Semaphore;
use url::Url;

const SEARCH_URL: &str = "https://duckduckgo.com/";
const IMAGES_URL: &str = "https://duckduckgo.com/i.js";
const BING_IMAGES_URL: &str = "https://www.bing.com/images/async";
const DUCKDUCKGO_PROVIDER: &str = "DuckDuckGo";
const BING_PROVIDER: &str = "Bing";
const USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/140.0.0.0 Safari/537.36";
const MAX_SEARCH_PAGE_BYTES: usize = 1024 * 1024;
const MAX_RESPONSE_BYTES: usize = 2 * 1024 * 1024;
const CACHE_TTL: Duration = Duration::from_secs(10 * 60);
const MAX_CACHE_ENTRIES: usize = 128;
const SESSION_TTL: Duration = Duration::from_secs(5 * 60);
const MAX_SESSIONS: usize = 256;
const MAX_RESULTS: usize = 10;

pub struct DuckDuckGoImageService {
    client: Client,
    cache: Mutex<HashMap<String, CacheEntry>>,
    sessions: Mutex<HashMap<u64, ImageSession>>,
    requests: Semaphore,
}

#[derive(Clone)]
struct CacheEntry {
    stored_at: Instant,
    results: Vec<ImageResult>,
}

struct ImageSession {
    owner: u64,
    query: String,
    results: Vec<ImageResult>,
    current: usize,
    touched_at: Instant,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ImageResult {
    pub provider: &'static str,
    pub title: String,
    pub image_url: String,
    pub page_url: Option<String>,
    pub source: Option<String>,
}

#[derive(Debug)]
struct DuckDuckGoBlocked;

impl fmt::Display for DuckDuckGoBlocked {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("DuckDuckGo Image Search rejected the request")
    }
}

impl Error for DuckDuckGoBlocked {}

#[derive(Clone, Copy)]
pub(crate) enum PageDirection {
    Previous,
    Next,
}

pub(crate) enum SessionAdvance {
    Updated(ImagePage),
    Expired,
    NotOwner,
}

pub(crate) struct ImagePage {
    pub query: String,
    pub result: ImageResult,
    pub index: usize,
    pub total: usize,
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
            sessions: Mutex::new(HashMap::new()),
            requests: Semaphore::new(4),
        })
    }

    pub async fn search(&self, query: &str) -> Result<Vec<ImageResult>> {
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

        let results = match self.search_duckduckgo(query).await {
            Ok(results) => results,
            Err(error) if error.is::<DuckDuckGoBlocked>() => self.search_bing(query).await?,
            Err(error) => return Err(error),
        };
        self.store(cache_key, results.clone())?;
        Ok(results)
    }

    async fn search_duckduckgo(&self, query: &str) -> Result<Vec<ImageResult>> {
        let search_page = self
            .client
            .get(SEARCH_URL)
            .query(&[("q", query)])
            .send()
            .await
            .map_err(|_| anyhow!("DuckDuckGo Image Search request failed"))?;
        reject_blocked_duckduckgo(&search_page)?;
        let search_page =
            read_body(search_page, MAX_SEARCH_PAGE_BYTES, DUCKDUCKGO_PROVIDER).await?;
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
        reject_blocked_duckduckgo(&response)?;
        let response: SearchResponse = read_json(response, DUCKDUCKGO_PROVIDER).await?;
        Ok(select_results(response.results, DUCKDUCKGO_PROVIDER))
    }

    async fn search_bing(&self, query: &str) -> Result<Vec<ImageResult>> {
        let response = self
            .client
            .get(BING_IMAGES_URL)
            .header(
                header::ACCEPT,
                "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8",
            )
            .header(header::ACCEPT_LANGUAGE, "en-US,en;q=0.5")
            .query(&[
                ("q", query),
                ("async", "1"),
                ("first", "1"),
                ("count", "35"),
                ("adlt", "strict"),
            ])
            .send()
            .await
            .map_err(|_| anyhow!("Bing Image Search fallback request failed"))?;
        let response = read_body(response, MAX_RESPONSE_BYTES, BING_PROVIDER).await?;
        let response = std::str::from_utf8(&response)
            .map_err(|_| anyhow!("Bing returned an invalid image response"))?;
        Ok(select_bing_results(response))
    }

    fn cached(&self, key: &str) -> Result<Option<Vec<ImageResult>>> {
        let cache = self
            .cache
            .lock()
            .map_err(|_| anyhow!("Image Search cache is unavailable"))?;
        Ok(cache
            .get(key)
            .filter(|entry| entry.stored_at.elapsed() < CACHE_TTL)
            .map(|entry| entry.results.clone()))
    }

    fn store(&self, key: String, results: Vec<ImageResult>) -> Result<()> {
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
                results,
            },
        );
        Ok(())
    }

    pub(crate) fn register_session(
        &self,
        id: u64,
        owner: u64,
        query: String,
        results: Vec<ImageResult>,
    ) -> Result<ImagePage> {
        let page = image_page(&query, &results, 0)
            .ok_or_else(|| anyhow!("No safe image results were found"))?;
        let mut sessions = self
            .sessions
            .lock()
            .map_err(|_| anyhow!("Image Search controls are unavailable"))?;
        sessions.retain(|_, session| session.touched_at.elapsed() < SESSION_TTL);
        if sessions.len() >= MAX_SESSIONS {
            sessions.clear();
        }
        sessions.insert(
            id,
            ImageSession {
                owner,
                query,
                results,
                current: 0,
                touched_at: Instant::now(),
            },
        );
        Ok(page)
    }

    pub(crate) fn advance_session(
        &self,
        id: u64,
        user: u64,
        direction: PageDirection,
    ) -> Result<SessionAdvance> {
        let mut sessions = self
            .sessions
            .lock()
            .map_err(|_| anyhow!("Image Search controls are unavailable"))?;
        sessions.retain(|_, session| session.touched_at.elapsed() < SESSION_TTL);
        let Some(session) = sessions.get_mut(&id) else {
            return Ok(SessionAdvance::Expired);
        };
        if session.owner != user {
            return Ok(SessionAdvance::NotOwner);
        }
        session.current = match direction {
            PageDirection::Previous => session
                .current
                .checked_sub(1)
                .unwrap_or(session.results.len() - 1),
            PageDirection::Next => (session.current + 1) % session.results.len(),
        };
        session.touched_at = Instant::now();
        let page = image_page(&session.query, &session.results, session.current)
            .ok_or_else(|| anyhow!("Image Search controls are unavailable"))?;
        Ok(SessionAdvance::Updated(page))
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

fn select_results(results: Vec<ApiImageResult>, provider: &'static str) -> Vec<ImageResult> {
    results
        .into_iter()
        .filter_map(|result| {
            let image_url = result.thumbnail.or(result.image)?;
            let image_url = safe_url(image_url, true)?;
            let page_url = result.url.and_then(|url| safe_url(url, false));
            Some(ImageResult {
                provider,
                title: result
                    .title
                    .filter(|title| !title.trim().is_empty())
                    .unwrap_or_else(|| "Image result".into()),
                image_url,
                page_url,
                source: result.source.filter(|source| !source.trim().is_empty()),
            })
        })
        .take(MAX_RESULTS)
        .collect()
}

#[derive(Deserialize)]
struct BingImageResult {
    #[serde(default)]
    t: Option<String>,
    #[serde(default)]
    murl: Option<String>,
    #[serde(default)]
    turl: Option<String>,
    #[serde(default)]
    purl: Option<String>,
}

fn select_bing_results(page: &str) -> Vec<ImageResult> {
    page.split("<a ")
        .skip(1)
        .filter_map(|element| {
            let tag = element.split_once('>')?.0;
            if !tag.contains("class=\"iusc\"") {
                return None;
            }
            let metadata = tag.split_once(" m=\"")?.1.split_once('"')?.0;
            let metadata = decode_html_attribute(metadata)?;
            let result: BingImageResult = serde_json::from_str(&metadata).ok()?;
            let image_url = safe_url(result.turl.or(result.murl)?, true)?;
            let page_url = result.purl.and_then(|url| safe_url(url, false));
            let source = page_url
                .as_deref()
                .and_then(|url| Url::parse(url).ok())
                .and_then(|url| url.host_str().map(str::to_owned));
            Some(ImageResult {
                provider: BING_PROVIDER,
                title: result
                    .t
                    .filter(|title| !title.trim().is_empty())
                    .unwrap_or_else(|| "Image result".into()),
                image_url,
                page_url,
                source,
            })
        })
        .take(MAX_RESULTS)
        .collect()
}

fn decode_html_attribute(value: &str) -> Option<String> {
    let mut decoded = String::with_capacity(value.len());
    let mut remaining = value;
    while let Some((before, entity)) = remaining.split_once('&') {
        decoded.push_str(before);
        let (entity, after) = entity.split_once(';')?;
        let character = match entity {
            "amp" => '&',
            "quot" => '"',
            "apos" | "#39" => '\'',
            "lt" => '<',
            "gt" => '>',
            value if value.starts_with("#x") => {
                char::from_u32(u32::from_str_radix(&value[2..], 16).ok()?)?
            }
            value if value.starts_with('#') => char::from_u32(value[1..].parse().ok()?)?,
            _ => return None,
        };
        decoded.push(character);
        remaining = after;
    }
    decoded.push_str(remaining);
    Some(decoded)
}

fn image_page(query: &str, results: &[ImageResult], index: usize) -> Option<ImagePage> {
    Some(ImagePage {
        query: query.to_owned(),
        result: results.get(index)?.clone(),
        index,
        total: results.len(),
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

fn reject_blocked_duckduckgo(response: &reqwest::Response) -> Result<()> {
    if matches!(
        response.status(),
        StatusCode::FORBIDDEN | StatusCode::TOO_MANY_REQUESTS
    ) {
        return Err(DuckDuckGoBlocked.into());
    }
    Ok(())
}

async fn read_body(
    mut response: reqwest::Response,
    limit: usize,
    provider: &str,
) -> Result<Vec<u8>> {
    let status = response.status();
    if status == StatusCode::TOO_MANY_REQUESTS {
        return Err(anyhow!(
            "{provider} Image Search is rate limited; try again later"
        ));
    }
    if !status.is_success() {
        return Err(anyhow!(
            "{provider} Image Search failed with status {}",
            status.as_u16()
        ));
    }

    let mut body = Vec::new();
    while let Some(chunk) = response
        .chunk()
        .await
        .map_err(|_| anyhow!("{provider} response could not be read"))?
    {
        ensure!(
            body.len().saturating_add(chunk.len()) <= limit,
            "{provider} response was too large"
        );
        body.extend_from_slice(&chunk);
    }
    Ok(body)
}

async fn read_json<T: for<'de> Deserialize<'de>>(
    response: reqwest::Response,
    provider: &str,
) -> Result<T> {
    let body = read_body(response, MAX_RESPONSE_BYTES, provider).await?;
    serde_json::from_slice(&body).map_err(|_| {
        anyhow!("{provider} returned an invalid response; its image endpoint may have changed")
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
            select_results(response.results, DUCKDUCKGO_PROVIDER),
            vec![ImageResult {
                provider: DUCKDUCKGO_PROVIDER,
                title: "Cat".into(),
                image_url: "https://external-content.duckduckgo.com/cat.jpg".into(),
                page_url: Some("https://example.com/cat".into()),
                source: Some("example.com".into()),
            }]
        );
    }

    #[test]
    fn image_sessions_are_owner_scoped_and_wrap() {
        let service = DuckDuckGoImageService::new().unwrap();
        let results = vec![
            ImageResult {
                provider: DUCKDUCKGO_PROVIDER,
                title: "One".into(),
                image_url: "https://example.com/one.jpg".into(),
                page_url: None,
                source: None,
            },
            ImageResult {
                provider: DUCKDUCKGO_PROVIDER,
                title: "Two".into(),
                image_url: "https://example.com/two.jpg".into(),
                page_url: None,
                source: None,
            },
        ];
        let first = service
            .register_session(7, 42, "cats".into(), results)
            .unwrap();
        assert_eq!(first.index, 0);
        assert!(matches!(
            service.advance_session(7, 99, PageDirection::Next).unwrap(),
            SessionAdvance::NotOwner
        ));
        let SessionAdvance::Updated(previous) = service
            .advance_session(7, 42, PageDirection::Previous)
            .unwrap()
        else {
            panic!("expected an updated page");
        };
        assert_eq!(previous.index, 1);
        let SessionAdvance::Updated(next) =
            service.advance_session(7, 42, PageDirection::Next).unwrap()
        else {
            panic!("expected an updated page");
        };
        assert_eq!(next.index, 0);
    }

    #[test]
    fn extracts_safe_bing_image_metadata() {
        let page = r#"<a class="iusc" m="{&quot;t&quot;:&quot;Orange &amp; white cat&quot;,&quot;murl&quot;:&quot;https://images.example/cat.jpg&quot;,&quot;turl&quot;:&quot;https://thumb.example/cat.jpg&quot;,&quot;purl&quot;:&quot;https://example.com/cat&quot;}">result</a>"#;
        assert_eq!(
            select_bing_results(page),
            vec![ImageResult {
                provider: BING_PROVIDER,
                title: "Orange & white cat".into(),
                image_url: "https://thumb.example/cat.jpg".into(),
                page_url: Some("https://example.com/cat".into()),
                source: Some("example.com".into()),
            }]
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
        assert!(!result.is_empty());
    }

    #[tokio::test]
    #[ignore = "uses the live unofficial Bing image endpoint"]
    async fn live_bing_smoke_test() {
        let service = DuckDuckGoImageService::new().unwrap();
        let result = service.search_bing("orange cat").await.unwrap();
        assert!(!result.is_empty());
        assert!(result.iter().all(|result| result.provider == BING_PROVIDER));
    }
}
