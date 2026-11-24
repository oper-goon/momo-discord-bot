//! Unofficial Easynews members 3.0 web interface; see docs/music-providers.md.
use super::{client, json};
use crate::{
    matching::{Metadata, audio_extension, confidence},
    source::{AudioSourceProvider, PlayableTrack, pinned_client, stream_authenticated},
};
use anyhow::{Result, anyhow, ensure};
use async_trait::async_trait;
use bot_core::TrackRef;
use serde_json::Value;
use tokio::sync::Semaphore;
use url::Url;

pub struct Easynews {
    search_url: String,
    client: reqwest::Client,
    credentials: Option<(String, String)>,
    requests: Semaphore,
    minimum: f64,
}

impl Easynews {
    async fn media_url(&self, mut url: Url) -> Result<String> {
        // Easynews's members endpoint redirects to its regional download farm.
        // Follow explicitly, never through reqwest's automatic redirect policy.
        for _ in 0..=3 {
            ensure!(trusted(&url), "Unsafe Easynews media origin");
            let (username, password) = self.auth()?;
            let response = pinned_client(&url)
                .await?
                .get(url.as_str())
                .basic_auth(username, Some(password))
                .header(reqwest::header::RANGE, "bytes=0-0")
                .send()
                .await
                .map_err(|_| anyhow!("Easynews media request failed"))?;
            if response.status().is_success() {
                return Ok(url.into());
            }
            if response.status().is_redirection() {
                let location = response
                    .headers()
                    .get(reqwest::header::LOCATION)
                    .and_then(|value| value.to_str().ok())
                    .ok_or_else(|| anyhow!("Invalid Easynews media redirect"))?;
                url = checked_redirect(&url, location)?;
            } else {
                match response.status().as_u16() {
                    401 | 403 => anyhow::bail!("Easynews authentication failed"),
                    429 => {
                        let retry = response
                            .headers()
                            .get(reqwest::header::RETRY_AFTER)
                            .and_then(|value| value.to_str().ok())
                            .and_then(|value| value.parse::<u64>().ok())
                            .unwrap_or(120);
                        return Err(super::RateLimited(retry.min(86400)).into());
                    }
                    _ => anyhow::bail!("Easynews media unavailable"),
                }
            }
        }
        anyhow::bail!("Easynews media exceeded redirect limit")
    }
    pub fn new(minimum: f64) -> Self {
        let credentials = std::env::var("EASYNEWS_USERNAME")
            .ok()
            .zip(std::env::var("EASYNEWS_PASSWORD").ok())
            .filter(|(u, p)| !u.is_empty() && !p.is_empty());
        Self {
            search_url: "https://members.easynews.com/3.0/api/search".into(),
            client: client(),
            credentials,
            requests: Semaphore::new(2),
            minimum,
        }
    }
    fn auth(&self) -> Result<(&str, &str)> {
        self.credentials
            .as_ref()
            .map(|(u, p)| (u.as_str(), p.as_str()))
            .ok_or_else(|| anyhow!("Easynews credentials are not configured"))
    }
    async fn lookup(&self, wanted: &Metadata, user: u64) -> Result<Vec<TrackRef>> {
        let _permit = self.requests.acquire().await?;
        let (username, password) = self.auth()?;
        let value = json(
            self.client
                .get(&self.search_url)
                .basic_auth(username, Some(password))
                .query(&[
                    ("gps", wanted.query()),
                    ("pno", "1".into()),
                    ("u", "1".into()),
                    ("s1", "relevance".into()),
                    ("s1d", "-".into()),
                    ("fty[]", "AUDIO".into()),
                ])
                .send()
                .await,
        )
        .await?;
        Self::results(&value, wanted, user, self.minimum)
    }
    fn results(value: &Value, wanted: &Metadata, user: u64, minimum: f64) -> Result<Vec<TrackRef>> {
        let rows = value
            .get("data")
            .and_then(Value::as_array)
            .ok_or_else(|| anyhow!("Invalid Easynews search response"))?;
        let base = value
            .get("downURL")
            .and_then(Value::as_str)
            .unwrap_or("https://members.easynews.com/dl");
        let mut base = Url::parse(base).map_err(|_| anyhow!("Invalid Easynews media origin"))?;
        ensure!(trusted(&base), "Unsafe Easynews media origin");
        let farm = scalar(value.get("dlFarm")).unwrap_or_else(|| "auto".into());
        let port = scalar(value.get("dlPort")).unwrap_or_else(|| "auto".into());
        let mut matches = Vec::new();
        for row in rows.iter().take(250) {
            let field = |name: &str, number: usize| {
                row.get(name)
                    .or_else(|| row.get(number.to_string()))
                    .or_else(|| row.as_array().and_then(|r| r.get(number)))
            };
            let Some(hash) = field("hash", 0).and_then(Value::as_str) else {
                continue;
            };
            let Some(filename) = field("fn", 10).and_then(Value::as_str) else {
                continue;
            };
            let Some(ext) = row
                .get("extension")
                .or_else(|| field("ext", 11))
                .and_then(Value::as_str)
            else {
                continue;
            };
            let ext = ext.trim_start_matches('.');
            if audio_extension(ext).is_none()
                || hash.is_empty()
                || !hash.chars().all(|c| c.is_ascii_hexdigit())
            {
                continue;
            }
            let suffix = row.get("id").and_then(Value::as_str).unwrap_or("");
            if !suffix.chars().all(|c| c.is_ascii_alphanumeric()) {
                continue;
            }
            let size = row
                .get("rawSize")
                .or_else(|| row.get("size"))
                .and_then(Value::as_u64);
            if size.is_some_and(|s| s == 0 || s > 64 * 1024 * 1024) {
                continue;
            }
            let duration = row
                .get("runtime")
                .and_then(Value::as_f64)
                .filter(|s| s.is_finite() && *s > 0.0)
                .map(|s| (s * 1000.0) as u64);
            let mut metadata = Metadata::from_title(filename, duration);
            metadata.format = Some(ext.into());
            metadata.bitrate = row.get("bps").and_then(Value::as_u64);
            let score = confidence(wanted, &metadata);
            if score < minimum {
                continue;
            }
            // Append encoded segments; never trust returned URLs or filename traversal.
            base.set_query(None);
            let mut url = base.clone();
            url.path_segments_mut()
                .map_err(|_| anyhow!("Invalid Easynews path"))?
                .pop_if_empty()
                .push(&farm)
                .push(&port)
                .push(&format!("{hash}{suffix}.{ext}"))
                .push(&format!("{filename}.{ext}"));
            matches.push((
                score,
                metadata.bitrate.unwrap_or(0),
                TrackRef {
                    provider: "easynews".into(),
                    canonical_url: url.into(),
                    title: filename.chars().take(200).collect(),
                    duration_ms: duration,
                    requested_by: user.to_string(),
                },
            ));
        }
        matches.sort_by(|a, b| b.0.total_cmp(&a.0).then_with(|| b.1.cmp(&a.1)));
        Ok(matches
            .into_iter()
            .take(5)
            .map(|(_, _, track)| track)
            .collect())
    }
}

fn scalar(value: Option<&Value>) -> Option<String> {
    match value? {
        Value::String(s) => Some(s.clone()),
        Value::Number(n) => Some(n.to_string()),
        _ => None,
    }
}
fn trusted(url: &Url) -> bool {
    url.scheme() == "https"
        && url.username().is_empty()
        && url.password().is_none()
        && url.port().is_none_or(|p| p == 443)
        && url
            .host_str()
            .is_some_and(|h| h == "members.easynews.com" || h.ends_with(".easynews.com"))
}
fn checked_redirect(current: &Url, location: &str) -> Result<Url> {
    let url = current
        .join(location)
        .map_err(|_| anyhow!("Invalid Easynews media redirect"))?;
    ensure!(trusted(&url), "Unsafe Easynews media redirect");
    Ok(url)
}

#[async_trait]
impl AudioSourceProvider for Easynews {
    fn id(&self) -> &'static str {
        "easynews"
    }
    fn accepts(&self, input: &str) -> bool {
        !input.contains("://")
    }
    async fn inspect(&self, input: &str, user: u64) -> Result<Vec<TrackRef>> {
        self.search(&Metadata::from_title(input, None), user).await
    }
    async fn search(&self, wanted: &Metadata, user: u64) -> Result<Vec<TrackRef>> {
        let mut tracks = self.lookup(wanted, user).await?;
        // Only a stable search marker reaches queues/playlists. Private links are ephemeral.
        for track in &mut tracks {
            track.canonical_url = "easynews:search".into();
        }
        Ok(tracks)
    }
    async fn resolve(&self, track: &TrackRef) -> Result<PlayableTrack> {
        let wanted = Metadata::from_title(&track.title, track.duration_ms);
        let tracks = self
            .lookup(&wanted, track.requested_by.parse().unwrap_or(0))
            .await?;
        let candidate = tracks
            .first()
            .ok_or_else(|| anyhow!("Easynews content unavailable"))?;
        let url = Url::parse(&candidate.canonical_url)
            .map_err(|_| anyhow!("Invalid Easynews reference"))?;
        ensure!(trusted(&url), "Unsafe Easynews reference");
        let url = self.media_url(url).await?;
        stream_authenticated(&url, Some(self.auth()?)).await
    }
    async fn health(&self) -> bool {
        self.credentials.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn regional_redirects_require_https_easynews_origin_without_credentials() {
        let origin = Url::parse("https://members.easynews.com/dl/example").unwrap();
        assert!(checked_redirect(&origin, "https://iad-dl-06.easynews.com/file").is_ok());
        for location in [
            "https://evil.test/file",
            "http://iad-dl-06.easynews.com/file",
            "https://easynews.com.evil.test/file",
            "https://user:secret@iad-dl-06.easynews.com/file",
            "https://iad-dl-06.easynews.com:8443/file",
            "https://127.0.0.1/file",
        ] {
            assert!(checked_redirect(&origin, location).is_err());
        }
    }
    #[tokio::test]
    async fn mocked_search_authentication_and_private_reference() {
        let (base, server) = super::super::mock_api(vec![("200 OK", serde_json::json!({"data":[{"hash":"abcdef","id":"123","fn":"Artist - Song","extension":".mp3"}]}).to_string())]).await;
        let provider = Easynews {
            search_url: base,
            client: client(),
            credentials: Some(("test-user".into(), "test-password".into())),
            requests: Semaphore::new(2),
            minimum: 0.85,
        };
        let tracks = provider.inspect("Artist - Song", 1).await.unwrap();
        assert_eq!(tracks[0].canonical_url, "easynews:search");
        let requests = server.await.unwrap();
        assert!(requests[0].contains("fty%5B%5D=AUDIO"));
        assert!(requests[0].to_lowercase().contains("authorization: basic "));
        let (base, server) =
            super::super::mock_api(vec![("401 Unauthorized", "private response".into())]).await;
        let provider = Easynews {
            search_url: base,
            ..provider
        };
        assert!(
            provider
                .inspect("Artist - Song", 1)
                .await
                .unwrap_err()
                .to_string()
                .contains("authentication")
        );
        server.await.unwrap();
    }
    #[test]
    fn standalone_audio_only_and_matching() {
        let value = serde_json::json!({"data":[
            {"hash":"abcdef", "id":"123", "fn":"Artist - Song", "extension":".mp3", "runtime":200},
            {"hash":"abcdef", "fn":"Artist - Song", "extension":".zip"},
            {"hash":"abcdef", "fn":"Artist - Song live", "extension":".flac"}
        ]});
        let tracks = Easynews::results(
            &value,
            &Metadata::from_title("Artist - Song", Some(200_000)),
            1,
            0.85,
        )
        .unwrap();
        assert_eq!(tracks.len(), 1);
        assert!(!tracks[0].canonical_url.contains("undefined"));
        assert!(!trusted(
            &Url::parse("https://easynews.com.evil.test/a").unwrap()
        ));
        assert!(
            Easynews::results(
                &serde_json::json!({"data":[],"downURL":"http://localhost"}),
                &Metadata::default(),
                1,
                0.85
            )
            .is_err()
        );
    }
}
