use anyhow::{Context, Result, ensure};
use async_trait::async_trait;
use bot_core::{TrackRef, config::MusicConfig};
use serde_json::Value;
use std::{
    net::{IpAddr, SocketAddr},
    process::Stdio,
    sync::Arc,
    time::Duration,
};
use tokio::{io::AsyncReadExt, process::Command, sync::Semaphore};
use url::Url;

fn classify_resolver_diagnostics(diagnostics: &[u8]) -> &'static str {
    let text = String::from_utf8_lossy(diagnostics).to_ascii_lowercase();
    if text.contains("sign in") || text.contains("captcha") || text.contains("bot") {
        "source access challenge"
    } else if text.contains("private video") || text.contains("members-only") {
        "source unavailable or restricted"
    } else if text.contains("requested format") {
        "requested audio format unavailable"
    } else if text.contains("timed out") || text.contains("timeout") {
        "source request timed out"
    } else {
        "resolver exited unsuccessfully"
    }
}

pub struct PlayableTrack {
    pub input: songbird::input::Input,
}
#[async_trait]
pub trait AudioSourceProvider: Send + Sync {
    fn id(&self) -> &'static str;
    fn accepts(&self, input: &str) -> bool;
    async fn inspect(&self, input: &str, user: u64) -> Result<Vec<TrackRef>>;
    async fn resolve(&self, track: &TrackRef) -> Result<PlayableTrack>;
    async fn health(&self) -> bool;
}

pub fn public_ip(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(v) => {
            let o = v.octets();
            !(v.is_private()
                || v.is_loopback()
                || v.is_link_local()
                || v.is_multicast()
                || v.is_broadcast()
                || v.is_documentation()
                || v.is_unspecified()
                || o[0] == 0
                || o[0] >= 240
                || (o[0] == 100 && (64..=127).contains(&o[1]))
                || (o[0] == 198 && (o[1] == 18 || o[1] == 19))
                || (o[0] == 192 && o[1] == 0))
        }
        IpAddr::V6(v) => {
            if let Some(v4) = v.to_ipv4_mapped() {
                return public_ip(IpAddr::V4(v4));
            }
            let s = v.segments();
            (s[0] & 0xe000) == 0x2000
                && !(s[0] == 0x2001 && (s[1] <= 0x1ff || s[1] == 0xdb8))
                && s[0] != 0x2002
        }
    }
}
pub fn parse_url(value: &str) -> Result<Url> {
    let url = Url::parse(value).context("Invalid media URL")?;
    ensure!(
        ["http", "https"].contains(&url.scheme()),
        "Only public HTTP(S) URLs are accepted"
    );
    ensure!(
        url.username().is_empty() && url.password().is_none(),
        "URLs with credentials are not accepted"
    );
    ensure!(
        url.port().is_none_or(|p| p == 80 || p == 443),
        "Only standard HTTP(S) ports are accepted"
    );
    ensure!(url.host_str().is_some(), "URL requires a hostname");
    Ok(url)
}
async fn pinned_client(url: &Url) -> Result<reqwest::Client> {
    let host = url.host_str().context("URL requires a hostname")?;
    let port = url.port_or_known_default().context("Invalid URL port")?;
    let addresses: Vec<SocketAddr> = tokio::time::timeout(
        Duration::from_secs(5),
        tokio::net::lookup_host((host, port)),
    )
    .await
    .context("DNS lookup timed out")??
    .collect();
    ensure!(
        !addresses.is_empty() && addresses.iter().all(|a| public_ip(a.ip())),
        "Media URLs must resolve exclusively to public addresses"
    );
    // Pin the checked DNS result to prevent rebinding. Redirects are rejected, not followed.
    Ok(reqwest::Client::builder()
        .no_proxy()
        .redirect(reqwest::redirect::Policy::none())
        .connect_timeout(Duration::from_secs(10))
        .timeout(Duration::from_secs(120))
        .resolve_to_addrs(host, &addresses)
        .build()?)
}
async fn download(url: &str) -> Result<PlayableTrack> {
    let url = parse_url(url)?;
    let client = pinned_client(&url).await?;
    let mut response = client
        .get(url)
        .send()
        .await
        .map_err(|_| anyhow::anyhow!("Media download failed"))?;
    ensure!(
        response.status().is_success(),
        "Media source rejected the request or redirected; use its final public URL"
    );
    const MAX: u64 = 64 * 1024 * 1024;
    ensure!(
        response.content_length().is_none_or(|n| n <= MAX),
        "Audio exceeds the 64 MiB limit"
    );
    let mut bytes = Vec::new();
    while let Some(chunk) = response
        .chunk()
        .await
        .map_err(|_| anyhow::anyhow!("Media download interrupted"))?
    {
        ensure!(
            bytes.len() + chunk.len() <= MAX as usize,
            "Audio exceeds the 64 MiB limit"
        );
        bytes.extend_from_slice(&chunk);
    }
    ensure!(!bytes.is_empty(), "Media source returned an empty file");
    Ok(PlayableTrack {
        input: bytes.into(),
    })
}
pub struct DirectHttpSource;
#[async_trait]
impl AudioSourceProvider for DirectHttpSource {
    fn id(&self) -> &'static str {
        "http"
    }
    fn accepts(&self, input: &str) -> bool {
        parse_url(input).is_ok()
    }
    async fn inspect(&self, input: &str, user: u64) -> Result<Vec<TrackRef>> {
        let url = parse_url(input)?;
        pinned_client(&url).await?;
        Ok(vec![TrackRef {
            provider: self.id().into(),
            canonical_url: url.to_string(),
            title: url
                .path_segments()
                .and_then(|mut s| s.next_back())
                .filter(|s| !s.is_empty())
                .unwrap_or("Direct audio")
                .chars()
                .take(200)
                .collect(),
            duration_ms: None,
            requested_by: user.to_string(),
        }])
    }
    async fn resolve(&self, track: &TrackRef) -> Result<PlayableTrack> {
        download(&track.canonical_url).await
    }
    async fn health(&self) -> bool {
        true
    }
}
pub struct YtDlpSource {
    config: MusicConfig,
    global: Arc<Semaphore>,
}
impl YtDlpSource {
    pub fn new(config: MusicConfig) -> Self {
        Self {
            config,
            global: Arc::new(Semaphore::new(4)),
        }
    }
    fn webpage(url: &Url) -> bool {
        url.host_str().is_some_and(|h| {
            ["youtube.com", "youtu.be", "soundcloud.com", "bandcamp.com"]
                .iter()
                .any(|d| h == *d || h.ends_with(&format!(".{d}")))
        })
    }
    async fn run(&self, input: &str, inspect: bool) -> Result<Value> {
        let _permit = tokio::time::timeout(Duration::from_secs(5), self.global.acquire())
            .await
            .context("Media resolver busy; try again")??;
        let mut cmd = Command::new(&self.config.yt_dlp_path);
        cmd.args([
            "--ignore-config",
            "--no-warnings",
            "--no-progress",
            "--skip-download",
            "--dump-single-json",
            "--socket-timeout",
            "10",
            "--retries",
            "1",
            "--no-check-formats",
        ]);
        if inspect {
            cmd.arg("--flat-playlist")
                .arg("--playlist-end")
                .arg((self.config.max_playlist_import + 1).to_string());
        } else {
            cmd.args([
                "--no-playlist",
                "-f",
                "bestaudio[protocol=https]/bestaudio[protocol=http]",
            ]);
        }
        cmd.arg("--")
            .arg(input)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .kill_on_drop(true);
        let mut child = cmd.spawn().map_err(|_| {
            anyhow::anyhow!("yt-dlp is unavailable; ask the operator to install it")
        })?;
        let stdout = child.stdout.take().context("Resolver stdout unavailable")?;
        let stderr = child.stderr.take().context("Resolver stderr unavailable")?;
        let work = async {
            let out = async {
                let mut bytes = vec![];
                stdout
                    .take(2 * 1024 * 1024 + 1)
                    .read_to_end(&mut bytes)
                    .await?;
                ensure!(
                    bytes.len() <= 2 * 1024 * 1024,
                    "Resolver output exceeded limit"
                );
                Ok::<_, anyhow::Error>(bytes)
            };
            let err = async {
                let mut bytes = vec![];
                stderr.take(65537).read_to_end(&mut bytes).await?;
                ensure!(bytes.len() <= 65536, "Resolver diagnostics exceeded limit");
                Ok::<_, anyhow::Error>(bytes)
            };
            let (out, diagnostics, status) = tokio::try_join!(out, err, async {
                Ok::<_, anyhow::Error>(child.wait().await?)
            })?;
            if !status.success() {
                anyhow::bail!(
                    "Media resolver failed: {}",
                    classify_resolver_diagnostics(&diagnostics)
                );
            }
            serde_json::from_slice::<Value>(&out).context("Resolver returned invalid metadata")
        };
        let result = tokio::time::timeout(
            Duration::from_secs(self.config.resolver_timeout_seconds),
            work,
        )
        .await;
        match result {
            Ok(Ok(value)) => Ok(value),
            Ok(Err(error)) => {
                let _ = child.kill().await;
                Err(error)
            }
            Err(_) => {
                let _ = child.kill().await;
                anyhow::bail!("Media resolver timed out")
            }
        }
    }
    pub fn parse_entries(value: Value, user: u64, limit: usize) -> Result<Vec<TrackRef>> {
        let entries = value
            .get("entries")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_else(|| vec![value]);
        ensure!(
            !entries.is_empty() && entries.len() <= limit,
            "Playlist is empty or exceeds the import limit; choose a smaller playlist"
        );
        entries
            .into_iter()
            .map(|v| {
                let url = v
                    .get("webpage_url")
                    .or_else(|| v.get("url"))
                    .and_then(Value::as_str)
                    .context("Source returned no canonical URL")?;
                let parsed = parse_url(url)?;
                ensure!(
                    Self::webpage(&parsed),
                    "Source returned an unsupported canonical URL"
                );
                Ok(TrackRef {
                    provider: "yt-dlp".into(),
                    canonical_url: parsed.to_string(),
                    title: v
                        .get("title")
                        .and_then(Value::as_str)
                        .unwrap_or("Untitled")
                        .chars()
                        .take(200)
                        .collect(),
                    duration_ms: v
                        .get("duration")
                        .and_then(Value::as_f64)
                        .filter(|d| d.is_finite() && *d >= 0.0)
                        .map(|d| (d * 1000.0) as u64),
                    requested_by: user.to_string(),
                })
            })
            .collect()
    }
}
#[async_trait]
impl AudioSourceProvider for YtDlpSource {
    fn id(&self) -> &'static str {
        "yt-dlp"
    }
    fn accepts(&self, input: &str) -> bool {
        parse_url(input)
            .map(|u| Self::webpage(&u))
            .unwrap_or(!input.contains("://"))
    }
    async fn inspect(&self, input: &str, user: u64) -> Result<Vec<TrackRef>> {
        ensure!(
            !input.trim().is_empty() && input.len() <= 2000,
            "Search must be 1–2000 bytes"
        );
        let input = if input.contains("://") {
            let url = parse_url(input)?;
            ensure!(Self::webpage(&url), "Unsupported media website");
            pinned_client(&url).await?;
            url.to_string()
        } else {
            format!("ytsearch1:{input}")
        };
        Self::parse_entries(
            self.run(&input, true).await?,
            user,
            self.config.max_playlist_import,
        )
    }
    async fn resolve(&self, track: &TrackRef) -> Result<PlayableTrack> {
        let canonical = parse_url(&track.canonical_url)?;
        ensure!(Self::webpage(&canonical), "Unsupported canonical media URL");
        let result = self.run(canonical.as_str(), false).await?;
        let url = result
            .get("url")
            .and_then(Value::as_str)
            .context("Resolver returned no playable audio")?;
        download(url).await
    }
    async fn health(&self) -> bool {
        tokio::time::timeout(
            Duration::from_secs(3),
            Command::new(&self.config.yt_dlp_path)
                .arg("--version")
                .stdin(Stdio::null())
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .kill_on_drop(true)
                .status(),
        )
        .await
        .is_ok_and(|r| r.is_ok_and(|s| s.success()))
    }
}
pub struct Sources {
    limit: Semaphore,
    providers: Vec<Arc<dyn AudioSourceProvider>>,
}
impl Sources {
    pub fn new(config: MusicConfig) -> Self {
        Self {
            limit: Semaphore::new(4),
            providers: vec![
                Arc::new(YtDlpSource::new(config)),
                Arc::new(DirectHttpSource),
            ],
        }
    }
    pub async fn inspect(&self, input: &str, user: u64) -> Result<Vec<TrackRef>> {
        self.providers
            .iter()
            .find(|p| p.accepts(input))
            .context("Unsupported source; local files are disabled")?
            .inspect(input, user)
            .await
    }
    pub async fn resolve(&self, track: &TrackRef) -> Result<PlayableTrack> {
        let _permit = tokio::time::timeout(Duration::from_secs(5), self.limit.acquire())
            .await
            .context("Media resolver busy")??;
        self.providers
            .iter()
            .find(|p| p.id() == track.provider)
            .context("Unknown source provider")?
            .resolve(track)
            .await
    }
    pub async fn health(&self) -> bool {
        self.providers[0].health().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rejects_private_and_special_networks() {
        for ip in [
            "127.0.0.1",
            "10.0.0.1",
            "172.16.0.1",
            "192.168.0.1",
            "169.254.169.254",
            "100.64.0.1",
            "224.0.0.1",
            "0.0.0.0",
            "::1",
            "fc00::1",
            "fe80::1",
            "::ffff:127.0.0.1",
            "2001:db8::1",
            "2002:7f00:1::",
        ] {
            assert!(!public_ip(ip.parse().unwrap()), "{ip}");
        }
        assert!(public_ip("8.8.8.8".parse().unwrap()));
        assert!(public_ip("2606:4700:4700::1111".parse().unwrap()));
    }
    #[test]
    fn validates_urls_and_bounded_metadata() {
        assert!(parse_url("file:///etc/passwd").is_err());
        assert!(parse_url("https://user:pass@example.com").is_err());
        assert!(parse_url("http://example.com:3000/audio").is_err());
        let fixture = serde_json::json!({"entries":[{"title":"Example","url":"https://www.youtube.com/watch?v=example","duration":10}]});
        assert_eq!(
            YtDlpSource::parse_entries(fixture.clone(), 1, 10).unwrap()[0].duration_ms,
            Some(10000)
        );
        assert!(YtDlpSource::parse_entries(fixture, 1, 0).is_err());
    }
}
