#![forbid(unsafe_code)]
use anyhow::{Result, ensure};
use bot_core::{serenity, *};
use rand::prelude::IndexedRandom;
use std::{sync::Arc, time::Duration};
pub fn module() -> RegisteredModule {
    RegisteredModule::new(
        "utility",
        "Utility",
        false,
        true,
        vec![
            choose(),
            poll(),
            remind(),
            reminders(),
            reminder(),
            bookmark(),
            bookmark_message(),
            remind_message(),
            timestamp(),
        ],
    )
}
/// Choose one option separated by a vertical bar.
#[poise::command(slash_command, prefix_command)]
async fn choose(ctx: Context<'_>, #[rest] options: String) -> Result<()> {
    ensure!(options.len() <= 1800, "Options are too long");
    let options: Vec<_> = options
        .split('|')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .collect();
    ensure!(
        (2..=25).contains(&options.len()),
        "Provide 2–25 options separated by |"
    );
    let choice = options
        .choose(&mut rand::rng())
        .copied()
        .unwrap_or_default()
        .to_string();
    reply(ctx, choice).await
}
/// Create a poll.
#[poise::command(slash_command, prefix_command, guild_only, subcommands("poll_create"))]
async fn poll(ctx: Context<'_>) -> Result<()> {
    reply(ctx, "Use /poll create.").await
}
/// Create a native Discord poll with options separated by a vertical bar.
#[poise::command(
    slash_command,
    prefix_command,
    guild_only,
    rename = "create",
    required_bot_permissions = "SEND_MESSAGES"
)]
async fn poll_create(ctx: Context<'_>, question: String, #[rest] options: String) -> Result<()> {
    ensure!(question.chars().count() <= 300, "Question is too long");
    let options: Vec<_> = options
        .split('|')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .collect();
    ensure!(
        (2..=10).contains(&options.len()) && options.iter().all(|s| s.chars().count() <= 55),
        "Use 2–10 options, each at most 55 characters"
    );
    let poll = serenity::CreatePoll::new().question(question).answers(
        options
            .into_iter()
            .map(|option| serenity::CreatePollAnswer::new().text(option))
            .collect(),
    );
    ctx.channel_id()
        .send_message(
            ctx.http(),
            serenity::CreateMessage::new().poll(poll.duration(Duration::from_secs(86400))),
        )
        .await?;
    reply(ctx, "Poll created.").await
}
pub fn parse_duration(input: &str) -> Result<u64> {
    let suffix = input
        .chars()
        .last()
        .ok_or_else(|| anyhow::anyhow!("Use a duration such as 10m, 2h, or 1d"))?;
    let multiplier = match suffix {
        's' => 1,
        'm' => 60,
        'h' => 3600,
        'd' => 86400,
        _ => anyhow::bail!("Use s, m, h, or d"),
    };
    let seconds = input[..input.len() - suffix.len_utf8()]
        .parse::<u64>()?
        .checked_mul(multiplier)
        .ok_or_else(|| anyhow::anyhow!("Duration is too large"))?;
    ensure!(
        (1..=365 * 86400).contains(&seconds),
        "Duration must be between 1 second and 365 days"
    );
    Ok(seconds)
}
/// Save a reminder; duration examples: 10m, 2h, 1d.
#[poise::command(slash_command, prefix_command, user_cooldown = 5)]
async fn remind(ctx: Context<'_>, duration: String, #[rest] message: String) -> Result<()> {
    ensure!(
        !message.trim().is_empty() && message.chars().count() <= 1500,
        "Reminder must be 1–1500 characters"
    );
    let due = now_ms() + parse_duration(&duration)? as i64 * 1000;
    let id = ctx
        .data()
        .database
        .add_reminder(
            ctx.guild_id().map(|g| g.get()),
            ctx.channel_id().get(),
            ctx.author().id.get(),
            &message,
            due,
            None,
        )
        .await?;
    reply(
        ctx,
        format!("Reminder #{id} saved for <t:{}:F>.", due / 1000),
    )
    .await
}
/// List your pending reminders.
#[poise::command(slash_command, prefix_command)]
async fn reminders(ctx: Context<'_>) -> Result<()> {
    let rows = ctx
        .data()
        .database
        .reminders(Some(ctx.author().id.get()), None)
        .await?;
    let lines: Vec<_> = rows
        .iter()
        .map(|r| {
            format!(
                "#{} <t:{}:R>{} {}",
                r.id,
                r.due_at_ms / 1000,
                r.interval_seconds
                    .map(|seconds| format!(" (every {})", format_duration(seconds as u64)))
                    .unwrap_or_default(),
                r.message
            )
        })
        .collect();
    reply(
        ctx,
        if lines.is_empty() {
            "No pending reminders.".into()
        } else {
            lines.join("\n")
        },
    )
    .await
}
/// Edit, repeat, snooze, or cancel reminders.
#[poise::command(
    slash_command,
    prefix_command,
    subcommands(
        "reminder_repeat",
        "reminder_edit",
        "reminder_snooze",
        "reminder_cancel"
    ),
    subcommand_required
)]
async fn reminder(ctx: Context<'_>) -> Result<()> {
    reply(ctx, "Use a reminder subcommand.").await
}

/// Create a recurring reminder.
#[poise::command(slash_command, prefix_command, rename = "repeat", user_cooldown = 5)]
async fn reminder_repeat(
    ctx: Context<'_>,
    interval: String,
    #[rest] message: String,
) -> Result<()> {
    ensure!(
        !message.trim().is_empty() && message.chars().count() <= 1500,
        "Reminder must be 1–1500 characters"
    );
    let seconds = parse_duration(&interval)?;
    ensure!(
        seconds >= 60,
        "Recurring reminders must be at least one minute apart"
    );
    let due = now_ms() + seconds as i64 * 1000;
    let id = ctx
        .data()
        .database
        .add_reminder(
            ctx.guild_id().map(|guild| guild.get()),
            ctx.channel_id().get(),
            ctx.author().id.get(),
            &message,
            due,
            Some(seconds),
        )
        .await?;
    reply(
        ctx,
        format!("Recurring reminder #{id} starts <t:{}:R>.", due / 1000),
    )
    .await
}

/// Change when a reminder fires and optionally replace its message.
#[poise::command(slash_command, prefix_command, rename = "edit")]
async fn reminder_edit(
    ctx: Context<'_>,
    id: i64,
    duration: String,
    #[rest] message: Option<String>,
) -> Result<()> {
    edit_reminder(ctx, id, duration, message).await
}

async fn edit_reminder(
    ctx: Context<'_>,
    id: i64,
    duration: String,
    message: Option<String>,
) -> Result<()> {
    if let Some(message) = &message {
        ensure!(
            !message.trim().is_empty() && message.chars().count() <= 1500,
            "Reminder must be 1–1500 characters"
        );
    }
    let due = now_ms() + parse_duration(&duration)? as i64 * 1000;
    ensure!(
        ctx.data()
            .database
            .update_reminder(id, ctx.author().id.get(), due, message.as_deref())
            .await?,
        "Reminder not found"
    );
    reply(
        ctx,
        format!("Reminder #{id} updated for <t:{}:F>.", due / 1000),
    )
    .await
}

/// Move a reminder later without changing its message.
#[poise::command(slash_command, prefix_command, rename = "snooze")]
async fn reminder_snooze(ctx: Context<'_>, id: i64, duration: String) -> Result<()> {
    edit_reminder(ctx, id, duration, None).await
}

/// Delete one of your reminders.
#[poise::command(slash_command, prefix_command, rename = "cancel")]
async fn reminder_cancel(ctx: Context<'_>, id: i64) -> Result<()> {
    ensure!(
        ctx.data()
            .database
            .cancel_reminder(id, ctx.author().id.get())
            .await?,
        "Reminder not found"
    );
    reply(ctx, format!("Reminder #{id} cancelled.")).await
}

/// Manage private message bookmarks.
#[poise::command(
    slash_command,
    subcommands("bookmark_list", "bookmark_search", "bookmark_note", "bookmark_delete"),
    subcommand_required
)]
async fn bookmark(ctx: Context<'_>) -> Result<()> {
    reply(ctx, "Use a bookmark subcommand or right-click a message.").await
}

/// List your most recent bookmarks.
#[poise::command(slash_command, rename = "list")]
async fn bookmark_list(ctx: Context<'_>) -> Result<()> {
    show_bookmarks(ctx, None).await
}

/// Search your saved message excerpts and notes.
#[poise::command(slash_command, rename = "search")]
async fn bookmark_search(ctx: Context<'_>, #[rest] query: String) -> Result<()> {
    ensure!(
        !query.trim().is_empty() && query.chars().count() <= 100,
        "Search must be 1–100 characters"
    );
    show_bookmarks(ctx, Some(query.trim())).await
}

/// Add or replace a private note on a bookmark.
#[poise::command(slash_command, rename = "note")]
async fn bookmark_note(ctx: Context<'_>, id: i64, #[rest] note: String) -> Result<()> {
    ensure!(
        !note.trim().is_empty() && note.chars().count() <= 300,
        "Note must be 1–300 characters"
    );
    ensure!(
        ctx.data()
            .database
            .note_bookmark(id, ctx.author().id.get(), note.trim())
            .await?,
        "Bookmark not found"
    );
    reply(ctx, format!("Bookmark #{id} note updated.")).await
}

/// Delete one of your bookmarks.
#[poise::command(slash_command, rename = "delete")]
async fn bookmark_delete(ctx: Context<'_>, id: i64) -> Result<()> {
    ensure!(
        ctx.data()
            .database
            .delete_bookmark(id, ctx.author().id.get())
            .await?,
        "Bookmark not found"
    );
    reply(ctx, format!("Bookmark #{id} deleted.")).await
}

/// Save this message to your private bookmarks.
#[poise::command(context_menu_command = "Bookmark Message")]
async fn bookmark_message(ctx: Context<'_>, message: serenity::Message) -> Result<()> {
    let excerpt = message_excerpt(&message);
    let id = ctx
        .data()
        .database
        .add_bookmark(
            ctx.author().id.get(),
            message.guild_id.map(|guild| guild.get()),
            message.channel_id.get(),
            message.id.get(),
            message.author.id.get(),
            &excerpt,
        )
        .await?;
    reply(
        ctx,
        format!(
            "Bookmark #{id} saved: [jump to message](<{}>)",
            message_url(&message)
        ),
    )
    .await
}

/// Create a one-hour reminder for this message.
#[poise::command(context_menu_command = "Remind Me in 1 Hour")]
async fn remind_message(ctx: Context<'_>, message: serenity::Message) -> Result<()> {
    let due = now_ms() + 3600 * 1000;
    let text = format!(
        "Message from {}: {}\n{}",
        message.author.display_name(),
        message_excerpt(&message),
        message_url(&message)
    );
    let id = ctx
        .data()
        .database
        .add_reminder(
            message.guild_id.map(|guild| guild.get()),
            message.channel_id.get(),
            ctx.author().id.get(),
            &text,
            due,
            None,
        )
        .await?;
    reply(
        ctx,
        format!("Reminder #{id} saved for <t:{}:F>.", due / 1000),
    )
    .await
}

async fn show_bookmarks(ctx: Context<'_>, search: Option<&str>) -> Result<()> {
    let rows = ctx
        .data()
        .database
        .bookmarks(ctx.author().id.get(), search)
        .await?;
    let lines: Vec<String> = rows
        .iter()
        .take(10)
        .map(|bookmark| {
            let guild = bookmark.guild_id.as_deref().unwrap_or("@me");
            let note = bookmark
                .note
                .as_deref()
                .map(|note| format!(" — {note}"))
                .unwrap_or_default();
            format!(
                "#{} [jump](<https://discord.com/channels/{}/{}/{}>) <@{}> — {}{} · <t:{}:R>",
                bookmark.id,
                guild,
                bookmark.channel_id,
                bookmark.message_id,
                bookmark.author_user_id,
                bookmark.excerpt,
                note,
                bookmark.created_at_ms / 1000
            )
        })
        .collect();
    reply(
        ctx,
        if lines.is_empty() {
            "No matching bookmarks.".to_owned()
        } else {
            lines.join("\n")
        },
    )
    .await
}

fn message_excerpt(message: &serenity::Message) -> String {
    let value = if message.content.trim().is_empty() {
        message
            .attachments
            .first()
            .map(|attachment| attachment.filename.as_str())
            .unwrap_or("Message without text")
    } else {
        message.content.trim()
    };
    value
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .chars()
        .take(160)
        .collect()
}

fn message_url(message: &serenity::Message) -> String {
    format!(
        "https://discord.com/channels/{}/{}/{}",
        message
            .guild_id
            .map(|guild| guild.to_string())
            .unwrap_or_else(|| "@me".to_owned()),
        message.channel_id,
        message.id
    )
}

fn format_duration(seconds: u64) -> String {
    for (suffix, unit) in [('d', 86_400), ('h', 3_600), ('m', 60)] {
        if seconds.is_multiple_of(unit) {
            return format!("{}{}", seconds / unit, suffix);
        }
    }
    format!("{seconds}s")
}
/// Format a Unix timestamp in Discord's local time display.
#[poise::command(slash_command, prefix_command)]
async fn timestamp(ctx: Context<'_>, unix_seconds: i64) -> Result<()> {
    ensure!(
        (0..=253402300799).contains(&unix_seconds),
        "Timestamp is outside the supported range"
    );
    reply(
        ctx,
        format!("<t:{unix_seconds}:F> • `<t:{unix_seconds}:F>`"),
    )
    .await
}
pub fn start_reminders(data: BotData, http: Arc<serenity::Http>) {
    let supervisor = data.supervisor.clone();
    let cancel = supervisor.cancel.clone();
    supervisor.spawn("reminder-scheduler",async move{let mut interval=tokio::time::interval(Duration::from_secs(15));loop{tokio::select!{_ = cancel.cancelled()=>break,_=interval.tick()=>{let rows=match data.database.reminders(None,Some(now_ms())).await{Ok(v)=>v,Err(_)=>continue};for r in rows{if cancel.is_cancelled(){break;}
if let Some(g)=r.guild_id.as_ref().and_then(|g|g.parse::<u64>().ok())&& !data.modules.enabled(&data.database,g,"utility").await.unwrap_or(false){continue;}let success=if let Some(channel)=r.channel_id.and_then(|c|c.parse::<u64>().ok()){serenity::ChannelId::new(channel).send_message(&http,serenity::CreateMessage::new().content(format!("Reminder for <@{}>: {}",r.user_id,r.message)).allowed_mentions(serenity::CreateAllowedMentions::new().users(r.user_id.parse::<u64>().ok().map(serenity::UserId::new)))).await.is_ok()}else{false};data.database.reminder_result(r.id,success).await?;}}}}Ok(())});
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn bounded_duration() {
        assert_eq!(parse_duration("2h").unwrap(), 7200);
        assert!(parse_duration("0s").is_err());
        assert!(parse_duration("999999999999999999999d").is_err());
        assert!(parse_duration("∞").is_err());
    }
}
