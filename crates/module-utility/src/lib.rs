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
        vec![choose(), poll(), remind(), reminders(), timestamp()],
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
        .map(|r| format!("#{} <t:{}:R> {}", r.id, r.due_at_ms / 1000, r.message))
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
