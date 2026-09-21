#![forbid(unsafe_code)]
use anyhow::{Result, ensure};
use bot_core::{serenity, *};
pub fn module() -> RegisteredModule {
    let mut module = RegisteredModule::new(
        "admin",
        "Administration",
        false,
        false,
        vec![announce(), slowmode(), nick()],
    );
    module
        .extensions
        .extend([("role", role_add()), ("role", role_remove())]);
    module
}
/// Send an announcement without mass mentions.
#[poise::command(
    slash_command,
    prefix_command,
    guild_only,
    required_permissions = "MANAGE_GUILD",
    required_bot_permissions = "SEND_MESSAGES"
)]
async fn announce(
    ctx: Context<'_>,
    channel: serenity::GuildChannel,
    #[rest] message: String,
) -> Result<()> {
    ensure!(
        channel.guild_id == guild_id(ctx)?,
        "Choose a channel in this server"
    );
    ensure!(
        !message.trim().is_empty() && message.chars().count() <= 1900,
        "Message must be 1–1900 characters"
    );
    ctx.defer_ephemeral().await?;
    require_channel_permissions(
        ctx,
        &channel,
        serenity::Permissions::SEND_MESSAGES,
        serenity::Permissions::SEND_MESSAGES,
    )
    .await?;
    channel
        .id
        .send_message(
            ctx.http(),
            serenity::CreateMessage::new()
                .content(message)
                .allowed_mentions(serenity::CreateAllowedMentions::new()),
        )
        .await?;
    reply(ctx, "Announcement sent.").await
}
/// Set a channel's slowmode in seconds.
#[poise::command(
    slash_command,
    prefix_command,
    guild_only,
    required_permissions = "MANAGE_CHANNELS",
    required_bot_permissions = "MANAGE_CHANNELS"
)]
async fn slowmode(
    ctx: Context<'_>,
    channel: serenity::GuildChannel,
    #[min = 0]
    #[max = 21600]
    seconds: u16,
) -> Result<()> {
    ensure!(seconds <= 21600, "Slowmode must be at most 21600 seconds");
    require_channel_permissions(
        ctx,
        &channel,
        serenity::Permissions::MANAGE_CHANNELS,
        serenity::Permissions::MANAGE_CHANNELS,
    )
    .await?;
    ensure!(
        channel.guild_id == guild_id(ctx)?,
        "Choose a channel in this server"
    );
    channel
        .id
        .edit(
            ctx.http(),
            serenity::EditChannel::new()
                .rate_limit_per_user(seconds)
                .audit_log_reason("Momo slowmode command"),
        )
        .await?;
    reply(ctx, "Slowmode updated.").await
}
/// Set or clear a member nickname.
#[poise::command(
    slash_command,
    prefix_command,
    guild_only,
    required_permissions = "MANAGE_NICKNAMES",
    required_bot_permissions = "MANAGE_NICKNAMES"
)]
async fn nick(ctx: Context<'_>, user: serenity::User, nickname: Option<String>) -> Result<()> {
    ctx.defer_ephemeral().await?;
    require_hierarchy(ctx, user.id, None).await?;
    let nickname = nickname.unwrap_or_default();
    ensure!(nickname.chars().count() <= 32, "Nickname is too long");
    guild_id(ctx)?
        .edit_member(
            ctx.http(),
            user.id,
            serenity::EditMember::new()
                .nickname(nickname)
                .audit_log_reason("Momo nickname command"),
        )
        .await?;
    reply(ctx, "Nickname updated.").await
}
/// Add a role to a member.
#[poise::command(
    slash_command,
    prefix_command,
    guild_only,
    required_permissions = "MANAGE_ROLES",
    required_bot_permissions = "MANAGE_ROLES",
    rename = "add"
)]
async fn role_add(ctx: Context<'_>, user: serenity::User, role: serenity::Role) -> Result<()> {
    ctx.defer_ephemeral().await?;
    require_hierarchy(ctx, user.id, None).await?;
    require_hierarchy(ctx, user.id, Some(role.id)).await?;
    guild_id(ctx)?
        .member(ctx.http(), user.id)
        .await?
        .add_role(ctx.http(), role.id)
        .await?;
    reply(ctx, "Role added.").await
}
/// Remove a role from a member.
#[poise::command(
    slash_command,
    prefix_command,
    guild_only,
    required_permissions = "MANAGE_ROLES",
    required_bot_permissions = "MANAGE_ROLES",
    rename = "remove"
)]
async fn role_remove(ctx: Context<'_>, user: serenity::User, role: serenity::Role) -> Result<()> {
    ctx.defer_ephemeral().await?;
    require_hierarchy(ctx, user.id, None).await?;
    require_hierarchy(ctx, user.id, Some(role.id)).await?;
    guild_id(ctx)?
        .member(ctx.http(), user.id)
        .await?
        .remove_role(ctx.http(), role.id)
        .await?;
    reply(ctx, "Role removed.").await
}
