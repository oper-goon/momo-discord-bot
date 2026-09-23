#![forbid(unsafe_code)]

use anyhow::{Context, Result, ensure};
use async_trait::async_trait;
use bot_core::{
    AppConfig, BotData, Context as BotContext, EventHandler, RegisteredModule,
    effective_permissions, serenity,
};
use reqwest::header::{ACCEPT, CONTENT_TYPE, USER_AGENT};
use std::{collections::HashMap, sync::Arc, time::Duration};
use tokio::sync::{Mutex, Semaphore};
use url::Url;

const WEBHOOK_NAME: &str = "Momo Social Embeds";
const MAX_PROBE_BYTES: usize = 256 * 1024;

#[derive(Clone, Copy)]
enum Platform {
    X,
    Bluesky,
    TikTok,
    Reddit,
    Threads,
    Instagram,
}

struct Creator {
    username: String,
    profile_url: String,
}

struct RewrittenLink {
    original_url: String,
    provider_url: String,
    creator: Option<Creator>,
}

#[derive(Default)]
struct ProviderMetadata {
    has_embed: bool,
    article_author: Option<String>,
    twitter_creator: Option<String>,
    title: Option<String>,
    site_name: Option<String>,
}

pub struct SocialEmbedService {
    client: reqwest::Client,
    instagram_providers: Vec<String>,
    probes: Semaphore,
    webhooks: Mutex<HashMap<serenity::ChannelId, serenity::Webhook>>,
}

impl SocialEmbedService {
    pub fn new(config: &AppConfig) -> Result<Self> {
        let timeout = Duration::from_secs(config.social_embeds.probe_timeout_seconds);
        let client = reqwest::Client::builder()
            .no_proxy()
            .redirect(reqwest::redirect::Policy::none())
            .connect_timeout(timeout)
            .read_timeout(timeout)
            .timeout(timeout)
            .build()?;
        Ok(Self {
            client,
            instagram_providers: config.social_embeds.instagram_providers.clone(),
            probes: Semaphore::new(8),
            webhooks: Mutex::new(HashMap::new()),
        })
    }

    async fn rewrite(&self, content: &str) -> Result<Option<String>> {
        if content
            .split_whitespace()
            .any(|word| word.eq_ignore_ascii_case("fxignore"))
        {
            return Ok(None);
        }

        let mut output = String::with_capacity(content.len());
        let mut links = Vec::new();
        let mut cursor = 0;
        let mut changed = false;
        while let Some(relative) = find_https(&content[cursor..]) {
            let start = cursor + relative;
            output.push_str(&content[cursor..start]);
            let raw_end = content[start..]
                .find(|character: char| {
                    character.is_whitespace() || character == '<' || character == '>'
                })
                .map_or(content.len(), |offset| start + offset);
            let (url_end, suffix_start) = trim_url_end(content, start, raw_end);
            let original = &content[start..url_end];
            let suppressed = start > 0 && content.as_bytes()[start - 1] == b'<';
            let in_code = inside_code_span(&content[..start]);
            let replacement = if suppressed || in_code {
                None
            } else {
                self.rewrite_url(original).await?
            };
            if let Some(replacement) = replacement {
                output.push_str(&replacement.provider_url);
                links.push(replacement);
                changed = true;
            } else {
                output.push_str(original);
            }
            output.push_str(&content[url_end..suffix_start]);
            cursor = suffix_start;
        }
        output.push_str(&content[cursor..]);
        if changed {
            let header = links.iter().map(link_header).collect::<Vec<_>>().join("\n");
            output.insert_str(0, &format!("{header}\n"));
        }
        ensure!(
            output.chars().count() <= 2000,
            "Corrected message exceeds Discord's limit"
        );
        Ok(changed.then_some(output))
    }

    async fn rewrite_url(&self, value: &str) -> Result<Option<RewrittenLink>> {
        let Ok(mut url) = Url::parse(value) else {
            return Ok(None);
        };
        if url.scheme() != "https"
            || url.username() != ""
            || url.password().is_some()
            || url.port().is_some()
        {
            return Ok(None);
        }
        let host = url
            .host_str()
            .unwrap_or_default()
            .trim_start_matches("www.")
            .to_ascii_lowercase();
        let path = url.path().to_ascii_lowercase();
        let (platform, provider) = match host.as_str() {
            "x.com" | "twitter.com" | "mobile.twitter.com" if path.contains("/status/") => {
                (Platform::X, "fixupx.com")
            }
            "bsky.app" if path.contains("/profile/") && path.contains("/post/") => {
                (Platform::Bluesky, "fxbsky.app")
            }
            "tiktok.com" | "m.tiktok.com" | "vm.tiktok.com" | "vt.tiktok.com"
                if path.contains("/video/")
                    || path.contains("/photo/")
                    || path.starts_with("/t/") =>
            {
                (Platform::TikTok, "tnktok.com")
            }
            "reddit.com" | "old.reddit.com" | "new.reddit.com" | "redd.it"
                if path.contains("/comments/") || host == "redd.it" =>
            {
                (Platform::Reddit, "vxreddit.com")
            }
            "threads.net" | "threads.com" if path.contains("/post/") => {
                (Platform::Threads, "fixthreads.seria.moe")
            }
            "instagram.com" | "m.instagram.com" if instagram_path(&path) => {
                let mut selected = None;
                for provider in &self.instagram_providers {
                    let mut candidate = url.clone();
                    candidate
                        .set_host(Some(provider))
                        .context("Invalid Instagram provider")?;
                    clean_tracking(&mut candidate);
                    if let Some(metadata) = self.fetch_metadata(&candidate).await
                        && metadata.has_embed
                    {
                        let creator = creator_from_url(Platform::Instagram, &url)
                            .or_else(|| creator_from_metadata(Platform::Instagram, &metadata));
                        selected = Some(RewrittenLink {
                            original_url: value.to_owned(),
                            provider_url: candidate.to_string(),
                            creator,
                        });
                        break;
                    }
                }
                return Ok(selected);
            }
            _ => return Ok(None),
        };
        let creator = creator_from_url(platform, &url);
        url.set_host(Some(provider))
            .context("Invalid embed provider")?;
        clean_tracking(&mut url);
        let creator = if creator.is_some() {
            creator
        } else {
            self.fetch_metadata(&url)
                .await
                .and_then(|metadata| creator_from_metadata(platform, &metadata))
        };
        Ok(Some(RewrittenLink {
            original_url: value.to_owned(),
            provider_url: url.to_string(),
            creator,
        }))
    }

    async fn fetch_metadata(&self, url: &Url) -> Option<ProviderMetadata> {
        let Ok(_permit) = self.probes.acquire().await else {
            return None;
        };
        let Ok(mut response) = self
            .client
            .get(url.clone())
            .header(USER_AGENT, "Discordbot/2.0")
            .header(ACCEPT, "text/html,application/xhtml+xml")
            .send()
            .await
        else {
            return None;
        };
        if !response.status().is_success()
            || response
                .headers()
                .get(CONTENT_TYPE)
                .and_then(|value| value.to_str().ok())
                .is_some_and(|value| {
                    !value.starts_with("text/html") && !value.starts_with("application/xhtml+xml")
                })
        {
            return None;
        }
        let mut body = Vec::new();
        loop {
            match response.chunk().await {
                Ok(Some(chunk)) if body.len() + chunk.len() <= MAX_PROBE_BYTES => {
                    body.extend_from_slice(&chunk);
                }
                Ok(None) => break,
                Ok(Some(_)) | Err(_) => return None,
            }
        }
        let body = String::from_utf8_lossy(&body);
        let lower = body.to_ascii_lowercase();
        Some(ProviderMetadata {
            has_embed: lower.contains("property=\"og:image")
                || lower.contains("property='og:image")
                || lower.contains("property=\"og:video")
                || lower.contains("property='og:video")
                || lower.contains("name=\"twitter:card")
                || lower.contains("name='twitter:card"),
            article_author: meta_content(&body, "property", "article:author"),
            twitter_creator: meta_content(&body, "name", "twitter:creator"),
            title: meta_content(&body, "property", "og:title"),
            site_name: meta_content(&body, "property", "og:site_name"),
        })
    }

    async fn repost(
        &self,
        ctx: &serenity::Context,
        message: &serenity::Message,
        content: String,
    ) -> Result<()> {
        let channel = message
            .channel_id
            .to_channel(&ctx.http)
            .await?
            .guild()
            .context("Guild channel unavailable")?;
        let (webhook_channel, thread_id) = match channel.kind {
            serenity::ChannelType::NewsThread
            | serenity::ChannelType::PublicThread
            | serenity::ChannelType::PrivateThread => (
                channel.parent_id.context("Thread parent unavailable")?,
                Some(channel.id),
            ),
            _ => (channel.id, None),
        };
        let webhook = self.webhook(ctx, webhook_channel).await?;
        let display_name = message
            .member
            .as_deref()
            .and_then(|member| member.nick.as_deref())
            .unwrap_or_else(|| message.author.display_name());
        let username = webhook_username(display_name);
        let mut builder = serenity::ExecuteWebhook::new()
            .content(content)
            .username(username)
            .avatar_url(message.author.face())
            .allowed_mentions(serenity::CreateAllowedMentions::new());
        if let Some(thread_id) = thread_id {
            builder = builder.in_thread(thread_id);
        }
        let repost = webhook
            .execute(&ctx.http, true, builder)
            .await?
            .context("Discord returned no webhook message")?;
        if let Err(error) = message.delete(&ctx.http).await {
            let _ = webhook
                .delete_message(&ctx.http, thread_id, repost.id)
                .await;
            return Err(error.into());
        }
        Ok(())
    }

    async fn webhook(
        &self,
        ctx: &serenity::Context,
        channel: serenity::ChannelId,
    ) -> Result<serenity::Webhook> {
        let mut cache = self.webhooks.lock().await;
        if let Some(webhook) = cache.get(&channel) {
            return Ok(webhook.clone());
        }
        let bot_id = ctx.cache.current_user().id;
        let webhook = channel
            .webhooks(&ctx.http)
            .await?
            .into_iter()
            .find(|webhook| {
                webhook.name.as_deref() == Some(WEBHOOK_NAME)
                    && webhook.token.is_some()
                    && webhook.user.as_ref().is_some_and(|user| user.id == bot_id)
            });
        let webhook = match webhook {
            Some(webhook) => webhook,
            None => {
                channel
                    .create_webhook(&ctx.http, serenity::CreateWebhook::new(WEBHOOK_NAME))
                    .await?
            }
        };
        cache.insert(channel, webhook.clone());
        Ok(webhook)
    }
}

struct SocialEmbedEvents {
    service: Arc<SocialEmbedService>,
}

#[async_trait]
impl EventHandler for SocialEmbedEvents {
    async fn handle(
        &self,
        ctx: serenity::Context,
        event: serenity::FullEvent,
        data: BotData,
    ) -> Result<()> {
        let serenity::FullEvent::Message {
            new_message: message,
        } = event
        else {
            return Ok(());
        };
        let Some(guild_id) = message.guild_id else {
            return Ok(());
        };
        if data.config.discord.prefix_commands {
            let prefix = match data.database.prefix(guild_id.get()).await? {
                Some((prefix, true)) => Some(prefix),
                Some((_, false)) => None,
                None => Some(data.config.discord.default_prefix.clone()),
            };
            let bot_id = ctx.cache.current_user().id;
            if prefix
                .as_ref()
                .is_some_and(|prefix| message.content.starts_with(prefix))
                || message.content.starts_with(&format!("<@{bot_id}>"))
                || message.content.starts_with(&format!("<@!{bot_id}>"))
            {
                return Ok(());
            }
        }
        if message.author.bot
            || message.webhook_id.is_some()
            || !message.attachments.is_empty()
            || !message.sticker_items.is_empty()
            || message.poll.is_some()
            || message.kind != serenity::MessageType::Regular
        {
            return Ok(());
        }
        if let Some(content) = self.service.rewrite(&message.content).await? {
            self.service.repost(&ctx, &message, content).await?;
        }
        Ok(())
    }
}

pub fn module(service: Arc<SocialEmbedService>) -> RegisteredModule {
    let mut module = RegisteredModule::new(
        "social-embeds",
        "Social Embeds",
        false,
        false,
        vec![repair_embed()],
    );
    module.descriptor.provided_services.push("social-embeds");
    module.intents =
        serenity::GatewayIntents::GUILD_MESSAGES | serenity::GatewayIntents::MESSAGE_CONTENT;
    module.subscriptions.push("message");
    module.handler = Some(Arc::new(SocialEmbedEvents { service }));
    module
}

/// Repair supported social links in this message.
#[poise::command(
    context_menu_command = "Repair Social Embed",
    guild_only,
    required_bot_permissions = "MANAGE_MESSAGES | MANAGE_WEBHOOKS"
)]
async fn repair_embed(ctx: BotContext<'_>, message: serenity::Message) -> Result<()> {
    ensure!(
        message.webhook_id.is_none(),
        "Webhook messages cannot be repaired"
    );
    ensure!(
        message.author.id == ctx.author().id
            || effective_permissions(ctx)
                .await?
                .is_some_and(|permissions| {
                    permissions.contains(serenity::Permissions::MANAGE_MESSAGES)
                }),
        "You can only repair your own messages"
    );
    let service = ctx.data().services.get::<SocialEmbedService>()?;
    let content = service
        .rewrite(&message.content)
        .await?
        .context("This message has no supported social link")?;
    service
        .repost(ctx.serenity_context(), &message, content)
        .await?;
    bot_core::reply(ctx, "Social embed repaired.").await
}

fn link_header(link: &RewrittenLink) -> String {
    let provider = Url::parse(&link.provider_url)
        .ok()
        .and_then(|url| url.host_str().map(str::to_owned))
        .unwrap_or_else(|| "Embed".to_owned());
    let mut header = format!(
        "[Original](<{}>) · [{}](<{}>)",
        link.original_url, provider, link.provider_url
    );
    if let Some(creator) = &link.creator {
        header.push_str(&format!(
            " · [@{}](<{}>)",
            creator.username, creator.profile_url
        ));
    }
    header
}

fn creator_from_url(platform: Platform, url: &Url) -> Option<Creator> {
    let segments: Vec<&str> = url
        .path_segments()?
        .filter(|part| !part.is_empty())
        .collect();
    let candidate = match platform {
        Platform::X => segments
            .iter()
            .position(|part| part.eq_ignore_ascii_case("status"))
            .and_then(|index| index.checked_sub(1))
            .and_then(|index| segments.get(index).copied()),
        Platform::Bluesky => segment_after(&segments, "profile"),
        Platform::TikTok | Platform::Threads => segments
            .first()
            .copied()
            .filter(|part| part.starts_with('@')),
        Platform::Reddit => None,
        Platform::Instagram => segment_after(&segments, "stories"),
    }?;
    creator(platform, candidate)
}

fn creator_from_metadata(platform: Platform, metadata: &ProviderMetadata) -> Option<Creator> {
    metadata
        .article_author
        .as_deref()
        .and_then(|value| creator_from_profile_url(platform, value))
        .or_else(|| {
            metadata
                .twitter_creator
                .as_deref()
                .and_then(|value| creator(platform, value))
        })
        .or_else(|| {
            metadata
                .site_name
                .as_deref()
                .and_then(|value| creator_from_metadata_text(platform, value))
        })
        .or_else(|| {
            metadata
                .title
                .as_deref()
                .and_then(|value| creator_from_metadata_text(platform, value))
        })
}

fn creator_from_metadata_text(platform: Platform, value: &str) -> Option<Creator> {
    let marker = match platform {
        Platform::Reddit => "u/",
        _ => "@",
    };
    let start = value.find(marker)? + marker.len();
    let username: String = value[start..]
        .chars()
        .take_while(|character| {
            character.is_ascii_alphanumeric() || matches!(character, '.' | '_' | '-')
        })
        .collect();
    creator(platform, &username)
}

fn creator_from_profile_url(platform: Platform, value: &str) -> Option<Creator> {
    let url = Url::parse(value).ok()?;
    if url.scheme() != "https" || url.port().is_some() || url.username() != "" {
        return None;
    }
    let host = url
        .host_str()?
        .trim_start_matches("www.")
        .trim_start_matches("m.")
        .to_ascii_lowercase();
    let valid_host = match platform {
        Platform::X => matches!(host.as_str(), "x.com" | "twitter.com"),
        Platform::Bluesky => host == "bsky.app",
        Platform::TikTok => host == "tiktok.com",
        Platform::Reddit => matches!(host.as_str(), "reddit.com" | "old.reddit.com"),
        Platform::Threads => matches!(host.as_str(), "threads.net" | "threads.com"),
        Platform::Instagram => host == "instagram.com",
    };
    if !valid_host {
        return None;
    }
    let segments: Vec<&str> = url
        .path_segments()?
        .filter(|part| !part.is_empty())
        .collect();
    let candidate = match platform {
        Platform::Bluesky => segment_after(&segments, "profile"),
        Platform::Reddit => {
            segment_after(&segments, "user").or_else(|| segment_after(&segments, "u"))
        }
        Platform::TikTok | Platform::Threads => segments.first().copied(),
        Platform::X | Platform::Instagram => segments.first().copied(),
    }?;
    creator(platform, candidate)
}

fn creator(platform: Platform, value: &str) -> Option<Creator> {
    let username = value
        .trim()
        .trim_start_matches('@')
        .strip_prefix("u/")
        .unwrap_or_else(|| value.trim().trim_start_matches('@'));
    if username.is_empty()
        || username.len() > 100
        || !username
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'-'))
    {
        return None;
    }
    let profile_url = match platform {
        Platform::X => format!("https://x.com/{username}"),
        Platform::Bluesky => format!("https://bsky.app/profile/{username}"),
        Platform::TikTok => format!("https://www.tiktok.com/@{username}"),
        Platform::Reddit => format!("https://www.reddit.com/user/{username}/"),
        Platform::Threads => format!("https://www.threads.net/@{username}"),
        Platform::Instagram => format!("https://www.instagram.com/{username}/"),
    };
    Some(Creator {
        username: username.to_owned(),
        profile_url,
    })
}

fn segment_after<'a>(segments: &'a [&str], marker: &str) -> Option<&'a str> {
    segments
        .iter()
        .position(|part| part.eq_ignore_ascii_case(marker))
        .and_then(|index| segments.get(index + 1).copied())
}

fn meta_content(html: &str, selector: &str, expected: &str) -> Option<String> {
    let lower = html.to_ascii_lowercase();
    let mut cursor = 0;
    while let Some(relative) = lower[cursor..].find("<meta") {
        let start = cursor + relative;
        let Some(relative_end) = lower[start..].find('>') else {
            break;
        };
        let end = start + relative_end + 1;
        let tag = &html[start..end];
        if tag_attribute(tag, selector).is_some_and(|value| value.eq_ignore_ascii_case(expected))
            && let Some(content) = tag_attribute(tag, "content")
        {
            return Some(decode_html_attribute(content));
        }
        cursor = end;
    }
    None
}

fn tag_attribute<'a>(tag: &'a str, wanted: &str) -> Option<&'a str> {
    let bytes = tag.as_bytes();
    let mut cursor = 0;
    while cursor < bytes.len() {
        while cursor < bytes.len()
            && (bytes[cursor].is_ascii_whitespace() || matches!(bytes[cursor], b'<' | b'/' | b'>'))
        {
            cursor += 1;
        }
        let name_start = cursor;
        while cursor < bytes.len()
            && !bytes[cursor].is_ascii_whitespace()
            && !matches!(bytes[cursor], b'=' | b'>' | b'/')
        {
            cursor += 1;
        }
        let name = &tag[name_start..cursor];
        while cursor < bytes.len() && bytes[cursor].is_ascii_whitespace() {
            cursor += 1;
        }
        if cursor >= bytes.len() || bytes[cursor] != b'=' {
            continue;
        }
        cursor += 1;
        while cursor < bytes.len() && bytes[cursor].is_ascii_whitespace() {
            cursor += 1;
        }
        let quote = bytes
            .get(cursor)
            .copied()
            .filter(|byte| matches!(byte, b'\'' | b'"'));
        if quote.is_some() {
            cursor += 1;
        }
        let value_start = cursor;
        while cursor < bytes.len()
            && quote.map_or_else(
                || !bytes[cursor].is_ascii_whitespace() && bytes[cursor] != b'>',
                |quote| bytes[cursor] != quote,
            )
        {
            cursor += 1;
        }
        let value = &tag[value_start..cursor];
        if name.eq_ignore_ascii_case(wanted) {
            return Some(value);
        }
        cursor += usize::from(quote.is_some());
    }
    None
}

fn decode_html_attribute(value: &str) -> String {
    value
        .replace("&amp;", "&")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&#x27;", "'")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
}

fn find_https(value: &str) -> Option<usize> {
    value.find("https://")
}

fn trim_url_end(content: &str, start: usize, raw_end: usize) -> (usize, usize) {
    let candidate = &content[start..raw_end];
    let trimmed = candidate.trim_end_matches(|character: char| {
        matches!(
            character,
            '.' | ',' | ';' | ':' | '!' | '?' | ')' | ']' | '}' | '"' | '\''
        )
    });
    (start + trimmed.len(), raw_end)
}

fn inside_code_span(prefix: &str) -> bool {
    let fences = prefix.matches("```").count();
    fences & 1 == 1 || (fences & 1 == 0 && prefix.matches('`').count() & 1 == 1)
}

fn instagram_path(path: &str) -> bool {
    path.starts_with("/p/")
        || path.starts_with("/reel/")
        || path.starts_with("/reels/")
        || path.starts_with("/stories/")
        || path.starts_with("/share/")
        || path.contains("/p/")
        || path.contains("/reel/")
        || path.contains("/reels/")
}

fn clean_tracking(url: &mut Url) {
    url.set_fragment(None);
    let pairs: Vec<(String, String)> = url
        .query_pairs()
        .filter(|(key, _)| {
            let key = key.to_ascii_lowercase();
            !key.starts_with("utm_")
                && !matches!(
                    key.as_str(),
                    "fbclid" | "igshid" | "ref_src" | "s" | "t" | "si"
                )
        })
        .map(|(key, value)| (key.into_owned(), value.into_owned()))
        .collect();
    url.set_query(None);
    if !pairs.is_empty() {
        url.query_pairs_mut().extend_pairs(pairs);
    }
}

fn webhook_username(display_name: &str) -> String {
    const SUFFIX: &str = " · via Momo";
    let available = 80usize.saturating_sub(SUFFIX.chars().count());
    let mut value: String = display_name.chars().take(available).collect();
    value.push_str(SUFFIX);
    value
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recognizes_supported_paths() {
        assert!(instagram_path("/p/abc/"));
        assert!(instagram_path("/someone/reel/abc/"));
        assert!(!instagram_path("/someone/"));
    }

    #[test]
    fn removes_tracking_without_losing_carousel_index() {
        let mut url =
            Url::parse("https://instagram.com/p/abc?img_index=2&utm_source=x&igshid=y").unwrap();
        clean_tracking(&mut url);
        assert_eq!(url.as_str(), "https://instagram.com/p/abc?img_index=2");
    }

    #[test]
    fn bounds_webhook_username() {
        let value = webhook_username(&"a".repeat(100));
        assert_eq!(value.chars().count(), 80);
        assert!(value.ends_with(" · via Momo"));
    }

    #[test]
    fn trims_sentence_punctuation() {
        let value = "https://x.com/a/status/1).";
        let (end, raw_end) = trim_url_end(value, 0, value.len());
        assert_eq!(&value[..end], "https://x.com/a/status/1");
        assert_eq!(raw_end, value.len());
    }

    #[tokio::test]
    async fn rewrites_common_platforms_and_cleans_tracking() {
        let service = SocialEmbedService::new(&AppConfig::default()).unwrap();
        let rewritten = service
            .rewrite(
                "look https://x.com/momo/status/123?s=20 and \
                 https://bsky.app/profile/a.test/post/xyz",
            )
            .await
            .unwrap()
            .unwrap();
        assert_eq!(
            rewritten,
            "[Original](<https://x.com/momo/status/123?s=20>) · \
             [fixupx.com](<https://fixupx.com/momo/status/123>) · \
             [@momo](<https://x.com/momo>)\n\
             [Original](<https://bsky.app/profile/a.test/post/xyz>) · \
             [fxbsky.app](<https://fxbsky.app/profile/a.test/post/xyz>) · \
             [@a.test](<https://bsky.app/profile/a.test>)\n\
             look https://fixupx.com/momo/status/123 and \
             https://fxbsky.app/profile/a.test/post/xyz"
        );
    }

    #[test]
    fn reads_creator_metadata_and_builds_platform_profile() {
        let metadata = ProviderMetadata {
            has_embed: true,
            article_author: meta_content(
                r#"<meta content="https://www.instagram.com/julianohodges/" property="article:author">"#,
                "property",
                "article:author",
            ),
            twitter_creator: None,
            title: None,
            site_name: None,
        };
        let creator = creator_from_metadata(Platform::Instagram, &metadata).unwrap();
        assert_eq!(creator.username, "julianohodges");
        assert_eq!(
            creator.profile_url,
            "https://www.instagram.com/julianohodges/"
        );
    }

    #[test]
    fn reads_reddit_creator_from_provider_site_name() {
        let metadata = ProviderMetadata {
            has_embed: true,
            article_author: None,
            twitter_creator: None,
            title: Some("A post title".to_owned()),
            site_name: Some("u/llogiq on r/rust - votes and comments".to_owned()),
        };
        let creator = creator_from_metadata(Platform::Reddit, &metadata).unwrap();
        assert_eq!(creator.username, "llogiq");
        assert_eq!(creator.profile_url, "https://www.reddit.com/user/llogiq/");
    }

    #[tokio::test]
    async fn respects_explicit_and_markdown_opt_outs() {
        let service = SocialEmbedService::new(&AppConfig::default()).unwrap();
        assert!(
            service
                .rewrite("fxignore https://x.com/momo/status/123")
                .await
                .unwrap()
                .is_none()
        );
        assert!(
            service
                .rewrite("<https://x.com/momo/status/123> `https://x.com/momo/status/456`")
                .await
                .unwrap()
                .is_none()
        );
    }
}
