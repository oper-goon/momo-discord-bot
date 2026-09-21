#![forbid(unsafe_code)]
use anyhow::Result;
use bot_core::{serenity, *};
pub fn module() -> RegisteredModule {
    let mut module = RegisteredModule::new(
        "general",
        "General",
        false,
        true,
        vec![ping(), avatar(), user(), server()],
    );
    module.extensions.push(("role", role_info()));
    module
}
/// Check whether the bot is responding.
#[poise::command(slash_command, prefix_command)]
async fn ping(ctx: Context<'_>) -> Result<()> {
    reply(ctx, "Pong!").await
}
/// Show a user's avatar.
#[poise::command(slash_command, prefix_command)]
async fn avatar(ctx: Context<'_>, user: Option<serenity::User>) -> Result<()> {
    let user = user.unwrap_or_else(|| ctx.author().clone());
    reply(ctx, user.face()).await
}
/// User information.
#[poise::command(slash_command, prefix_command, subcommands("user_info"))]
async fn user(ctx: Context<'_>) -> Result<()> {
    reply(ctx, "Use /user info.").await
}
/// Show a user's profile and account creation time.
#[poise::command(slash_command, prefix_command, rename = "info")]
async fn user_info(ctx: Context<'_>, user: Option<serenity::User>) -> Result<()> {
    let user = user.unwrap_or_else(|| ctx.author().clone());
    reply(
        ctx,
        format!(
            "{} • {}\nCreated: {}\nBot: {}",
            user.name,
            user.id,
            user.id.created_at(),
            user.bot
        ),
    )
    .await
}
/// Server information.
#[poise::command(slash_command, prefix_command, guild_only, subcommands("server_info"))]
async fn server(ctx: Context<'_>) -> Result<()> {
    reply(ctx, "Use /server info.").await
}
/// Show the server name and owner.
#[poise::command(slash_command, prefix_command, guild_only, rename = "info")]
async fn server_info(ctx: Context<'_>) -> Result<()> {
    let guild = guild_id(ctx)?.to_partial_guild(ctx.http()).await?;
    reply(
        ctx,
        format!(
            "{} • {}\nOwner: {}\nRoles: {}",
            guild.name,
            guild.id,
            guild.owner_id,
            guild.roles.len()
        ),
    )
    .await
}
/// Inspect a role.
#[poise::command(slash_command, prefix_command, guild_only, rename = "info")]
async fn role_info(ctx: Context<'_>, role: serenity::Role) -> Result<()> {
    reply(
        ctx,
        format!(
            "{} • {}\nPosition: {}\nPermissions: {:?}",
            role.name, role.id, role.position, role.permissions
        ),
    )
    .await
}
