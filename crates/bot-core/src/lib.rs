#![forbid(unsafe_code)]

pub mod channel_policy;
pub mod commands;
pub mod config;
pub mod permissions;
pub mod runtime;
pub use bot_database::{Database, now_ms};
pub use bot_module_api::*;
pub use config::AppConfig;
pub use poise::serenity_prelude as serenity;
pub use runtime::*;
pub type BotError = anyhow::Error;
pub type Context<'a> = poise::Context<'a, BotData, BotError>;
pub type Command = poise::Command<BotData, BotError>;

pub async fn reply(ctx: Context<'_>, message: impl Into<String>) -> anyhow::Result<()> {
    let message: String = message.into();
    let safe: String = message.chars().take(1900).collect();
    ctx.send(
        poise::CreateReply::default()
            .content(safe)
            .ephemeral(true)
            .allowed_mentions(serenity::CreateAllowedMentions::new()),
    )
    .await?;
    Ok(())
}
pub fn guild_id(ctx: Context<'_>) -> anyhow::Result<serenity::GuildId> {
    ctx.guild_id()
        .ok_or_else(|| anyhow::anyhow!("Use this command in a server"))
}

pub async fn effective_permissions(
    ctx: Context<'_>,
) -> anyhow::Result<Option<serenity::Permissions>> {
    let Some(guild) = ctx.guild_id() else {
        return Ok(None);
    };
    let guild = guild.to_partial_guild(ctx.http()).await?;
    let member = guild.id.member(ctx.http(), ctx.author().id).await?;
    let channel = ctx
        .channel_id()
        .to_channel(ctx.http())
        .await?
        .guild()
        .ok_or_else(|| anyhow::anyhow!("Server channel required"))?;
    Ok(Some(guild.user_permissions_in(&channel, &member)))
}

pub async fn require_channel_permissions(
    ctx: Context<'_>,
    channel: &serenity::GuildChannel,
    actor: serenity::Permissions,
    bot: serenity::Permissions,
) -> anyhow::Result<()> {
    anyhow::ensure!(
        Some(channel.guild_id) == ctx.guild_id(),
        "Choose a channel in this server"
    );
    let guild = channel.guild_id.to_partial_guild(ctx.http()).await?;
    let member = guild.id.member(ctx.http(), ctx.author().id).await?;
    let bot_id = ctx.serenity_context().cache.current_user().id;
    let bot_member = guild.id.member(ctx.http(), bot_id).await?;
    anyhow::ensure!(
        guild.user_permissions_in(channel, &member).contains(actor),
        "You lack the required permission in the target channel"
    );
    anyhow::ensure!(
        guild
            .user_permissions_in(channel, &bot_member)
            .contains(bot),
        "The bot lacks the required permission in the target channel"
    );
    Ok(())
}

pub async fn require_hierarchy(
    ctx: Context<'_>,
    target: serenity::UserId,
    role: Option<serenity::RoleId>,
) -> anyhow::Result<()> {
    let guild_id = guild_id(ctx)?;
    let guild = guild_id.to_partial_guild(ctx.http()).await?;
    let bot_id = ctx.serenity_context().cache.current_user().id;
    anyhow::ensure!(
        target != guild.owner_id && target != bot_id && target != ctx.author().id,
        "Cannot target the server owner, yourself, or this bot"
    );
    let actor = guild_id.member(ctx.http(), ctx.author().id).await?;
    let bot = guild_id.member(ctx.http(), bot_id).await?;
    let member = guild_id.member(ctx.http(), target).await?;
    let highest = |m: &serenity::Member| {
        m.roles
            .iter()
            .filter_map(|id| guild.roles.get(id))
            .max()
            .cloned()
    };
    let target_role = if let Some(id) = role {
        Some(
            guild
                .roles
                .get(&id)
                .ok_or_else(|| anyhow::anyhow!("Role not found"))?
                .clone(),
        )
    } else {
        highest(&member)
    };
    if let Some(role) = target_role {
        anyhow::ensure!(
            !role.managed && role.id.get() != guild_id.get(),
            "Managed and everyone roles cannot be modified"
        );
        anyhow::ensure!(
            highest(&bot).is_some_and(|r| r > role),
            "The bot role must be above the target role"
        );
        anyhow::ensure!(
            ctx.author().id == guild.owner_id || highest(&actor).is_some_and(|r| r > role),
            "Your role must be above the target role"
        );
    }
    Ok(())
}
