use crate::{DownloadService, service::MediaSource};
use anyhow::{Context as _, Result, ensure};
use bot_core::{Command, Context, serenity};
use reqwest::{Body, header::AUTHORIZATION, multipart};
use serde_json::json;
use std::{path::Path, sync::Arc, time::Duration};
use tokio_util::io::ReaderStream;
use url::Url;

pub fn commands() -> Vec<Command> {
    vec![download()]
}

fn service(ctx: Context<'_>) -> Result<Arc<DownloadService>> {
    ctx.data().services.get::<DownloadService>()
}

/// Download and reupload media from a URL, replied message, or recent chat.
#[poise::command(
    slash_command,
    prefix_command,
    guild_only,
    user_cooldown = 15,
    required_bot_permissions = "SEND_MESSAGES | ATTACH_FILES"
)]
async fn download(
    ctx: Context<'_>,
    #[description = "Public video URL; omit to inspect a reply or recent messages"] url: Option<
        String,
    >,
) -> Result<()> {
    let service = service(ctx)?;
    let source = resolve_source(ctx, &service, url.as_deref()).await?;
    let limit = upload_limit(ctx, service.config().absolute_max_bytes).await?;
    if let MediaSource::Attachment { declared_size, .. } = &source {
        ensure!(
            *declared_size <= limit,
            "The attachment exceeds Discord's upload limit for this channel"
        );
    }
    let artifact = service.download(source, limit).await?;
    upload(ctx, &artifact.path, &artifact.filename, limit).await
}

async fn resolve_source(
    ctx: Context<'_>,
    service: &DownloadService,
    explicit: Option<&str>,
) -> Result<MediaSource> {
    if let Some(value) = explicit.map(str::trim).filter(|value| !value.is_empty()) {
        return web_source(service, value);
    }

    if let poise::Context::Prefix(prefix) = ctx {
        if let Some(message) = &prefix.msg.referenced_message
            && let Some(source) = message_source(service, message)
        {
            return Ok(source);
        }
        if let Some(reference) = &prefix.msg.message_reference
            && let Some(message) = reference.message_id
        {
            let message = reference.channel_id.message(ctx.http(), message).await?;
            if let Some(source) = message_source(service, &message) {
                return Ok(source);
            }
        }
    }

    let mut builder = serenity::GetMessages::new().limit(service.config().scan_messages);
    if let poise::Context::Prefix(prefix) = ctx {
        builder = builder.before(prefix.msg.id);
    }
    for message in ctx.channel_id().messages(ctx.http(), builder).await? {
        if message.author.bot {
            continue;
        }
        if let Some(source) = message_source(service, &message) {
            return Ok(source);
        }
    }
    anyhow::bail!(
        "No supported media found; provide a URL, reply to media, or post a supported link recently"
    )
}

fn message_source(service: &DownloadService, message: &serenity::Message) -> Option<MediaSource> {
    if let Some(attachment) = message.attachments.iter().rev().find(|attachment| {
        attachment
            .content_type
            .as_deref()
            .is_some_and(|kind| kind.starts_with("video/"))
            || video_extension(&attachment.filename)
    }) && let Ok(url) = Url::parse(&attachment.url)
    {
        return Some(MediaSource::Attachment {
            url,
            filename: attachment.filename.clone(),
            declared_size: u64::from(attachment.size),
        });
    }
    extract_urls(&message.content)
        .into_iter()
        .rev()
        .find(|url| service.accepts(url))
        .map(MediaSource::Web)
}

fn web_source(service: &DownloadService, value: &str) -> Result<MediaSource> {
    let url = Url::parse(value).context("Provide a complete HTTPS video URL")?;
    ensure!(service.accepts(&url), "Unsupported media website");
    Ok(MediaSource::Web(url))
}

fn extract_urls(content: &str) -> Vec<Url> {
    content
        .split_whitespace()
        .filter_map(|token| {
            let value = token.trim_matches(|character: char| {
                matches!(
                    character,
                    '<' | '>' | '(' | ')' | '[' | ']' | '{' | '}' | ',' | ';' | '.' | '"' | '\''
                )
            });
            Url::parse(value).ok()
        })
        .filter(|url| url.scheme() == "https")
        .collect()
}

fn video_extension(filename: &str) -> bool {
    Path::new(filename)
        .extension()
        .and_then(|value| value.to_str())
        .is_some_and(|value| {
            ["mp4", "webm", "mov", "m4v", "gif"]
                .iter()
                .any(|extension| value.eq_ignore_ascii_case(extension))
        })
}

async fn upload_limit(ctx: Context<'_>, ceiling: u64) -> Result<u64> {
    let reported = match ctx {
        poise::Context::Application(application) => {
            u64::from(application.interaction.attachment_size_limit)
        }
        poise::Context::Prefix(_) => {
            let guild = bot_core::guild_id(ctx)?
                .to_partial_guild(ctx.http())
                .await?;
            prefix_tier_limit(guild.premium_tier)
        }
    };
    Ok(reported.min(ceiling))
}

fn prefix_tier_limit(tier: serenity::PremiumTier) -> u64 {
    match tier {
        serenity::PremiumTier::Tier2 => 50 * 1024 * 1024,
        serenity::PremiumTier::Tier3 => 100 * 1024 * 1024,
        _ => 20 * 1024 * 1024,
    }
}

fn human_size(bytes: u64) -> String {
    if bytes >= 1024 * 1024 {
        format!("{} MiB", bytes / (1024 * 1024))
    } else {
        format!("{} KiB", bytes / 1024)
    }
}

async fn upload(ctx: Context<'_>, path: &Path, filename: &str, limit: u64) -> Result<()> {
    let size = tokio::fs::metadata(path).await?.len();
    ensure!(
        size <= limit,
        "The downloaded media exceeds Discord's upload limit"
    );
    let file = tokio::fs::File::open(path).await?;
    let part =
        multipart::Part::stream_with_length(Body::wrap_stream(ReaderStream::new(file)), size)
            .file_name(filename.to_owned())
            .mime_str(media_type(filename))?;
    let payload = json!({
        "content": format!("Downloaded media (limit {}).", human_size(limit)),
        "allowed_mentions": { "parse": [] },
        "attachments": [{ "id": 0, "filename": filename }],
    });
    let form = multipart::Form::new()
        .text("payload_json", payload.to_string())
        .part("files[0]", part);
    let client = reqwest::Client::builder()
        .no_proxy()
        .redirect(reqwest::redirect::Policy::none())
        .connect_timeout(Duration::from_secs(10))
        .read_timeout(Duration::from_secs(600))
        .build()?;
    let request = match ctx {
        poise::Context::Application(application) => client.post(format!(
            "https://discord.com/api/v10/webhooks/{}/{}?wait=true",
            application.interaction.application_id, application.interaction.token
        )),
        poise::Context::Prefix(_) => client
            .post(format!(
                "https://discord.com/api/v10/channels/{}/messages",
                ctx.channel_id()
            ))
            .header(AUTHORIZATION, format!("Bot {}", ctx.http().token())),
    };
    let response = tokio::time::timeout(Duration::from_secs(600), request.multipart(form).send())
        .await
        .context("Discord upload timed out")?
        .context("Discord upload failed")?;
    ensure!(
        response.status().is_success(),
        "Discord rejected the media upload"
    );
    if let poise::Context::Application(application) = ctx {
        let _ = application.interaction.delete_response(ctx.http()).await;
    }
    Ok(())
}

fn media_type(filename: &str) -> &'static str {
    match Path::new(filename)
        .extension()
        .and_then(|value| value.to_str())
        .map(str::to_ascii_lowercase)
        .as_deref()
    {
        Some("webm") => "video/webm",
        Some("mov") => "video/quicktime",
        Some("gif") => "image/gif",
        _ => "video/mp4",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_last_clean_url() {
        let urls = extract_urls("first <https://youtu.be/a> then https://x.com/u/status/1).");
        assert_eq!(urls.len(), 2);
        assert_eq!(urls[1].host_str(), Some("x.com"));
        assert_eq!(urls[1].path(), "/u/status/1");
    }

    #[test]
    fn maps_known_prefix_tiers() {
        assert_eq!(
            prefix_tier_limit(serenity::PremiumTier::Tier0),
            20 * 1024 * 1024
        );
        assert_eq!(
            prefix_tier_limit(serenity::PremiumTier::Tier2),
            50 * 1024 * 1024
        );
        assert_eq!(
            prefix_tier_limit(serenity::PremiumTier::Tier3),
            100 * 1024 * 1024
        );
    }
}
