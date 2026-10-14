use crate::picker::PickerStore;
use anyhow::{Result, anyhow, ensure};
use reqwest::{Client, StatusCode, multipart};
use serde::Deserialize;
use serde_json::Value;
use std::time::Duration;
use url::Url;

const API_BASE: &str = "https://api.torbox.app/v1/api";
const MAX_RESPONSE_BYTES: usize = 4 * 1024 * 1024;

pub struct TorBoxService {
    client: Client,
    api_key: Option<Box<str>>,
    base_url: Box<str>,
    pub(crate) pickers: PickerStore,
}

#[derive(Debug, PartialEq, Eq)]
pub enum CachedTorrent {
    Added(u64),
    NotCached,
}

#[derive(Debug, PartialEq, Eq)]
pub enum StreamResult {
    Ready(String),
    PlanUnavailable,
    Unsupported,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
pub struct TorrentFile {
    pub id: u64,
    pub name: String,
    #[serde(default)]
    pub short_name: Option<String>,
    #[serde(default)]
    pub size: u64,
    #[serde(default)]
    pub infected: bool,
    #[serde(default)]
    pub mimetype: Option<String>,
}

#[derive(Deserialize)]
struct Envelope<T> {
    success: bool,
    #[serde(default)]
    error: Option<Value>,
    #[serde(default)]
    detail: String,
    data: Option<T>,
}

#[derive(Deserialize)]
struct AddedTorrent {
    torrent_id: u64,
}

#[derive(Deserialize)]
struct Torrent {
    #[serde(default)]
    files: Vec<TorrentFile>,
    #[serde(default)]
    download_present: bool,
    #[serde(default)]
    download_finished: bool,
    #[serde(default)]
    cached: bool,
}

#[derive(Deserialize)]
#[serde(untagged)]
enum TorrentData {
    One(Torrent),
    Many(Vec<Torrent>),
}

impl TorBoxService {
    pub fn from_env() -> Result<Self> {
        Self::new(
            std::env::var("TORBOX_API_KEY")
                .ok()
                .filter(|key| !key.trim().is_empty()),
            API_BASE,
        )
    }

    fn new(api_key: Option<String>, base_url: &str) -> Result<Self> {
        let client = Client::builder()
            .connect_timeout(Duration::from_secs(10))
            .timeout(Duration::from_secs(30))
            .redirect(reqwest::redirect::Policy::none())
            .no_proxy()
            .user_agent("momo-discord-bot/0.1")
            .build()?;
        Ok(Self {
            client,
            api_key: api_key.map(String::into_boxed_str),
            base_url: base_url.trim_end_matches('/').into(),
            pickers: PickerStore::default(),
        })
    }

    fn key(&self) -> Result<&str> {
        self.api_key
            .as_deref()
            .ok_or_else(|| anyhow!("TorBox is not configured by the bot operator"))
    }

    pub fn validate_magnet(magnet: &str) -> Result<()> {
        ensure!(magnet.len() <= 8192, "Magnet link is too long");
        let url = Url::parse(magnet).map_err(|_| anyhow!("Enter a valid magnet link"))?;
        ensure!(url.scheme() == "magnet", "Enter a valid magnet link");
        ensure!(
            url.query_pairs().any(|(key, value)| {
                key.eq_ignore_ascii_case("xt")
                    && (value.to_ascii_lowercase().starts_with("urn:btih:")
                        || value.to_ascii_lowercase().starts_with("urn:btmh:"))
            }),
            "Magnet link must contain a BitTorrent hash"
        );
        Ok(())
    }

    pub async fn add_cached(&self, magnet: &str) -> Result<CachedTorrent> {
        Self::validate_magnet(magnet)?;
        let form = multipart::Form::new()
            .text("magnet", magnet.to_owned())
            .text("seed", "3")
            .text("allow_zip", "true")
            .text("add_only_if_cached", "true");
        let response = self
            .client
            .post(format!("{}/torrents/createtorrent", self.base_url))
            .bearer_auth(self.key()?)
            .multipart(form)
            .send()
            .await
            .map_err(|_| anyhow!("TorBox request failed"))?;
        let envelope: Envelope<AddedTorrent> = read_json(response).await?;
        if !envelope.success {
            let code = error_text(&envelope);
            if code.contains("DOWNLOAD_NOT_CACHED") || code.contains("NOT_CACHED") {
                return Ok(CachedTorrent::NotCached);
            }
            return Err(api_error(&code));
        }
        Ok(CachedTorrent::Added(
            envelope
                .data
                .ok_or_else(|| anyhow!("TorBox returned an incomplete response"))?
                .torrent_id,
        ))
    }

    pub async fn files(&self, torrent_id: u64) -> Result<Vec<TorrentFile>> {
        let response = self
            .client
            .get(format!("{}/torrents/mylist", self.base_url))
            .bearer_auth(self.key()?)
            .query(&[
                ("id", torrent_id.to_string()),
                ("bypass_cache", "true".to_owned()),
            ])
            .send()
            .await
            .map_err(|_| anyhow!("TorBox request failed"))?;
        let envelope: Envelope<TorrentData> = read_json(response).await?;
        ensure!(envelope.success, "{}", api_error(&error_text(&envelope)));
        let torrent = match envelope
            .data
            .ok_or_else(|| anyhow!("TorBox returned an incomplete response"))?
        {
            TorrentData::One(torrent) => torrent,
            TorrentData::Many(mut torrents) if torrents.len() == 1 => torrents.remove(0),
            TorrentData::Many(_) => return Err(anyhow!("TorBox returned an unexpected file list")),
        };
        ensure!(
            torrent.cached && torrent.download_present && torrent.download_finished,
            "TorBox has not made this cached torrent ready yet; try again shortly"
        );
        ensure!(
            !torrent.files.is_empty(),
            "TorBox returned no downloadable files"
        );
        Ok(torrent.files)
    }

    pub async fn download_link(&self, torrent_id: u64, file_id: u64) -> Result<String> {
        self.request_download_link(torrent_id, Some(file_id), false)
            .await
    }

    pub async fn download_all_link(&self, torrent_id: u64) -> Result<String> {
        self.request_download_link(torrent_id, None, true).await
    }

    async fn request_download_link(
        &self,
        torrent_id: u64,
        file_id: Option<u64>,
        zip_link: bool,
    ) -> Result<String> {
        let mut query = vec![
            ("token", self.key()?.to_owned()),
            ("torrent_id", torrent_id.to_string()),
            ("zip_link", zip_link.to_string()),
            ("redirect", "false".to_owned()),
            ("append_name", "true".to_owned()),
        ];
        if let Some(file_id) = file_id {
            query.push(("file_id", file_id.to_string()));
        }
        let response = self
            .client
            .get(format!("{}/torrents/requestdl", self.base_url))
            .query(&query)
            .send()
            .await
            .map_err(|_| anyhow!("TorBox download-link request failed"))?;
        let envelope: Envelope<String> = read_json(response).await?;
        ensure!(envelope.success, "{}", api_error(&error_text(&envelope)));
        validate_generated_url(
            envelope
                .data
                .ok_or_else(|| anyhow!("TorBox returned no download link"))?,
        )
    }

    pub async fn create_stream(&self, torrent_id: u64, file_id: u64) -> Result<StreamResult> {
        let response = self
            .client
            .get(format!("{}/stream/createstream", self.base_url))
            .bearer_auth(self.key()?)
            .query(&[
                ("id", torrent_id.to_string()),
                ("file_id", file_id.to_string()),
                ("type", "torrent".to_owned()),
            ])
            .send()
            .await
            .map_err(|_| anyhow!("TorBox stream request failed"))?;
        let envelope: Envelope<Value> = read_json(response).await?;
        if !envelope.success {
            let code = error_text(&envelope);
            if code.contains("PLAN_RESTRICTED_FEATURE") {
                return Ok(StreamResult::PlanUnavailable);
            }
            if code.contains("BOZO_FILE")
                || code.contains("STREAM_INFO_ERROR")
                || code.contains("LINK_OFFLINE")
            {
                return Ok(StreamResult::Unsupported);
            }
            return Err(api_error(&code));
        }
        let data = envelope
            .data
            .ok_or_else(|| anyhow!("TorBox returned an incomplete stream response"))?;
        if let Some(url) = find_url(&data) {
            return Ok(StreamResult::Ready(validate_generated_url(url)?));
        }
        let token = find_string_key(&data, "presigned_token")
            .ok_or_else(|| anyhow!("TorBox created a stream but returned no playback token"))?;
        self.get_stream_data(&token).await
    }

    async fn get_stream_data(&self, presigned_token: &str) -> Result<StreamResult> {
        let response = self
            .client
            .get(format!("{}/stream/getstreamdata", self.base_url))
            .query(&[
                ("presigned_token", presigned_token.to_owned()),
                ("token", self.key()?.to_owned()),
            ])
            .send()
            .await
            .map_err(|_| anyhow!("TorBox stream-data request failed"))?;
        let envelope: Envelope<Value> = read_json(response).await?;
        if !envelope.success {
            let code = error_text(&envelope);
            if code.contains("PLAN_RESTRICTED_FEATURE") {
                return Ok(StreamResult::PlanUnavailable);
            }
            if code.contains("ITEM_NOT_FOUND") || code.contains("LINK_OFFLINE") {
                return Ok(StreamResult::Unsupported);
            }
            return Err(api_error(&code));
        }
        let data = envelope
            .data
            .ok_or_else(|| anyhow!("TorBox returned an incomplete stream response"))?;
        let url =
            find_url(&data).ok_or_else(|| anyhow!("TorBox returned no playable stream link"))?;
        Ok(StreamResult::Ready(validate_generated_url(url)?))
    }
}

async fn read_json<T: for<'de> Deserialize<'de>>(mut response: reqwest::Response) -> Result<T> {
    let status = response.status();
    let mut body = Vec::new();
    while let Some(chunk) = response
        .chunk()
        .await
        .map_err(|_| anyhow!("TorBox response could not be read"))?
    {
        ensure!(
            body.len().saturating_add(chunk.len()) <= MAX_RESPONSE_BYTES,
            "TorBox response was too large"
        );
        body.extend_from_slice(&chunk);
    }
    if status == StatusCode::TOO_MANY_REQUESTS {
        return Err(anyhow!("TorBox is rate limited; try again shortly"));
    }
    match serde_json::from_slice(&body) {
        Ok(value) => Ok(value),
        Err(_) if !status.is_success() => Err(anyhow!(
            "TorBox request failed with status {}",
            status.as_u16()
        )),
        Err(_) => Err(anyhow!("TorBox returned an invalid response")),
    }
}

fn error_text<T>(envelope: &Envelope<T>) -> String {
    let mut text = envelope.detail.to_ascii_uppercase();
    if let Some(error) = &envelope.error {
        text.push(' ');
        text.push_str(&error.to_string().to_ascii_uppercase());
    }
    text
}

fn api_error(text: &str) -> anyhow::Error {
    if text.contains("AUTH_ERROR") || text.contains("BAD_TOKEN") || text.contains("NO_AUTH") {
        anyhow!("TorBox authentication failed; ask the bot operator to check its API key")
    } else if text.contains("DOWNLOAD_NOT_CACHED") || text.contains("NOT_CACHED") {
        anyhow!("This torrent is not cached by TorBox")
    } else {
        anyhow!("TorBox could not complete the request")
    }
}

fn validate_generated_url(value: String) -> Result<String> {
    let url = Url::parse(&value).map_err(|_| anyhow!("TorBox returned an invalid link"))?;
    ensure!(url.scheme() == "https", "TorBox returned an unsafe link");
    Ok(url.into())
}

fn find_url(value: &Value) -> Option<String> {
    match value {
        Value::String(value) if value.starts_with("https://") => Some(value.clone()),
        Value::Object(values) => {
            for key in [
                "hls_url",
                "stream_url",
                "playlist_url",
                "url",
                "playlist",
                "download_url",
            ] {
                if let Some(url) = values.get(key).and_then(find_url) {
                    return Some(url);
                }
            }
            None
        }
        Value::Array(values) => values.iter().find_map(find_url),
        _ => None,
    }
}

fn find_string_key(value: &Value, wanted: &str) -> Option<String> {
    match value {
        Value::Object(values) => values
            .get(wanted)
            .and_then(Value::as_str)
            .map(str::to_owned)
            .or_else(|| {
                values
                    .values()
                    .find_map(|value| find_string_key(value, wanted))
            }),
        Value::Array(values) => values
            .iter()
            .find_map(|value| find_string_key(value, wanted)),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validates_bittorrent_magnets_without_echoing_them() {
        assert!(
            TorBoxService::validate_magnet(
                "magnet:?xt=urn:btih:0123456789abcdef0123456789abcdef01234567&dn=test"
            )
            .is_ok()
        );
        assert!(TorBoxService::validate_magnet("https://example.com/file").is_err());
        assert!(TorBoxService::validate_magnet("magnet:?dn=no-hash").is_err());
    }

    #[test]
    fn extracts_supported_stream_response_shapes() {
        assert_eq!(
            find_url(&serde_json::json!({
                "domain": "https://stream.example",
                "hls_url": "https://stream.example/video.m3u8",
                "search_metadata": {"image": "https://images.example/poster.jpg"}
            })),
            Some("https://stream.example/video.m3u8".into())
        );
        assert_eq!(
            find_url(&serde_json::json!({
                "domain": "https://stream.example",
                "webdav_url": "https://files.example/video.mkv",
                "search_metadata": {"image": "https://images.example/poster.jpg"}
            })),
            None
        );
        assert_eq!(
            find_url(&serde_json::json!({"stream_url": "https://stream.example/video.m3u8"})),
            Some("https://stream.example/video.m3u8".into())
        );
        assert_eq!(
            find_url(&serde_json::json!("https://stream.example/video.mp4")),
            Some("https://stream.example/video.mp4".into())
        );
        assert_eq!(
            find_string_key(
                &serde_json::json!({"stream": {"presigned_token": "opaque"}}),
                "presigned_token"
            ),
            Some("opaque".into())
        );
    }

    #[test]
    fn generated_links_must_use_https() {
        assert!(validate_generated_url("https://cdn.example/file".into()).is_ok());
        assert!(validate_generated_url("http://cdn.example/file".into()).is_err());
    }

    #[test]
    fn generated_links_are_normalized_for_discord() {
        assert_eq!(
            validate_generated_url("https://cdn.example/a file.mkv".into()).unwrap(),
            "https://cdn.example/a%20file.mkv"
        );
    }
}
