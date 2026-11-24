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
    if text.contains("429") || text.contains("too many requests") {
        "source rate limited"
    } else if text.contains("403") || text.contains("ip block") {
        "source IP blocked"
    } else if text.contains("sign in") || text.contains("captcha") || text.contains("bot") {
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
#[cfg(test)]
pub(crate) fn mock_playable() -> PlayableTrack {
    PlayableTrack {
        input: Input::Lazy(Box::new(BoundedHttpRequest::new(
            Client::new(),
            "https://example.com/audio".into(),
            MAX_AUDIO_BYTES,
        ))),
        provider: None,
    }
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
pub(crate) async fn pinned_client(url: &Url) -> Result<reqwest::Client> {
    client_with_headers(url, reqwest::header::HeaderMap::new()).await
}
async fn client_with_headers(
    url: &Url,
    headers: reqwest::header::HeaderMap,
) -> Result<reqwest::Client> {
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
        .default_headers(headers)
        .no_proxy()
        .redirect(reqwest::redirect::Policy::none())
        .connect_timeout(Duration::from_secs(10))
        .read_timeout(Duration::from_secs(30))
        .resolve_to_addrs(host, &addresses)
        .build()?)
}

#[derive(Clone)]
struct BoundedHttpRequest {
    client: Client,
    request: String,
    max_bytes: u64,
}
impl std::fmt::Debug for BoundedHttpRequest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("BoundedHttpRequest([private media])")
    }
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
    stream_authenticated(url, None).await
}

pub(crate) async fn stream_authenticated(
    url: &str,
    credentials: Option<(&str, &str)>,
) -> Result<PlayableTrack> {
    let url = parse_url(url)?;
    let mut client = pinned_client(&url).await?;
    if let Some((username, password)) = credentials {
        let request = client
            .get(url.as_str())
            .basic_auth(username, Some(password))
            .build()
            .map_err(|_| anyhow::anyhow!("Invalid media authentication"))?;
        let mut headers = request.headers().clone();
        if let Some(auth) = headers.get_mut(reqwest::header::AUTHORIZATION) {
            auth.set_sensitive(true);
        }
        // Reuse the checked, pinned client; credentials stay on this single origin.
        client = client_with_headers(&url, headers).await?;
    }
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
    Ok(PlayableTrack {
        input,
        provider: None,
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
    providers: Vec<ProviderSlot>,
    config: MusicConfig,
}
struct ProviderSlot {
    provider: Arc<dyn AudioSourceProvider>,
    state: tokio::sync::Mutex<ProviderState>,
    search_gate: tokio::sync::Mutex<()>,
}
#[derive(Default)]
struct ProviderState {
    open_until: Option<tokio::time::Instant>,
    last_error: Option<ProviderError>,
    search: Option<(String, tokio::time::Instant, Vec<TrackRef>)>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProviderError {
    Authentication,
    RateLimited,
    Blocked,
    Timeout,
    Unavailable,
    Network,
    InvalidAudio,
    NoMatch,
    UnsafeResponse,
    Dns,
}
impl ProviderError {
    pub(crate) fn classify(error: &anyhow::Error) -> Self {
        let text = error.to_string().to_ascii_lowercase();
        if text.contains("authentication") || text.contains("credentials") {
            Self::Authentication
        } else if text.contains("rate limit") || text.contains("429") {
            Self::RateLimited
        } else if text.contains("challenge") || text.contains("blocked") {
            Self::Blocked
        } else if text.contains("timed out") || text.contains("timeout") {
            Self::Timeout
        } else if text.contains("media stream could not start") {
            Self::InvalidAudio
        } else if text.contains("request failed") || text.contains("network") {
            Self::Network
        } else if text.contains("no accurate") {
            Self::NoMatch
        } else if text.contains("unsafe") || text.contains("invalid easynews") {
            Self::UnsafeResponse
        } else if text.contains("dns") {
            Self::Dns
        } else {
            Self::Unavailable
        }
    }
}
impl ProviderSlot {
    fn new(provider: Arc<dyn AudioSourceProvider>) -> Self {
        Self {
            provider,
            state: tokio::sync::Mutex::new(ProviderState::default()),
            search_gate: tokio::sync::Mutex::new(()),
        }
    }
}
impl Sources {
    pub fn new(config: MusicConfig) -> Self {
        let mut providers: Vec<Arc<dyn AudioSourceProvider>> = Vec::new();
        for id in &config.provider_priority {
            match id.as_str() {
                "youtube" => providers.push(Arc::new(YtDlpSource::new(config.clone()))),
                "easynews" if config.easynews_enabled => providers.push(Arc::new(
                    crate::providers::easynews::Easynews::new(config.match_min_confidence),
                )),
                "torbox" if config.torbox_enabled && config.torbox_cached_only => {
                    providers.push(Arc::new(crate::providers::torbox::Torbox::new(
                        config.match_min_confidence,
                    )))
                }
                _ => (),
            }
        }
        providers.push(Arc::new(DirectHttpSource));
        Self {
            limit: Semaphore::new(4),
            providers: providers.into_iter().map(ProviderSlot::new).collect(),
            config,
        }
    }
    async fn attempt<T>(
        &self,
        slot: &ProviderSlot,
        operation: &'static str,
        work: impl std::future::Future<Output = Result<T>>,
    ) -> Result<T> {
        let started = tokio::time::Instant::now();
        let provider = slot.provider.id();
        // Serializing each provider also gives an open circuit exactly one recovery probe.
        let mut state = match tokio::time::timeout(Duration::from_secs(5), slot.state.lock()).await
        {
            Ok(state) => state,
            Err(_) => {
                tracing::info!(
                    provider,
                    operation,
                    reason = "busy",
                    elapsed_ms = started.elapsed().as_millis() as u64,
                    "Music provider skipped"
                );
                anyhow::bail!("Provider busy");
            }
        };
        if let Some(until) = state.open_until
            && until > tokio::time::Instant::now()
        {
            tracing::info!(
                provider,
                operation,
                reason = "cooling_down",
                remaining_ms = until
                    .saturating_duration_since(tokio::time::Instant::now())
                    .as_millis() as u64,
                "Music provider skipped"
            );
            anyhow::bail!("Provider cooling down");
        }
        let recovery = state.open_until.is_some();
        if recovery {
            tracing::info!(provider, operation, "Music provider recovery probe");
        }
        tracing::info!(provider, operation, "Music provider attempt started");
        let result = tokio::time::timeout(
            Duration::from_secs(self.config.provider_timeout_seconds),
            work,
        )
        .await
        .unwrap_or_else(|_| Err(anyhow::anyhow!("Provider timed out")));
        match &result {
            Ok(_) => {
                state.open_until = None;
                state.last_error = None;
                tracing::info!(
                    provider,
                    operation,
                    outcome = "success",
                    elapsed_ms = started.elapsed().as_millis() as u64,
                    "Music provider attempt finished"
                );
                if recovery {
                    tracing::info!(provider, "Music provider circuit closed");
                }
            }
            Err(error) => {
                let reason = ProviderError::classify(error);
                state.last_error = Some(reason);
                let retry_after = error
                    .downcast_ref::<crate::providers::RateLimited>()
                    .map_or(0, |limit| limit.0);
                let cooldown_seconds = self.config.provider_cooldown_seconds.max(retry_after);
                state.open_until =
                    Some(tokio::time::Instant::now() + Duration::from_secs(cooldown_seconds));
                state.search = None;
                tracing::info!(
                    provider,
                    operation,
                    outcome = "failure",
                    ?reason,
                    elapsed_ms = started.elapsed().as_millis() as u64,
                    "Music provider attempt finished"
                );
                tracing::info!(
                    provider,
                    ?reason,
                    cooldown_seconds,
                    "Music provider circuit opened"
                );
            }
        }
        result
    }
    async fn search(
        &self,
        slot: &ProviderSlot,
        wanted: &crate::matching::Metadata,
        user: u64,
    ) -> Result<Vec<TrackRef>> {
        let _gate = tokio::time::timeout(Duration::from_secs(5), slot.search_gate.lock())
            .await
            .context("Provider search busy")?;
        let key = format!("{}:{:?}", wanted.query(), wanted.duration_ms);
        // Bounded one-entry metadata cache per provider; no audio cache exists.
        let work = async { slot.provider.search(wanted, user).await };
        {
            let state = tokio::time::timeout(Duration::from_secs(5), slot.state.lock())
                .await
                .context("Provider busy")?;
            if !state
                .open_until
                .is_some_and(|until| until > tokio::time::Instant::now())
                && let Some((cached_key, at, tracks)) = &state.search
                && cached_key == &key
                && at.elapsed() < Duration::from_secs(30)
            {
                let mut tracks = tracks.clone();
                for track in &mut tracks {
                    track.requested_by = user.to_string();
                }
                tracing::info!(
                    provider = slot.provider.id(),
                    operation = "search",
                    outcome = "metadata_cache_hit",
                    result_count = tracks.len(),
                    "Music provider search finished"
                );
                return Ok(tracks);
            }
        }
        let mut tracks = self.attempt(slot, "search", work).await?;
        tracks.retain(|track| {
            crate::matching::confidence(
                wanted,
                &crate::matching::Metadata::from_title(&track.title, track.duration_ms),
            ) >= self.config.match_min_confidence
        });
        tracing::info!(
            provider = slot.provider.id(),
            operation = "search",
            outcome = if tracks.is_empty() {
                "no_match"
            } else {
                "matched"
            },
            result_count = tracks.len(),
            "Music provider search finished"
        );
        slot.state.lock().await.search = Some((key, tokio::time::Instant::now(), tracks.clone()));
        Ok(tracks)
    }
    #[tracing::instrument(name = "music_search", skip_all, fields(search_id = %uuid::Uuid::new_v4()))]
    pub async fn inspect(&self, input: &str, user: u64) -> Result<Vec<TrackRef>> {
        ensure!(
            !input.trim().is_empty() && input.len() <= 2000,
            "Search must be 1–2000 bytes"
        );
        let _permit = tokio::time::timeout(Duration::from_secs(5), self.limit.acquire())
            .await
            .context("Music providers busy")??;
        if input.contains("://") {
            let slot = self
                .providers
                .iter()
                .find(|p| p.provider.accepts(input))
                .context("Unsupported source; local files are disabled")?;
            return self
                .attempt(slot, "inspect", slot.provider.inspect(input, user))
                .await;
        }
        let wanted = crate::matching::Metadata::from_title(input, None);
        let mut previous = None;
        for slot in self.providers.iter().filter(|p| p.provider.id() != "http") {
            if let Some(from_provider) = previous {
                tracing::info!(
                    from_provider,
                    to_provider = slot.provider.id(),
                    operation = "search",
                    "Music provider fallback"
                );
            }
            if let Ok(tracks) = self.search(slot, &wanted, user).await
                && let Some(track) = tracks.first()
            {
                tracing::info!(
                    provider = slot.provider.id(),
                    operation = "search",
                    "Music provider selected"
                );
                return Ok(vec![track.clone()]);
            }
            previous = Some(slot.provider.id());
        }
        tracing::info!(
            operation = "search",
            reason = "all_providers_unavailable_or_no_match",
            "Music providers exhausted"
        );
        anyhow::bail!("No enabled music provider found a sufficiently accurate available track")
    }
    pub async fn resolve(&self, track: &TrackRef) -> Result<PlayableTrack> {
        self.resolve_excluding(track, &[]).await
    }
    #[tracing::instrument(name = "music_resolution", skip_all, fields(resolution_id = %uuid::Uuid::new_v4()))]
    pub async fn resolve_excluding(
        &self,
        track: &TrackRef,
        excluded: &[String],
    ) -> Result<PlayableTrack> {
        let _permit = tokio::time::timeout(Duration::from_secs(5), self.limit.acquire())
            .await
            .context("Media resolver busy")??;
        let wanted = crate::matching::Metadata::from_title(&track.title, track.duration_ms);
        let user = track.requested_by.parse().unwrap_or(0);
        let mut ordered: Vec<_> = self
            .providers
            .iter()
            .filter(|p| p.provider.id() == track.provider)
            .collect();
        if track.provider != "http" {
            ordered.extend(
                self.providers
                    .iter()
                    .filter(|p| p.provider.id() != track.provider && p.provider.id() != "http"),
            );
        }
        let mut previous = None;
        for slot in ordered {
            let id = slot.provider.id();
            if excluded.iter().any(|p| p == id) {
                tracing::info!(
                    provider = id,
                    operation = "resolve",
                    reason = "initialization_already_failed",
                    "Music provider skipped"
                );
                previous = Some(id);
                continue;
            }
            if let Some(from_provider) = previous {
                tracing::info!(
                    from_provider,
                    to_provider = id,
                    operation = "resolve",
                    "Music provider fallback"
                );
            }
            let work = async {
                let candidate;
                let target = if id == track.provider {
                    track
                } else {
                    let results = slot.provider.search(&wanted, user).await?;
                    candidate = results
                        .into_iter()
                        .find(|result| {
                            crate::matching::confidence(
                                &wanted,
                                &crate::matching::Metadata::from_title(
                                    &result.title,
                                    result.duration_ms,
                                ),
                            ) >= self.config.match_min_confidence
                        })
                        .context("No accurate fallback result")?;
                    &candidate
                };
                slot.provider.resolve(target).await
            };
            if let Ok(mut playable) = self.attempt(slot, "resolve", work).await {
                playable.provider = Some(id.into());
                tracing::info!(provider = id, "Music audio resolved");
                return Ok(playable);
            }
            previous = Some(id);
        }
        tracing::info!(
            operation = "resolve",
            reason = "all_providers_unavailable",
            "Music providers exhausted"
        );
        anyhow::bail!("All enabled music providers are unavailable or cooling down")
    }
    pub async fn health(&self) -> bool {
        for slot in self.providers.iter().filter(|p| p.provider.id() != "http") {
            if slot.provider.health().await
                && !slot
                    .state
                    .lock()
                    .await
                    .open_until
                    .is_some_and(|until| until > tokio::time::Instant::now())
            {
                return true;
            }
        }
        false
    }
    pub async fn status(&self) -> String {
        let mut lines = Vec::new();
        for slot in &self.providers {
            let configured = slot.provider.health().await;
            let Ok(state) = slot.state.try_lock() else {
                lines.push(format!("{}: request in progress", slot.provider.id()));
                continue;
            };
            let status = if !configured {
                "not configured / unavailable"
            } else if state
                .open_until
                .is_some_and(|until| until > tokio::time::Instant::now())
            {
                "cooling down"
            } else {
                "ready"
            };
            lines.push(format!(
                "{}: {status}{}",
                slot.provider.id(),
                state
                    .last_error
                    .map(|e| format!(" ({e:?})"))
                    .unwrap_or_default()
            ));
        }
        lines.join("\n")
    }
    pub async fn initialization_failed(&self, id: &str) {
        if let Some(slot) = self.providers.iter().find(|slot| slot.provider.id() == id) {
            // Never stall the guild actor behind another guild's network request.
            let Ok(mut state) = slot.state.try_lock() else {
                return;
            };
            state.last_error = Some(ProviderError::Unavailable);
            state.open_until = Some(
                tokio::time::Instant::now()
                    + Duration::from_secs(self.config.provider_cooldown_seconds),
            );
            state.search = None;
            tracing::info!(
                provider = slot.provider.id(),
                reason = "playback_initialization_failed",
                cooldown_seconds = self.config.provider_cooldown_seconds,
                "Music provider circuit opened"
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tracing::instrument::WithSubscriber;
    #[tokio::test(start_paused = true)]
    async fn logs_safe_provider_outcomes_fallback_and_recovery_without_private_data() {
        let capture = crate::log_capture::Capture::default();
        async {
            let (sources, _, mut track) = fixture(
                [
                    Some("source access challenge SECRET_CANARY https://private.invalid/token"),
                    None,
                    None,
                ],
                false,
            );
            track.canonical_url = "https://private.invalid/SECRET_CANARY".into();
            track.requested_by = "PRIVATE_REQUESTER_CANARY".into();
            assert_eq!(
                sources.resolve(&track).await.unwrap().provider.as_deref(),
                Some("easynews")
            );
            sources.resolve(&track).await.unwrap(); // Shared open circuit is skipped.
            tokio::time::advance(Duration::from_secs(121)).await;
            sources
                .attempt(&sources.providers[0], "resolve", async { Ok(()) })
                .await
                .unwrap();
        }
        .with_subscriber(capture.clone())
        .await;
        let logs = capture.snapshot();
        for expected in [
            "INFO",
            "yt-dlp",
            "easynews",
            "Blocked",
            "elapsed_ms",
            "resolution_id",
            "Music provider fallback",
            "cooling_down",
            "Music provider circuit opened",
            "Music provider recovery probe",
            "Music provider circuit closed",
            "Music audio resolved",
        ] {
            assert!(
                logs.contains(expected),
                "Missing log field/event: {expected}"
            );
        }
        for private in [
            "SECRET_CANARY",
            "PRIVATE_REQUESTER_CANARY",
            "private.invalid",
            "Artist - Song",
        ] {
            assert!(!logs.contains(private), "Private field leaked to logs");
        }
    }
    #[tokio::test]
    async fn bounded_stream_probes_audio_and_rejects_expired_or_corrupt_data() {
        let mut wav = Vec::new();
        wav.extend_from_slice(b"RIFF");
        wav.extend_from_slice(&68_u32.to_le_bytes());
        wav.extend_from_slice(b"WAVEfmt ");
        wav.extend_from_slice(&16_u32.to_le_bytes());
        wav.extend_from_slice(&1_u16.to_le_bytes()); // PCM
        wav.extend_from_slice(&1_u16.to_le_bytes()); // mono
        wav.extend_from_slice(&8000_u32.to_le_bytes());
        wav.extend_from_slice(&8000_u32.to_le_bytes());
        wav.extend_from_slice(&1_u16.to_le_bytes());
        wav.extend_from_slice(&8_u16.to_le_bytes());
        wav.extend_from_slice(b"data");
        wav.extend_from_slice(&32_u32.to_le_bytes());
        wav.extend_from_slice(&[0; 32]);
        for (status, bytes, expected) in [
            ("200 OK", String::from_utf8(wav).unwrap(), true),
            ("200 OK", "corrupt audio".into(), false),
            ("410 Gone", "expired private URL".into(), false),
        ] {
            let (url, server) = crate::providers::mock_api(vec![(status, bytes)]).await;
            let input = Input::Lazy(Box::new(BoundedHttpRequest::new(
                Client::new(),
                url,
                MAX_AUDIO_BYTES,
            )));
            let result = input
                .make_playable_async(
                    songbird::input::codecs::get_codec_registry(),
                    songbird::input::codecs::get_probe(),
                )
                .await;
            assert_eq!(result.is_ok(), expected);
            server.await.unwrap();
        }
    }
    use std::sync::atomic::{AtomicUsize, Ordering};
    struct MockProvider {
        id: &'static str,
        failure: Option<&'static str>,
        calls: Arc<AtomicUsize>,
        slow: bool,
    }
    #[async_trait]
    impl AudioSourceProvider for MockProvider {
        fn id(&self) -> &'static str {
            self.id
        }
        fn accepts(&self, _: &str) -> bool {
            true
        }
        async fn inspect(&self, _: &str, user: u64) -> Result<Vec<TrackRef>> {
            Ok(vec![TrackRef {
                provider: self.id.into(),
                canonical_url: "mock:track".into(),
                title: "Artist - Song".into(),
                duration_ms: Some(200_000),
                requested_by: user.to_string(),
            }])
        }
        async fn resolve(&self, _: &TrackRef) -> Result<PlayableTrack> {
            self.calls.fetch_add(1, Ordering::SeqCst);
            if self.slow {
                tokio::time::sleep(Duration::from_secs(300)).await;
            }
            if let Some(error) = self.failure {
                anyhow::bail!(error);
            }
            Ok(mock_playable())
        }
        async fn health(&self) -> bool {
            true
        }
    }
    fn fixture(
        failures: [Option<&'static str>; 3],
        slow: bool,
    ) -> (Sources, Vec<Arc<AtomicUsize>>, TrackRef) {
        let calls: Vec<_> = (0..3).map(|_| Arc::new(AtomicUsize::new(0))).collect();
        let providers = ["yt-dlp", "easynews", "torbox"]
            .into_iter()
            .enumerate()
            .map(|(i, id)| {
                ProviderSlot::new(Arc::new(MockProvider {
                    id,
                    failure: failures[i],
                    calls: calls[i].clone(),
                    slow: slow && i == 0,
                }))
            })
            .collect();
        (
            Sources {
                limit: Semaphore::new(4),
                providers,
                config: MusicConfig::default(),
            },
            calls,
            TrackRef {
                provider: "yt-dlp".into(),
                canonical_url: "https://www.youtube.com/watch?v=test".into(),
                title: "Artist - Song".into(),
                duration_ms: Some(200_000),
                requested_by: "1".into(),
            },
        )
    }
    #[tokio::test]
    async fn youtube_success_and_fallback_order() {
        for (failures, expected, counts) in [
            ([None, None, None], "yt-dlp", [1, 0, 0]),
            (
                [Some("source access challenge"), None, None],
                "easynews",
                [1, 1, 0],
            ),
            (
                [
                    Some("source access challenge"),
                    Some("Provider authentication failed"),
                    None,
                ],
                "torbox",
                [1, 1, 1],
            ),
        ] {
            let (sources, calls, track) = fixture(failures, false);
            assert_eq!(
                sources.resolve(&track).await.unwrap().provider.as_deref(),
                Some(expected)
            );
            for (calls, count) in calls.iter().zip(counts) {
                assert_eq!(calls.load(Ordering::SeqCst), count);
            }
        }
    }
    #[tokio::test]
    async fn persistent_block_skipped_for_next_song_and_exhaustion_is_finite() {
        let (sources, calls, track) = fixture([Some("source access challenge"), None, None], false);
        sources.resolve(&track).await.unwrap();
        sources.resolve(&track).await.unwrap();
        assert_eq!(calls[0].load(Ordering::SeqCst), 1);
        assert!(sources.status().await.contains("Blocked"));
        let (sources, calls, track) = fixture(
            [Some("unavailable"), Some("unavailable"), Some("not cached")],
            false,
        );
        assert!(sources.resolve(&track).await.is_err());
        assert!(calls.iter().all(|count| count.load(Ordering::SeqCst) == 1));
    }
    #[tokio::test(start_paused = true)]
    async fn timeout_falls_back_and_circuit_recovers_after_cooldown() {
        let (sources, calls, track) = fixture([None, None, None], true);
        assert_eq!(
            sources.resolve(&track).await.unwrap().provider.as_deref(),
            Some("easynews")
        );
        assert!(sources.status().await.contains("Timeout"));
        tokio::time::advance(Duration::from_secs(121)).await;
        sources.resolve(&track).await.unwrap();
        assert_eq!(calls[0].load(Ordering::SeqCst), 2);
    }
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
