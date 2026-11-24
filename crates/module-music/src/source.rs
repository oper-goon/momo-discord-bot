use anyhow::{Context, Result, ensure};
use async_trait::async_trait;
use bot_core::{TrackRef, config::MusicConfig};
use futures_util::TryStreamExt;
use reqwest::{
    Client,
    header::{ACCEPT_RANGES, CONTENT_LENGTH, CONTENT_RANGE, RANGE},
};
use serde_json::Value;
use songbird::input::{
    AsyncAdapterStream, AsyncMediaSource, AudioStream, AudioStreamError, Compose, Input,
};
use std::{
    io::{Error as IoError, ErrorKind as IoErrorKind, Result as IoResult, SeekFrom},
    net::{IpAddr, SocketAddr},
    pin::Pin,
    process::Stdio,
    sync::Arc,
    task::{Context as TaskContext, Poll},
    time::Duration,
};
use tokio::{
    io::{AsyncRead, AsyncReadExt, AsyncSeek, ReadBuf},
    process::Command,
    sync::Semaphore,
};
use tokio_util::io::StreamReader;
use url::Url;

const MAX_AUDIO_BYTES: u64 = 64 * 1024 * 1024;

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
    pub provider: Option<String>,
}
#[async_trait]
pub trait AudioSourceProvider: Send + Sync {
    fn id(&self) -> &'static str;
    fn accepts(&self, input: &str) -> bool;
    async fn inspect(&self, input: &str, user: u64) -> Result<Vec<TrackRef>>;
    async fn resolve(&self, track: &TrackRef) -> Result<PlayableTrack>;
    async fn health(&self) -> bool;
    /// Normalized individual-track search; explicit website/playlist inspection stays separate.
    async fn search(&self, query: &crate::matching::Metadata, user: u64) -> Result<Vec<TrackRef>> {
        self.inspect(&query.query(), user).await
    }
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
        .read_timeout(Duration::from_secs(30))
        .resolve_to_addrs(host, &addresses)
        .build()?)
}

#[derive(Clone, Debug)]
struct BoundedHttpRequest {
    client: Client,
    request: String,
    max_bytes: u64,
}

impl BoundedHttpRequest {
    fn new(client: Client, request: String, max_bytes: u64) -> Self {
        Self {
            client,
            request,
            max_bytes,
        }
    }

    async fn create_stream(&self, offset: u64) -> Result<BoundedHttpStream, AudioStreamError> {
        if offset >= self.max_bytes {
            return Err(stream_error("Audio exceeds the 64 MiB limit"));
        }
        let response = self
            .client
            .get(&self.request)
            .header(RANGE, format!("bytes={offset}-{}", self.max_bytes - 1))
            .send()
            .await
            .map_err(|_| stream_error("Media stream request failed"))?;
        if !response.status().is_success() {
            return Err(stream_error(format!(
                "Media source rejected the stream with status {}",
                response.status()
            )));
        }
        if offset > 0 && response.status() != reqwest::StatusCode::PARTIAL_CONTENT {
            return Err(stream_error(
                "Media source cannot resume an interrupted stream",
            ));
        }
        let headers = response.headers();
        let total_bytes = headers
            .get(CONTENT_RANGE)
            .and_then(|value| value.to_str().ok())
            .and_then(|value| value.rsplit('/').next())
            .filter(|value| *value != "*")
            .and_then(|value| value.parse::<u64>().ok())
            .or_else(|| {
                headers
                    .get(CONTENT_LENGTH)
                    .and_then(|value| value.to_str().ok())
                    .and_then(|value| value.parse::<u64>().ok())
                    .map(|length| {
                        if response.status() == reqwest::StatusCode::PARTIAL_CONTENT {
                            offset.saturating_add(length)
                        } else {
                            length
                        }
                    })
            });
        if total_bytes.is_some_and(|length| length > self.max_bytes) {
            return Err(stream_error("Audio exceeds the 64 MiB limit"));
        }
        let resumable = response.status() == reqwest::StatusCode::PARTIAL_CONTENT
            || headers
                .get(ACCEPT_RANGES)
                .and_then(|value| value.to_str().ok())
                .is_some_and(|value| value.eq_ignore_ascii_case("bytes"));
        let stream = Box::new(StreamReader::new(
            response.bytes_stream().map_err(IoError::other),
        ));
        Ok(BoundedHttpStream {
            stream,
            position: offset,
            total_bytes,
            max_bytes: self.max_bytes,
            resume: resumable.then(|| self.clone()),
        })
    }
}

fn stream_error(message: impl Into<String>) -> AudioStreamError {
    AudioStreamError::Fail(Box::new(IoError::other(message.into())))
}

struct BoundedHttpStream {
    stream: Box<dyn AsyncRead + Send + Sync + Unpin>,
    position: u64,
    total_bytes: Option<u64>,
    max_bytes: u64,
    resume: Option<BoundedHttpRequest>,
}

impl AsyncRead for BoundedHttpStream {
    fn poll_read(
        mut self: Pin<&mut Self>,
        context: &mut TaskContext<'_>,
        buffer: &mut ReadBuf<'_>,
    ) -> Poll<IoResult<()>> {
        let before = buffer.filled().len();
        match Pin::new(&mut *self.stream).poll_read(context, buffer) {
            Poll::Ready(Ok(())) => {
                let read = (buffer.filled().len() - before) as u64;
                self.position = self.position.saturating_add(read);
                if self.position > self.max_bytes {
                    Poll::Ready(Err(IoError::other("Audio exceeds the 64 MiB limit")))
                } else {
                    Poll::Ready(Ok(()))
                }
            }
            result => result,
        }
    }
}

impl AsyncSeek for BoundedHttpStream {
    fn start_seek(self: Pin<&mut Self>, _: SeekFrom) -> IoResult<()> {
        Err(IoErrorKind::Unsupported.into())
    }

    fn poll_complete(self: Pin<&mut Self>, _: &mut TaskContext<'_>) -> Poll<IoResult<u64>> {
        Poll::Ready(Err(IoErrorKind::Unsupported.into()))
    }
}

#[async_trait]
impl AsyncMediaSource for BoundedHttpStream {
    fn is_seekable(&self) -> bool {
        false
    }

    async fn byte_len(&self) -> Option<u64> {
        self.total_bytes
    }

    async fn try_resume(
        &mut self,
        offset: u64,
    ) -> Result<Box<dyn AsyncMediaSource>, AudioStreamError> {
        match &self.resume {
            Some(request) => request
                .create_stream(offset)
                .await
                .map(|stream| Box::new(stream) as Box<dyn AsyncMediaSource>),
            None => Err(AudioStreamError::Unsupported),
        }
    }
}

#[async_trait]
impl Compose for BoundedHttpRequest {
    fn create(
        &mut self,
    ) -> Result<AudioStream<Box<dyn songbird::input::core::io::MediaSource>>, AudioStreamError>
    {
        Err(AudioStreamError::Unsupported)
    }

    async fn create_async(
        &mut self,
    ) -> Result<AudioStream<Box<dyn songbird::input::core::io::MediaSource>>, AudioStreamError>
    {
        self.create_stream(0).await.map(|stream| AudioStream {
            input: Box::new(AsyncAdapterStream::new(Box::new(stream), 64 * 1024))
                as Box<dyn songbird::input::core::io::MediaSource>,
        })
    }

    fn should_create_async(&self) -> bool {
        true
    }
}

async fn stream(url: &str) -> Result<PlayableTrack> {
    let url = parse_url(url)?;
    let client = pinned_client(&url).await?;
    let mut input = Input::Lazy(Box::new(BoundedHttpRequest::new(
        client,
        url.to_string(),
        MAX_AUDIO_BYTES,
    )));
    input = input
        .make_playable_async(
            songbird::input::codecs::get_codec_registry(),
            songbird::input::codecs::get_probe(),
        )
        .await
        .map_err(|_| anyhow::anyhow!("Media stream could not start"))?;
    Ok(PlayableTrack { input, provider: None })
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
        stream(&track.canonical_url).await
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
        stream(url).await
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
