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
        let filter = format!(
            "!is_live & duration <= {}",
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
            let reason = if text.contains("larger than max-filesize")
                || text.contains("max-filesize")
            {
                "No available video fits Discord's upload limit"
            } else if text.contains("duration") || text.contains("live") {
                "The video is live or exceeds the duration limit"
            } else if text.contains("sign in") || text.contains("private") || text.contains("login")
            {
                "The source is private or requires authentication"
            } else {
                "The media provider could not download this video"
            };
            anyhow::bail!(reason);
        }
        single_output(path).await
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
b[filesize<={limit}][vcodec!=none]/b[filesize_approx<={limit}][vcodec!=none]/b[vcodec!=none]"
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
    }
}
