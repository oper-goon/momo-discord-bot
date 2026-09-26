use anyhow::{Context, Result, ensure};
use bot_core::config::DownloadConfig;
use futures_util::TryStreamExt;
use reqwest::header::CONTENT_LENGTH;
use std::{
    net::{IpAddr, SocketAddr},
    path::{Path, PathBuf},
    process::Stdio,
    sync::Arc,
    time::Duration,
};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    process::Command,
    sync::Semaphore,
};
use tokio_util::sync::CancellationToken;
use url::Url;

#[derive(Clone, Debug)]
pub enum MediaSource {
    Web(Url),
    Attachment {
        url: Url,
        filename: String,
        declared_size: u64,
    },
}

pub struct DownloadArtifact {
    pub path: PathBuf,
    pub filename: String,
    _job: TempJob,
    _permit: tokio::sync::OwnedSemaphorePermit,
}

#[derive(Clone, Copy)]
pub enum MediaOperation {
    Audio,
    Compress,
    Gif,
    Thumbnail,
}

struct TempJob {
    path: PathBuf,
}

impl TempJob {
    fn create() -> Result<Self> {
        let path = PathBuf::from("data")
            .join("downloads")
            .join(uuid::Uuid::new_v4().simple().to_string());
        std::fs::create_dir_all(&path).context("Could not create download workspace")?;
        Ok(Self { path })
    }
}

impl Drop for TempJob {
    fn drop(&mut self) {
        if let Err(error) = std::fs::remove_dir_all(&self.path)
            && error.kind() != std::io::ErrorKind::NotFound
        {
            tracing::warn!("Could not remove download workspace");
        }
    }
}

pub struct DownloadService {
    config: DownloadConfig,
    jobs: Arc<Semaphore>,
    cancel: CancellationToken,
}

impl DownloadService {
    pub fn new(config: &bot_core::AppConfig) -> Self {
        Self {
            config: config.download.clone(),
            jobs: Arc::new(Semaphore::new(1)),
            cancel: CancellationToken::new(),
        }
    }

    pub fn config(&self) -> &DownloadConfig {
        &self.config
    }

    pub fn cancel(&self) {
        self.cancel.cancel();
    }

    pub fn accepts(&self, url: &Url) -> bool {
        url.scheme() == "https"
            && url.username().is_empty()
            && url.password().is_none()
            && url.port().is_none_or(|port| port == 443)
            && url.host_str().is_some_and(|host| {
                let host = host.to_ascii_lowercase();
                self.config
                    .allowed_domains
                    .iter()
                    .any(|domain| host == *domain || host.ends_with(&format!(".{domain}")))
            })
    }

    pub async fn download(&self, source: MediaSource, limit: u64) -> Result<DownloadArtifact> {
        let limit = limit.min(self.config.absolute_max_bytes);
        ensure!(limit >= 1024 * 1024, "Discord's upload limit is too small");
        let permit = self
            .jobs
            .clone()
            .try_acquire_owned()
            .map_err(|_| anyhow::anyhow!("The downloader is busy; try again shortly"))?;
        let job = TempJob::create()?;
        let result = tokio::select! {
            _ = self.cancel.cancelled() => Err(anyhow::anyhow!("The bot is shutting down")),
            result = tokio::time::timeout(
                Duration::from_secs(self.config.timeout_seconds),
                self.download_inner(&source, limit, &job.path),
            ) => result.context("Media download timed out")?,
        }?;
        let size = tokio::fs::metadata(&result).await?.len();
        ensure!(size > 0, "The downloaded media was empty");
        ensure!(
            size <= limit,
            "No available video fits Discord's upload limit"
        );
        let extension = result
            .extension()
            .and_then(|value| value.to_str())
            .filter(|value| value.len() <= 10)
            .unwrap_or("mp4")
            .to_owned();
        Ok(DownloadArtifact {
            path: result,
            filename: format!("download.{extension}"),
            _job: job,
            _permit: permit,
        })
    }

    pub async fn transform(
        &self,
        source: MediaSource,
        output_limit: u64,
        operation: MediaOperation,
    ) -> Result<DownloadArtifact> {
        let mut artifact = self
            .download(source, self.config.absolute_max_bytes)
            .await?;
        let (filename, arguments): (&str, &[&str]) = match operation {
            MediaOperation::Audio => ("audio.mp3", &["-vn", "-c:a", "libmp3lame", "-b:a", "192k"]),
            MediaOperation::Compress => (
                "compressed.mp4",
                &[
                    "-c:v",
                    "libx264",
                    "-preset",
                    "veryfast",
                    "-crf",
                    "28",
                    "-c:a",
                    "aac",
                    "-b:a",
                    "96k",
                    "-movflags",
                    "+faststart",
                ],
            ),
            MediaOperation::Gif => (
                "clip.gif",
                &[
                    "-t",
                    "10",
                    "-vf",
                    "fps=10,scale=640:-2:flags=lanczos",
                    "-loop",
                    "0",
                ],
            ),
            MediaOperation::Thumbnail => (
                "thumbnail.jpg",
                &[
                    "-ss",
                    "1",
                    "-frames:v",
                    "1",
                    "-vf",
                    "scale=1280:-2:force_original_aspect_ratio=decrease",
                    "-q:v",
                    "2",
                ],
            ),
        };
        let output = artifact
            .path
            .parent()
            .context("Media workspace unavailable")?
            .join(filename);
        let mut command = Command::new("ffmpeg");
        command
            .args(["-nostdin", "-hide_banner", "-loglevel", "error", "-y", "-i"])
            .arg(&artifact.path)
            .args(arguments)
            .arg(&output);
        self.run_media_command(command, "Media conversion").await?;
        let size = tokio::fs::metadata(&output).await?.len();
        ensure!(size > 0, "Media conversion produced an empty file");
        ensure!(
            size <= output_limit.min(self.config.absolute_max_bytes),
            "Converted media does not fit Discord's upload limit"
        );
        artifact.path = output;
        artifact.filename = filename.to_owned();
        Ok(artifact)
    }

    pub async fn inspect(&self, source: MediaSource) -> Result<String> {
        let artifact = self
            .download(source, self.config.absolute_max_bytes)
            .await?;
        let mut command = Command::new("ffprobe");
        command
            .args([
                "-v",
                "error",
                "-show_entries",
                "format=format_name,duration,size:stream=codec_type,codec_name,width,height,channels,sample_rate",
                "-of",
                "json",
            ])
            .arg(&artifact.path)
            .stdin(Stdio::null())
            .stderr(Stdio::null())
            .kill_on_drop(true);
        let output = tokio::time::timeout(
            Duration::from_secs(self.config.timeout_seconds),
            command.output(),
        )
        .await
        .context("Media inspection timed out")??;
        ensure!(output.status.success(), "Media inspection failed");
        ensure!(
            output.stdout.len() <= 64 * 1024,
            "Media information was too large"
        );
        describe_probe(&serde_json::from_slice(&output.stdout)?)
    }

    async fn run_media_command(&self, mut command: Command, action: &str) -> Result<()> {
        command
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .kill_on_drop(true);
        let mut child = command
            .spawn()
            .map_err(|_| anyhow::anyhow!("FFmpeg is unavailable on this host"))?;
        let stderr = child
            .stderr
            .take()
            .context("FFmpeg diagnostics unavailable")?;
        let diagnostics = async {
            let mut bytes = Vec::new();
            stderr.take(65_537).read_to_end(&mut bytes).await?;
            ensure!(bytes.len() <= 65_536, "FFmpeg diagnostics exceeded limit");
            Ok::<_, anyhow::Error>(bytes)
        };
        let result =
            tokio::time::timeout(Duration::from_secs(self.config.timeout_seconds), async {
                let (diagnostics, status) = tokio::try_join!(diagnostics, async {
                    Ok::<_, anyhow::Error>(child.wait().await?)
                })?;
                Ok::<_, anyhow::Error>((diagnostics, status))
            })
            .await
            .with_context(|| format!("{action} timed out"))??;
        if !result.1.success() {
            tracing::warn!(category = "ffmpeg", "Media conversion failed");
            anyhow::bail!("{action} failed for this media");
        }
        Ok(())
    }

    async fn download_inner(
        &self,
        source: &MediaSource,
        limit: u64,
        path: &Path,
    ) -> Result<PathBuf> {
        match source {
            MediaSource::Web(url) => self.download_web(url, limit, path).await,
            MediaSource::Attachment {
                url,
                filename,
                declared_size,
            } => {
                ensure!(
                    *declared_size <= limit,
                    "The attachment exceeds Discord's upload limit"
                );
                self.download_attachment(url, filename, limit, path).await
            }
        }
    }

    async fn download_web(&self, url: &Url, limit: u64, path: &Path) -> Result<PathBuf> {
        ensure!(self.accepts(url), "Unsupported media website");
        let output = path.join("media.%(ext)s");
        let format = format_selector(limit);
        // Some extractors (notably Instagram) omit duration and size metadata
        // until after their separate video/audio streams have been merged. Let
        // those entries reach the bounded download, then verify the actual
        // artifact with ffprobe below.
        let filter = format!(
            "!is_live & duration <=? {}",
            self.config.max_duration_seconds
        );
        let mut child = Command::new(&self.config.yt_dlp_path)
            .args([
                "--ignore-config",
                "--no-warnings",
                "--no-progress",
                "--no-playlist",
                "--js-runtimes",
                "deno",
                "--sleep-requests",
                "1",
                "--extractor-args",
                "youtube:player_client=mweb",
                "--extractor-args",
                "youtubepot-bgutilhttp:base_url=http://pot-provider:4416",
                "--socket-timeout",
                "15",
                "--retries",
                "1",
                "--max-filesize",
                &limit.to_string(),
                "--match-filter",
                &filter,
                "--merge-output-format",
                "mp4",
                "--restrict-filenames",
                "-f",
                &format,
                "-o",
            ])
            .arg(&output)
            .arg("--")
            .arg(url.as_str())
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .kill_on_drop(true)
            .spawn()
            .map_err(|_| {
                anyhow::anyhow!("yt-dlp is unavailable; ask the operator to install it")
            })?;
        let stderr = child
            .stderr
            .take()
            .context("Downloader diagnostics unavailable")?;
        let diagnostics = async {
            let mut bytes = Vec::new();
            stderr.take(65537).read_to_end(&mut bytes).await?;
            ensure!(
                bytes.len() <= 65536,
                "Downloader diagnostics exceeded limit"
            );
            Ok::<_, anyhow::Error>(bytes)
        };
        let (diagnostics, status) = tokio::try_join!(diagnostics, async {
            Ok::<_, anyhow::Error>(child.wait().await?)
        })?;
        if !status.success() {
            let text = String::from_utf8_lossy(&diagnostics).to_ascii_lowercase();
            let (reason, category) = classify_download_failure(&text);
            tracing::warn!(category, "Media provider rejected a download request");
            anyhow::bail!(reason);
        }
        let output = single_output(path).await?;
        self.validate_download_duration(&output).await?;
        Ok(output)
    }

    async fn validate_download_duration(&self, path: &Path) -> Result<()> {
        let output = tokio::time::timeout(
            Duration::from_secs(self.config.timeout_seconds),
            Command::new("ffprobe")
                .args([
                    "-v",
                    "error",
                    "-show_entries",
                    "format=duration",
                    "-of",
                    "default=noprint_wrappers=1:nokey=1",
                ])
                .arg(path)
                .stdin(Stdio::null())
                .stderr(Stdio::null())
                .kill_on_drop(true)
                .output(),
        )
        .await
        .context("Media duration check timed out")?
        .map_err(|_| anyhow::anyhow!("FFprobe is unavailable on this host"))?;
        ensure!(output.status.success(), "Could not verify media duration");
        ensure!(
            output.stdout.len() <= 128,
            "Media duration information was too large"
        );
        let duration = String::from_utf8_lossy(&output.stdout)
            .trim()
            .parse::<f64>()
            .context("Could not verify media duration")?;
        ensure!(
            duration.is_finite() && duration >= 0.0,
            "Could not verify media duration"
        );
        ensure!(
            duration <= self.config.max_duration_seconds as f64,
            "The video exceeds the duration limit"
        );
        Ok(())
    }

    async fn download_attachment(
        &self,
        url: &Url,
        filename: &str,
        limit: u64,
        path: &Path,
    ) -> Result<PathBuf> {
        ensure!(discord_cdn(url), "Unsupported attachment host");
        let client = pinned_client(url).await?;
        let response = client
            .get(url.clone())
            .send()
            .await
            .context("Attachment download failed")?;
        ensure!(response.status().is_success(), "Attachment download failed");
        if let Some(length) = response
            .headers()
            .get(CONTENT_LENGTH)
            .and_then(|value| value.to_str().ok())
            .and_then(|value| value.parse::<u64>().ok())
        {
            ensure!(
                length <= limit,
                "The attachment exceeds Discord's upload limit"
            );
        }
        let extension = Path::new(filename)
            .extension()
            .and_then(|value| value.to_str())
            .filter(|value| value.len() <= 10)
            .unwrap_or("mp4");
        let output = path.join(format!("media.{extension}"));
        let mut file = tokio::fs::File::create(&output).await?;
        let mut written = 0_u64;
        let mut stream = response.bytes_stream();
        while let Some(chunk) = stream
            .try_next()
            .await
            .context("Attachment download failed")?
        {
            written = written.saturating_add(chunk.len() as u64);
            ensure!(
                written <= limit,
                "The attachment exceeds Discord's upload limit"
            );
            file.write_all(&chunk).await?;
        }
        file.flush().await?;
        Ok(output)
    }

    pub async fn health(&self) -> bool {
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
        .is_ok_and(|result| result.is_ok_and(|status| status.success()))
    }
}

fn describe_probe(value: &serde_json::Value) -> Result<String> {
    let format = value.get("format").and_then(serde_json::Value::as_object);
    let duration = format
        .and_then(|format| format.get("duration"))
        .and_then(serde_json::Value::as_str)
        .and_then(|value| value.parse::<f64>().ok())
        .map(|value| format!("{value:.1}s"))
        .unwrap_or_else(|| "unknown".to_owned());
    let size = format
        .and_then(|format| format.get("size"))
        .and_then(serde_json::Value::as_str)
        .and_then(|value| value.parse::<u64>().ok())
        .map(human_size)
        .unwrap_or_else(|| "unknown".to_owned());
    let container = format
        .and_then(|format| format.get("format_name"))
        .and_then(serde_json::Value::as_str)
        .unwrap_or("unknown");
    let streams = value
        .get("streams")
        .and_then(serde_json::Value::as_array)
        .context("Media information was incomplete")?;
    let video = streams
        .iter()
        .find(|stream| stream["codec_type"] == "video");
    let audio = streams
        .iter()
        .find(|stream| stream["codec_type"] == "audio");
    let video = video.map_or_else(
        || "none".to_owned(),
        |stream| {
            format!(
                "{} ({}×{})",
                stream["codec_name"].as_str().unwrap_or("unknown"),
                stream["width"].as_u64().unwrap_or(0),
                stream["height"].as_u64().unwrap_or(0)
            )
        },
    );
    let audio = audio.map_or_else(
        || "none".to_owned(),
        |stream| {
            format!(
                "{} ({} channels, {} Hz)",
                stream["codec_name"].as_str().unwrap_or("unknown"),
                stream["channels"].as_u64().unwrap_or(0),
                stream["sample_rate"].as_str().unwrap_or("unknown")
            )
        },
    );
    Ok(format!(
        "Container: `{container}`\nDuration: `{duration}`\nSize: `{size}`\nVideo: `{video}`\nAudio: `{audio}`"
    ))
}

fn human_size(bytes: u64) -> String {
    if bytes >= 1024 * 1024 {
        format!("{:.1} MiB", bytes as f64 / (1024.0 * 1024.0))
    } else {
        format!("{} KiB", bytes / 1024)
    }
}

fn classify_download_failure(text: &str) -> (&'static str, &'static str) {
    if text.contains("larger than max-filesize") || text.contains("max-filesize") {
        ("No available video fits Discord's upload limit", "size")
    } else if text.contains("duration") || text.contains("live") {
        (
            "The video is live or exceeds the duration limit",
            "duration",
        )
    } else if text.contains("confirm you're not a bot")
        || text.contains("confirm you’re not a bot")
        || text.contains("bot check")
    {
        (
            "YouTube blocked this server temporarily; try again shortly",
            "youtube_bot_check",
        )
    } else if text.contains("private video")
        || text.contains("members-only")
        || text.contains("login required")
        || text.contains("requires authentication")
    {
        (
            "The source is private or requires authentication",
            "authentication",
        )
    } else {
        (
            "The media provider could not download this video",
            "provider",
        )
    }
}

fn format_selector(limit: u64) -> String {
    const MIB: u64 = 1024 * 1024;
    // Reserve a bounded portion of the Discord limit for audio. yt-dlp applies
    // each selector constraint before merging, and the final artifact is
    // checked against the full limit before upload.
    let audio_limit = (limit / 5).clamp(2 * MIB, 32 * MIB).min(limit / 2);
    let video_limit = limit.saturating_sub(audio_limit);
    format!(
        "bv[filesize<={video_limit}][vcodec!=none]+ba[filesize<={audio_limit}]/\
bv[filesize_approx<={video_limit}][vcodec!=none]+ba[filesize_approx<={audio_limit}]/\
b[filesize<={limit}][vcodec!=none]/b[filesize_approx<={limit}][vcodec!=none]/\
bv[vcodec!=none]+ba/b[vcodec!=none]"
    )
}

async fn single_output(path: &Path) -> Result<PathBuf> {
    let mut entries = tokio::fs::read_dir(path).await?;
    let mut output = None;
    while let Some(entry) = entries.next_entry().await? {
        let file_type = entry.file_type().await?;
        ensure!(
            file_type.is_file() && !file_type.is_symlink(),
            "Downloader produced an invalid output"
        );
        ensure!(output.is_none(), "Downloader produced multiple files");
        output = Some(entry.path());
    }
    output.context("Downloader produced no media file")
}

fn discord_cdn(url: &Url) -> bool {
    url.scheme() == "https"
        && url.username().is_empty()
        && url.password().is_none()
        && url.port().is_none_or(|port| port == 443)
        && url.host_str().is_some_and(|host| {
            ["cdn.discordapp.com", "media.discordapp.net"]
                .iter()
                .any(|domain| host == *domain || host.ends_with(&format!(".{domain}")))
        })
}

fn public_ip(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(value) => {
            let octets = value.octets();
            !(value.is_private()
                || value.is_loopback()
                || value.is_link_local()
                || value.is_multicast()
                || value.is_broadcast()
                || value.is_documentation()
                || value.is_unspecified()
                || octets[0] == 0
                || octets[0] >= 240
                || (octets[0] == 100 && (64..=127).contains(&octets[1]))
                || (octets[0] == 198 && (octets[1] == 18 || octets[1] == 19))
                || (octets[0] == 192 && octets[1] == 0))
        }
        IpAddr::V6(value) => {
            if let Some(v4) = value.to_ipv4_mapped() {
                return public_ip(IpAddr::V4(v4));
            }
            let segments = value.segments();
            (segments[0] & 0xe000) == 0x2000
                && !(segments[0] == 0x2001 && (segments[1] <= 0x1ff || segments[1] == 0xdb8))
                && segments[0] != 0x2002
        }
    }
}

async fn pinned_client(url: &Url) -> Result<reqwest::Client> {
    let host = url.host_str().context("Attachment URL has no hostname")?;
    let port = url
        .port_or_known_default()
        .context("Attachment URL has no port")?;
    let addresses: Vec<SocketAddr> = tokio::time::timeout(
        Duration::from_secs(5),
        tokio::net::lookup_host((host, port)),
    )
    .await
    .context("Attachment DNS lookup timed out")??
    .collect();
    ensure!(
        !addresses.is_empty() && addresses.iter().all(|address| public_ip(address.ip())),
        "Attachment host must resolve exclusively to public addresses"
    );
    Ok(reqwest::Client::builder()
        .no_proxy()
        .redirect(reqwest::redirect::Policy::none())
        .connect_timeout(Duration::from_secs(10))
        .read_timeout(Duration::from_secs(30))
        .resolve_to_addrs(host, &addresses)
        .build()?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn allowlist_requires_exact_or_subdomain_match() {
        let service = DownloadService::new(&bot_core::AppConfig::default());
        assert!(service.accepts(&Url::parse("https://www.youtube.com/watch?v=x").unwrap()));
        assert!(!service.accepts(&Url::parse("https://youtube.com.example.test/watch").unwrap()));
        assert!(!service.accepts(&Url::parse("http://youtube.com/watch?v=x").unwrap()));
    }

    #[test]
    fn cdn_hosts_are_bounded() {
        assert!(discord_cdn(
            &Url::parse("https://cdn.discordapp.com/attachments/a/b").unwrap()
        ));
        assert!(!discord_cdn(
            &Url::parse("https://cdn.discordapp.com.example.test/a").unwrap()
        ));
    }

    #[test]
    fn format_selector_supports_bounded_video_audio_merges() {
        let selector = format_selector(100 * 1024 * 1024);
        assert!(selector.contains("bv[filesize<=83886080]"));
        assert!(selector.contains("+ba[filesize<=20971520]"));
        assert!(selector.contains("b[filesize<=104857600]"));
        assert!(selector.contains("bv[vcodec!=none]+ba/b[vcodec!=none]"));
    }

    #[test]
    fn download_failures_distinguish_bot_checks_from_private_media() {
        assert_eq!(
            classify_download_failure("sign in to confirm you're not a bot").1,
            "youtube_bot_check"
        );
        assert_eq!(
            classify_download_failure("this is a private video").1,
            "authentication"
        );
        assert_eq!(
            classify_download_failure("login attempt failed").1,
            "provider"
        );
    }

    #[test]
    fn formats_bounded_probe_metadata() {
        let description = describe_probe(&serde_json::json!({
            "format": {"format_name": "mov,mp4", "duration": "12.5", "size": "2097152"},
            "streams": [
                {"codec_type": "video", "codec_name": "h264", "width": 1280, "height": 720},
                {"codec_type": "audio", "codec_name": "aac", "channels": 2, "sample_rate": "48000"}
            ]
        }))
        .unwrap();
        assert!(description.contains("12.5s"));
        assert!(description.contains("h264 (1280×720)"));
        assert!(description.contains("aac (2 channels, 48000 Hz)"));
    }
}
