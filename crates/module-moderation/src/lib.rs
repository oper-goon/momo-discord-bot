#![forbid(unsafe_code)]
use anyhow::{Result, ensure};
use bot_core::{serenity, *};
pub fn module() -> RegisteredModule {
    RegisteredModule::new(
        "moderation",
        "Moderation",
        false,
        false,
        vec![
            kick(),
            ban(),
            unban(),
            timeout(),
            untimeout(),
            purge(),
            warn(),
            warnings(),
            case(),
        ],
    )
}
fn reason(value: Option<String>) -> Result<String> {
    let value = value.unwrap_or_else(|| "No reason provided".into());
    ensure!(
        !value.trim().is_empty() && value.chars().count() <= 400,
        "Reason must be 1–400 characters"
    );
    Ok(value)
}
async fn finish(ctx: Context<'_>, case: i64, result: Result<()>) -> Result<()> {
    let success = result.is_ok();
    if ctx
        .data()
        .database
        .finish_case(guild_id(ctx)?.get(), case, success)
        .await
        .is_err()
    {
        tracing_error(ctx, case);
        reply(
            ctx,
            format!(
                "Discord action {} but the audit update failed. Case #{case} needs reconciliation.",
                if success { "succeeded" } else { "failed" }
            ),
        )
        .await?;
        return Ok(());
    }
    result?;
    reply(ctx, format!("Action completed. Case #{case}.")).await
}
fn tracing_error(ctx: Context<'_>, case: i64) {
    tracing::error!(guild=?ctx.guild_id(),case, "CRITICAL: Discord action completed but moderation audit status could not be persisted");
}
/// Kick a member with an audit reason.
#[poise::command(
    slash_command,
    prefix_command,
    guild_only,
    required_permissions = "KICK_MEMBERS",
    required_bot_permissions = "KICK_MEMBERS"
)]
async fn kick(ctx: Context<'_>, user: serenity::User, #[rest] why: Option<String>) -> Result<()> {
    ctx.defer_ephemeral().await?;
    require_hierarchy(ctx, user.id, None).await?;
    let why = reason(why)?;
    let guild = guild_id(ctx)?;
    let n = ctx
        .data()
        .database
        .begin_case(
            guild.get(),
            "kick",
            user.id.get(),
            ctx.author().id.get(),
            Some(&why),
        )
        .await?;
    finish(
        ctx,
        n,
        guild
            .kick_with_reason(ctx.http(), user.id, &why)
            .await
            .map_err(Into::into),
    )
    .await
}
/// Ban a server member with a bounded message deletion window.
#[poise::command(
    slash_command,
    prefix_command,
    guild_only,
    required_permissions = "BAN_MEMBERS",
    required_bot_permissions = "BAN_MEMBERS"
)]
async fn ban(
    ctx: Context<'_>,
    user: serenity::User,
    #[min = 0]
    #[max = 7]
    delete_days: Option<u8>,
    #[rest] why: Option<String>,
) -> Result<()> {
    ctx.defer_ephemeral().await?;
    require_hierarchy(ctx, user.id, None).await?;
    let why = reason(why)?;
    let guild = guild_id(ctx)?;
    let n = ctx
        .data()
        .database
        .begin_case(
            guild.get(),
            "ban",
            user.id.get(),
            ctx.author().id.get(),
            Some(&why),
        )
        .await?;
    finish(
        ctx,
        n,
        guild
            .ban_with_reason(ctx.http(), user.id, delete_days.unwrap_or(0), &why)
            .await
            .map_err(Into::into),
    )
    .await
}
/// Remove a ban by user ID.
#[poise::command(
    slash_command,
    prefix_command,
    guild_only,
    required_permissions = "BAN_MEMBERS",
    required_bot_permissions = "BAN_MEMBERS"
)]
async fn unban(
    ctx: Context<'_>,
    user: serenity::UserId,
    #[rest] why: Option<String>,
) -> Result<()> {
    ctx.defer_ephemeral().await?;
    let why = reason(why)?;
    let guild = guild_id(ctx)?;
    let n = ctx
        .data()
        .database
        .begin_case(
            guild.get(),
            "unban",
            user.get(),
            ctx.author().id.get(),
            Some(&why),
        )
        .await?;
    finish(
        ctx,
        n,
        ctx.http()
            .remove_ban(guild, user, Some(&why))
            .await
            .map_err(Into::into),
    )
    .await
}
/// Timeout a member for up to 28 days.
#[poise::command(
    slash_command,
    prefix_command,
    guild_only,
    required_permissions = "MODERATE_MEMBERS",
    required_bot_permissions = "MODERATE_MEMBERS"
)]
async fn timeout(
    ctx: Context<'_>,
    user: serenity::User,
    #[min = 1]
    #[max = 2419200]
    seconds: u32,
    #[rest] why: Option<String>,
) -> Result<()> {
    ctx.defer_ephemeral().await?;
    require_hierarchy(ctx, user.id, None).await?;
    let why = reason(why)?;
    let guild = guild_id(ctx)?;
    let n = ctx
        .data()
        .database
        .begin_case(
            guild.get(),
            "timeout",
            user.id.get(),
            ctx.author().id.get(),
            Some(&why),
        )
        .await?;
    let until = serenity::Timestamp::from_unix_timestamp(now_ms() / 1000 + i64::from(seconds))?;
    finish(
        ctx,
        n,
        guild
            .edit_member(
                ctx.http(),
                user.id,
                serenity::EditMember::new()
                    .disable_communication_until_datetime(until)
                    .audit_log_reason(&why),
            )
            .await
            .map(|_| ())
            .map_err(Into::into),
    )
    .await
}
/// Remove a member timeout.
#[poise::command(
    slash_command,
    prefix_command,
    guild_only,
    required_permissions = "MODERATE_MEMBERS",
    required_bot_permissions = "MODERATE_MEMBERS"
)]
async fn untimeout(
    ctx: Context<'_>,
    user: serenity::User,
    #[rest] why: Option<String>,
) -> Result<()> {
    ctx.defer_ephemeral().await?;
    require_hierarchy(ctx, user.id, None).await?;
    let why = reason(why)?;
    let guild = guild_id(ctx)?;
    let n = ctx
        .data()
        .database
        .begin_case(
            guild.get(),
            "untimeout",
            user.id.get(),
            ctx.author().id.get(),
            Some(&why),
        )
        .await?;
    finish(
        ctx,
        n,
        guild
            .edit_member(
                ctx.http(),
                user.id,
                serenity::EditMember::new()
                    .enable_communication()
                    .audit_log_reason(&why),
            )
            .await
            .map(|_| ())
            .map_err(Into::into),
    )
    .await
}
/// Delete up to 100 recent messages, optionally by author.
#[poise::command(
    slash_command,
    prefix_command,
    guild_only,
    required_permissions = "MANAGE_MESSAGES",
    required_bot_permissions = "MANAGE_MESSAGES | READ_MESSAGE_HISTORY"
)]
async fn purge(
    ctx: Context<'_>,
    #[min = 1]
    #[max = 100]
    count: u8,
    user: Option<serenity::User>,
) -> Result<()> {
    ctx.defer_ephemeral().await?;
    let messages = ctx
        .channel_id()
        .messages(ctx.http(), serenity::GetMessages::new().limit(count))
        .await?;
    let ids: Vec<_> = messages
        .into_iter()
        .filter(|m| {
            user.as_ref().is_none_or(|u| u.id == m.author.id)
                && m.id.created_at().unix_timestamp() > now_ms() / 1000 - 14 * 86400 + 60
        })
        .map(|m| m.id)
        .collect();
    ensure!(!ids.is_empty(), "No matching messages younger than 14 days");
    let n = ctx
        .data()
        .database
        .begin_case(
            guild_id(ctx)?.get(),
            "purge",
            user.as_ref()
                .map(|u| u.id.get())
                .unwrap_or(ctx.author().id.get()),
            ctx.author().id.get(),
            Some("Recent-message purge"),
        )
        .await?;
    let result = if ids.len() == 1 {
        ctx.channel_id().delete_message(ctx.http(), ids[0]).await
    } else {
        ctx.channel_id().delete_messages(ctx.http(), ids).await
    };
    finish(ctx, n, result.map_err(Into::into)).await
}
/// Record a warning in the audit history.
#[poise::command(
    slash_command,
    prefix_command,
    guild_only,
    required_permissions = "MODERATE_MEMBERS"
)]
async fn warn(ctx: Context<'_>, user: serenity::User, #[rest] why: String) -> Result<()> {
    require_hierarchy(ctx, user.id, None).await?;
    let why = reason(Some(why))?;
    let n = ctx
        .data()
        .database
        .begin_case(
            guild_id(ctx)?.get(),
            "warn",
            user.id.get(),
            ctx.author().id.get(),
            Some(&why),
        )
        .await?;
    finish(ctx, n, Ok(())).await
}
/// List recent warnings for a member.
#[poise::command(
    slash_command,
    prefix_command,
    guild_only,
    required_permissions = "MODERATE_MEMBERS"
)]
async fn warnings(ctx: Context<'_>, user: serenity::User) -> Result<()> {
    let cases = ctx
        .data()
        .database
        .cases(guild_id(ctx)?.get(), Some(user.id.get()), None)
        .await?;
    let lines: Vec<_> = cases
        .iter()
        .filter(|c| c.action == "warn")
        .map(|c| {
            format!(
                "#{} [{}] {}",
                c.number,
                c.status,
                c.reason.as_deref().unwrap_or("")
            )
        })
        .collect();
    reply(
        ctx,
        if lines.is_empty() {
            "No recent warnings.".into()
        } else {
            lines.join("\n")
        },
    )
    .await
}
/// Inspect an audit case.
#[poise::command(
    slash_command,
    prefix_command,
    guild_only,
    required_permissions = "MODERATE_MEMBERS"
)]
async fn case(ctx: Context<'_>, number: i64) -> Result<()> {
    let cases = ctx
        .data()
        .database
        .cases(guild_id(ctx)?.get(), None, Some(number))
        .await?;
    let c = cases
        .first()
        .ok_or_else(|| anyhow::anyhow!("Case not found"))?;
    reply(
        ctx,
        format!(
            "#{} {} [{}]\nTarget: {} • Moderator: {}\n{}",
            c.number,
            c.action,
            c.status,
            c.target,
            c.moderator,
            c.reason.as_deref().unwrap_or("")
        ),
    )
    .await
}
