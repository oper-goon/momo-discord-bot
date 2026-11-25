//! Read-only discovery in the operator's library. No download-creation API exists here.
use super::{client, data, json};
use crate::{
    matching::{Metadata, audio_extension, confidence},
    source::{AudioSourceProvider, PlayableTrack, stream_authenticated},
};
use anyhow::{Result, anyhow, ensure};
use async_trait::async_trait;
use bot_core::TrackRef;
use serde_json::Value;
use tokio::sync::Semaphore;

pub struct Torbox {
    base: String,
    client: reqwest::Client,
    key: Option<String>,
    requests: Semaphore,
    minimum: f64,
}

impl Torbox {
    pub fn new(minimum: f64) -> Self {
        Self {
            base: "https://api.torbox.app/v1/api/torrents".into(),
            client: client(),
            key: std::env::var("TORBOX_API_KEY")
                .ok()
                .filter(|s| !s.is_empty()),
            requests: Semaphore::new(1),
            minimum,
        }
    }
    fn key(&self) -> Result<&str> {
        self.key
            .as_deref()
            .ok_or_else(|| anyhow!("TorBox credentials are not configured"))
    }
    async fn get(&self, endpoint: &str, query: &[(&str, String)]) -> Result<Value> {
        json(
            self.client
                .get(format!("{}/{endpoint}", self.base))
                .bearer_auth(self.key()?)
                .query(query)
                .send()
                .await,
        )
        .await
    }
    fn ready(torrent: &Value) -> bool {
        ["cached", "download_present", "download_finished"]
            .iter()
            .all(|key| torrent.get(key).and_then(Value::as_bool) == Some(true))
    }
    fn candidates(torrents: &Value, wanted: &Metadata, user: u64, minimum: f64) -> Vec<TrackRef> {
        let Some(torrents) = torrents.as_array() else {
            return vec![];
        };
        let mut matches = Vec::new();
        for torrent in torrents.iter().take(1000).filter(|t| Self::ready(t)) {
            let Some(id) = torrent.get("id").and_then(Value::as_u64) else {
                continue;
            };
            let Some(hash) = torrent.get("hash").and_then(Value::as_str) else {
                continue;
            };
            if !valid_hash(hash) {
                continue;
            }
            let Some(files) = torrent.get("files").and_then(Value::as_array) else {
                continue;
            };
            for file in files.iter().take(1000) {
                if file.get("infected").and_then(Value::as_bool) != Some(false) {
                    continue;
                }
                let Some(file_id) = file.get("id").and_then(Value::as_u64) else {
                    continue;
                };
                let Some(name) = file.get("name").and_then(Value::as_str) else {
                    continue;
                };
                let Some(ext) = audio_extension(name) else {
                    continue;
                };
                let size = file.get("size").and_then(Value::as_u64).unwrap_or(0);
                if size == 0 || size > 64 * 1024 * 1024 {
                    continue;
                }
                let name = name.rsplit('/').next().unwrap_or(name);
                let title = name.rsplit_once('.').map_or(name, |(s, _)| s);
                let mut metadata = Metadata::from_title(title, None);
                metadata.format = Some(ext.into());
                let score = confidence(wanted, &metadata);
                if score < minimum {
                    continue;
                }
                matches.push((
                    score,
                    TrackRef {
                        provider: "torbox".into(),
                        canonical_url: format!("torbox:{id}:{file_id}:{hash}"),
                        title: title.chars().take(200).collect(),
                        duration_ms: None,
                        requested_by: user.to_string(),
                    },
                ));
            }
        }
        matches.sort_by(|a, b| b.0.total_cmp(&a.0));
        matches.into_iter().take(5).map(|(_, t)| t).collect()
    }
    async fn cached(&self, hash: &str) -> Result<bool> {
        let value = self
            .get(
                "checkcached",
                &[
                    ("hash", hash.into()),
                    ("format", "object".into()),
                    ("list_files", "true".into()),
                ],
            )
            .await?;
        let cached = data(&value)?;
        // Require an explicit entry for this exact hash with file metadata.
        Ok(cached
            .get(hash)
            .and_then(|entry| entry.get("files"))
            .and_then(Value::as_array)
            .is_some_and(|files| !files.is_empty()))
    }
}

fn valid_hash(hash: &str) -> bool {
    [40, 64].contains(&hash.len()) && hash.chars().all(|c| c.is_ascii_hexdigit())
}

#[async_trait]
impl AudioSourceProvider for Torbox {
    fn id(&self) -> &'static str {
        "torbox"
    }
    fn accepts(&self, input: &str) -> bool {
        !input.contains("://")
    }
    async fn inspect(&self, input: &str, user: u64) -> Result<Vec<TrackRef>> {
        self.search(&Metadata::from_title(input, None), user).await
    }
    async fn search(&self, wanted: &Metadata, user: u64) -> Result<Vec<TrackRef>> {
        let _permit = self.requests.acquire().await?;
        let response = self.get("mylist", &[]).await?;
        Ok(Self::candidates(
            data(&response)?,
            wanted,
            user,
            self.minimum,
        ))
    }
    async fn resolve(&self, track: &TrackRef) -> Result<PlayableTrack> {
        let _permit = self.requests.acquire().await?;
        let parts: Vec<_> = track.canonical_url.split(':').collect();
        ensure!(
            parts.len() == 4 && parts[0] == "torbox" && valid_hash(parts[3]),
            "Invalid TorBox reference"
        );
        let id = parts[1]
            .parse::<u64>()
            .map_err(|_| anyhow!("Invalid TorBox reference"))?;
        let file_id = parts[2]
            .parse::<u64>()
            .map_err(|_| anyhow!("Invalid TorBox reference"))?;
        ensure!(
            self.cached(parts[3]).await?,
            "TorBox content is not confirmed cached"
        );
        let response = self
            .get(
                "mylist",
                &[("id", id.to_string()), ("bypass_cache", "true".into())],
            )
            .await?;
        let torrent = data(&response)?;
        let torrent = if let Some(list) = torrent.as_array() {
            ensure!(list.len() == 1, "TorBox content unavailable");
            &list[0]
        } else {
            torrent
        };
        ensure!(
            Self::ready(torrent) && torrent.get("hash").and_then(Value::as_str) == Some(parts[3]),
            "TorBox content is not ready"
        );
        let file = torrent
            .get("files")
            .and_then(Value::as_array)
            .and_then(|files| {
                files
                    .iter()
                    .find(|file| file.get("id").and_then(Value::as_u64) == Some(file_id))
            })
            .ok_or_else(|| anyhow!("TorBox file unavailable"))?;
        ensure!(
            file.get("infected").and_then(Value::as_bool) == Some(false)
                && file
                    .get("name")
                    .and_then(Value::as_str)
                    .and_then(audio_extension)
                    .is_some()
                && file
                    .get("size")
                    .and_then(Value::as_u64)
                    .is_some_and(|size| size > 0 && size <= 64 * 1024 * 1024),
            "TorBox file is not eligible audio"
        );
        let response = self
            .get(
                "requestdl",
                &[
                    ("token", self.key()?.into()),
                    ("torrent_id", id.to_string()),
                    ("file_id", file_id.to_string()),
                    ("zip_link", "false".into()),
                    ("redirect", "false".into()),
                ],
            )
            .await?;
        let url = data(&response)?
            .as_str()
            .ok_or_else(|| anyhow!("Invalid TorBox download response"))?;
        ensure!(url.starts_with("https://"), "Unsafe TorBox media URL");
        stream_authenticated(url, None).await
    }
    async fn health(&self) -> bool {
        self.key.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn unconfirmed_cache_never_requests_retrieval() {
        let hash = "0123456789abcdef0123456789abcdef01234567";
        for response in [
            serde_json::json!({"success":true,"data":{}}),
            serde_json::json!({"success":false,"data":null}),
            serde_json::json!({"success":true,"data":{hash:{"files":[]}}}),
        ] {
            let (base, server) =
                super::super::mock_api(vec![("200 OK", response.to_string())]).await;
            let provider = Torbox {
                base,
                key: Some("test-secret".into()),
                client: client(),
                requests: Semaphore::new(1),
                minimum: 0.85,
            };
            let track = TrackRef {
                provider: "torbox".into(),
                canonical_url: format!("torbox:1:2:{hash}"),
                title: "Artist - Song".into(),
                duration_ms: None,
                requested_by: "1".into(),
            };
            assert!(provider.resolve(&track).await.is_err());
            let requests = server.await.unwrap();
            assert_eq!(requests.len(), 1);
            assert!(requests[0].starts_with("GET /checkcached?"));
        }
    }
    #[tokio::test]
    async fn cached_file_requests_only_selected_audio_link() {
        let hash = "0123456789abcdef0123456789abcdef01234567";
        let (base, server) = super::super::mock_api(vec![
            ("200 OK", serde_json::json!({"success":true,"data":{hash:{"files":[{"name":"Artist - Song.mp3"}]}}}).to_string()),
            ("200 OK", serde_json::json!({"success":true,"data":{"hash":hash,"id":1,"cached":true,"download_present":true,"download_finished":true,"files":[{"id":2,"name":"Artist - Song.mp3","size":100,"infected":false}]}}).to_string()),
            // Expired link service: safe failure, without connecting to a media host.
            ("410 Gone", "private-expired-link".into()),
        ]).await;
        let provider = Torbox {
            base,
            key: Some("test-secret".into()),
            client: client(),
            requests: Semaphore::new(1),
            minimum: 0.85,
        };
        let track = TrackRef {
            provider: "torbox".into(),
            canonical_url: format!("torbox:1:2:{hash}"),
            title: "Artist - Song".into(),
            duration_ms: None,
            requested_by: "1".into(),
        };
        let error = provider.resolve(&track).await.err().unwrap().to_string();
        assert!(!error.contains("test-secret"));
        let requests = server.await.unwrap();
        assert!(requests.iter().all(|r| r.starts_with("GET ")));
        assert!(requests[2].contains("file_id=2") && requests[2].contains("zip_link=false"));
    }
    #[test]
    fn rejects_uncached_incomplete_infected_and_archives() {
        let mut torrent = serde_json::json!({"id":1,"hash":"0123456789abcdef0123456789abcdef01234567", "cached":true,"download_present":true,"download_finished":true,"files":[{"id":2,"name":"Artist - Song.mp3","size":100,"infected":false}]});
        let wanted = Metadata::from_title("Artist - Song", None);
        assert_eq!(
            Torbox::candidates(&serde_json::json!([torrent.clone()]), &wanted, 1, 0.85).len(),
            1
        );
        torrent["cached"] = Value::Bool(false);
        assert!(Torbox::candidates(&serde_json::json!([torrent]), &wanted, 1, 0.85).is_empty());
        assert!(!Torbox::ready(&serde_json::json!({"cached":true})));
        assert!(!valid_hash("not-a-hash"));
    }
}
