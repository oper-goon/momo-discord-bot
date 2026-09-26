use anyhow::{Result, ensure};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct AppConfig {
    pub discord: DiscordConfig,
    pub commands: CommandConfig,
    pub database: DatabaseConfig,
    pub music: MusicConfig,
    pub download: DownloadConfig,
    pub social_embeds: SocialEmbedsConfig,
    pub logging: LoggingConfig,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct DiscordConfig {
    pub owners: Vec<u64>,
    pub default_prefix: String,
    pub prefix_commands: bool,
    pub administrator_bypass: bool,
}
impl Default for DiscordConfig {
    fn default() -> Self {
        Self {
            owners: vec![],
            default_prefix: "!".into(),
            prefix_commands: false,
            administrator_bypass: true,
        }
    }
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct CommandConfig {
    pub mode: String,
    pub development_guilds: Vec<u64>,
}
impl Default for CommandConfig {
    fn default() -> Self {
        Self {
            mode: "none".into(),
            development_guilds: vec![],
        }
    }
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
}
impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            url: "sqlite://data/bot.db".into(),
            max_connections: 5,
        }
    }
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct MusicConfig {
    pub yt_dlp_path: String,
    pub resolver_timeout_seconds: u64,
    pub idle_disconnect_seconds: u64,
    pub empty_channel_disconnect_seconds: u64,
    pub default_volume_percent: u16,
    pub max_queue_length: usize,
    pub max_playlist_import: usize,
}
impl Default for MusicConfig {
    fn default() -> Self {
        Self {
            yt_dlp_path: "yt-dlp".into(),
            resolver_timeout_seconds: 30,
            idle_disconnect_seconds: 300,
            empty_channel_disconnect_seconds: 120,
            default_volume_percent: 100,
            max_queue_length: 500,
            max_playlist_import: 100,
        }
    }
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct DownloadConfig {
    pub yt_dlp_path: String,
    pub scan_messages: u8,
    pub timeout_seconds: u64,
    pub max_duration_seconds: u64,
    pub absolute_max_bytes: u64,
    pub allowed_domains: Vec<String>,
}
impl Default for DownloadConfig {
    fn default() -> Self {
        Self {
            yt_dlp_path: "yt-dlp".into(),
            scan_messages: 25,
            timeout_seconds: 180,
            max_duration_seconds: 900,
            absolute_max_bytes: 500 * 1024 * 1024,
            allowed_domains: vec![
                "youtube.com".into(),
                "youtu.be".into(),
                "tiktok.com".into(),
                "x.com".into(),
                "twitter.com".into(),
                "reddit.com".into(),
                "v.redd.it".into(),
                "instagram.com".into(),
            ],
        }
    }
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct LoggingConfig {
    pub format: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct SocialEmbedsConfig {
    pub probe_timeout_seconds: u64,
    pub instagram_providers: Vec<String>,
}
impl Default for SocialEmbedsConfig {
    fn default() -> Self {
        Self {
            probe_timeout_seconds: 6,
            instagram_providers: vec![
                "kkinstagram.com".into(),
                "oginstagram.com".into(),
                "vxinstagram.com".into(),
            ],
        }
    }
}
impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            format: "pretty".into(),
        }
    }
}
impl AppConfig {
    pub fn load(path: Option<&str>) -> Result<Self> {
        let mut c: Self = if let Some(path) = path {
            toml::from_str(&std::fs::read_to_string(path)?)?
        } else {
            Self::default()
        };
        if let Ok(url) = std::env::var("DATABASE_URL") {
            c.database.url = url;
        }
        if let Ok(path) = std::env::var("YT_DLP_PATH") {
            c.music.yt_dlp_path = path.clone();
            c.download.yt_dlp_path = path;
        }
        c.validate()?;
        Ok(c)
    }
    pub fn validate(&self) -> Result<()> {
        ensure!(
            !self.discord.default_prefix.trim().is_empty()
                && self.discord.default_prefix.chars().count() <= 16,
            "Prefix must contain 1–16 nonblank characters"
        );
        ensure!(
            ["none", "guild", "global"].contains(&self.commands.mode.as_str()),
            "Invalid command synchronization mode"
        );
        ensure!(
            self.commands.mode != "guild" || !self.commands.development_guilds.is_empty(),
            "Guild command mode requires development_guilds"
        );
        ensure!(
            self.commands.development_guilds.iter().all(|id| *id > 0),
            "Invalid guild ID"
        );
        ensure!(
            self.database.url.starts_with("sqlite:"),
            "Only SQLite is supported"
        );
        ensure!(
            (1..=10).contains(&self.database.max_connections),
            "Use 1–10 database connections"
        );
        ensure!(
            (1..=120).contains(&self.music.resolver_timeout_seconds),
            "Resolver timeout must be 1–120 seconds"
        );
        ensure!(
            (1..=5000).contains(&self.music.max_queue_length),
            "Queue limit must be 1–5000"
        );
        ensure!(
            (1..=self.music.max_queue_length).contains(&self.music.max_playlist_import),
            "Playlist limit must fit the queue limit"
        );
        ensure!(
            (1..=100).contains(&self.download.scan_messages),
            "Download history scan must be 1–100 messages"
        );
        ensure!(
            (10..=900).contains(&self.download.timeout_seconds),
            "Download timeout must be 10–900 seconds"
        );
        ensure!(
            (1..=86400).contains(&self.download.max_duration_seconds),
            "Download duration limit must be 1–86400 seconds"
        );
        ensure!(
            (1024 * 1024..=2 * 1024 * 1024 * 1024).contains(&self.download.absolute_max_bytes),
            "Download byte ceiling must be between 1 MiB and 2 GiB"
        );
        ensure!(
            !self.download.allowed_domains.is_empty()
                && self.download.allowed_domains.iter().all(|domain| {
                    !domain.is_empty()
                        && domain == &domain.to_ascii_lowercase()
                        && !domain.contains('/')
                        && !domain.starts_with('.')
                        && !domain.ends_with('.')
                }),
            "Download domains must be lowercase hostnames"
        );
        ensure!(
            (1..=15).contains(&self.social_embeds.probe_timeout_seconds),
            "Social embed probe timeout must be 1–15 seconds"
        );
        ensure!(
            !self.social_embeds.instagram_providers.is_empty()
                && self.social_embeds.instagram_providers.len() <= 5
                && self.social_embeds.instagram_providers.iter().all(|domain| {
                    [
                        "oginstagram.com",
                        "vxinstagram.com",
                        "kkinstagram.com",
                        "instagram7.com",
                        "zzinstagram.com",
                    ]
                    .contains(&domain.as_str())
                }),
            "Instagram providers must be 1–5 approved embed hosts"
        );
        self.music_defaults()
            .validate(self.music.max_queue_length)?;
        ensure!(
            ["pretty", "json"].contains(&self.logging.format.as_str()),
            "Logging format must be pretty or json"
        );
        Ok(())
    }
    pub fn music_defaults(&self) -> crate::MusicSettings {
        crate::MusicSettings {
            default_volume_percent: self.music.default_volume_percent,
            max_queue_length: self.music.max_queue_length,
            idle_disconnect_seconds: self.music.idle_disconnect_seconds,
            empty_channel_disconnect_seconds: self.music.empty_channel_disconnect_seconds,
            ..Default::default()
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn validates_complete_configuration() {
        let mut c: AppConfig = toml::from_str("[discord]\ndefault_prefix='?'\n").unwrap();
        assert_eq!(c.discord.default_prefix, "?");
        assert_eq!(c.music.max_queue_length, 500);
        assert!(c.validate().is_ok());
        c.commands.mode = "guild".into();
        assert!(c.validate().is_err());
        c.commands.development_guilds.push(1);
        assert!(c.validate().is_ok());
        c.music.default_volume_percent = 201;
        assert!(c.validate().is_err());
        assert!(toml::from_str::<AppConfig>("typo=1").is_err());

        let mut c = AppConfig::default();
        c.social_embeds.instagram_providers = vec!["localhost".into()];
        assert!(c.validate().is_err());
    }
}
