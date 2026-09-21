#![forbid(unsafe_code)]
use anyhow::Result;
use bot_core::{serenity, *};

mod info;

async fn send_info(ctx: Context<'_>, embed: serenity::CreateEmbed) -> Result<()> {
    ctx.send(
        poise::CreateReply::default()
            .embed(embed)
            .ephemeral(true)
            .allowed_mentions(serenity::CreateAllowedMentions::new()),
    )
    .await?;
    Ok(())
}

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
    ctx.defer_ephemeral().await?;

    let guild_id = ctx.guild_id();
    let member = if let Some(guild_id) = guild_id {
        guild_id.member(ctx.http(), user.id).await.ok()
    } else {
        None
    };
    let guild = ctx.partial_guild().await;

    let display_name = member
        .as_ref()
        .map(|member| member.display_name())
        .unwrap_or_else(|| user.display_name());
    let username = if user.discriminator.is_some() {
        user.tag()
    } else {
        format!("@{}", user.name)
    };
    let avatar = member
        .as_ref()
        .map(serenity::Member::face)
        .unwrap_or_else(|| user.face());

    let account_type = if user.bot { "Bot" } else { "Human" };
    let account = format!(
        "**Username**\n{username}\n\n**Account created**\n{}\n\n**Account type**\n{account_type}",
        info::discord_date(user.id.created_at())
    );

    let mut embed = info::embed(format!("User Info • {display_name}"))
        .description(format!("Profile for {username}"))
        .thumbnail(avatar)
        .field("Account", account, false);

    if guild_id.is_some() {
        let (server_details, roles) = if let (Some(member), Some(guild)) = (&member, &guild) {
            let mut statuses = vec!["Member"];
            if member.user.id == guild.owner_id {
                statuses.push("Server Owner");
            }
            if guild.member_permissions(member).administrator() {
                statuses.push("Administrator");
            }

            let mut roles = member
                .roles
                .iter()
                .filter(|role_id| role_id.get() != guild.id.get())
                .filter_map(|role_id| guild.roles.get(role_id))
                .collect::<Vec<_>>();
            roles.sort_by(|left, right| right.cmp(left));
            let highest = roles
                .first()
                .map(|role| role.name.as_str())
                .unwrap_or("None");
            let nickname = member.nick.as_deref().unwrap_or("None");
            let joined = member
                .joined_at
                .map(info::discord_date)
                .unwrap_or_else(|| "Not available".to_string());
            let details = format!(
                "**Nickname**\n{nickname}\n\n**Joined server**\n{joined}\n\n**Highest role**\n{highest}\n\n**Status**\n{}",
                statuses.join(" • ")
            );
            let names = roles
                .iter()
                .map(|role| role.name.clone())
                .collect::<Vec<_>>();
            (details, info::role_list(&names))
        } else {
            (
                "**Nickname**\nNone\n\n**Joined server**\nNot available\n\n**Highest role**\nNot available\n\n**Status**\nNot available"
                    .to_string(),
                "Not available".to_string(),
            )
        };

        embed = embed
            .field("Server", server_details, false)
            .field("Roles", roles, false);
    }

    embed = embed.field("Technical", format!("**User ID**\n{}", user.id), false);
    send_info(ctx, embed).await
}
/// Server information.
#[poise::command(slash_command, prefix_command, guild_only, subcommands("server_info"))]
async fn server(ctx: Context<'_>) -> Result<()> {
    reply(ctx, "Use /server info.").await
}
/// Show the server name and owner.
#[poise::command(slash_command, prefix_command, guild_only, rename = "info")]
async fn server_info(ctx: Context<'_>) -> Result<()> {
    let guild_id = guild_id(ctx)?;
    let cached_counts = ctx
        .guild()
        .map(|guild| (guild.member_count, guild.channels.len()));
    ctx.defer_ephemeral().await?;

    let guild = guild_id.to_partial_guild(ctx.http()).await?;
    let owner_member = guild_id.member(ctx.http(), guild.owner_id).await.ok();
    let owner_user = if owner_member.is_none() {
        guild.owner_id.to_user(ctx.http()).await.ok()
    } else {
        None
    };
    let owner_name = owner_member
        .as_ref()
        .map(|member| member.display_name())
        .or_else(|| owner_user.as_ref().map(serenity::User::display_name))
        .unwrap_or("Unknown");
    let owner_avatar = owner_member
        .as_ref()
        .map(serenity::Member::face)
        .or_else(|| owner_user.as_ref().map(serenity::User::face));

    let member_count = cached_counts
        .map(|(members, _)| members)
        .or(guild.approximate_member_count)
        .map(|count| count.to_string())
        .unwrap_or_else(|| "Not available".to_string());
    let channel_count = if let Some((_, channels)) = cached_counts {
        Some(channels)
    } else {
        guild
            .channels(ctx.http())
            .await
            .ok()
            .map(|channels| channels.len())
    };
    let boost_level = match guild.premium_tier {
        serenity::PremiumTier::Tier0 => "Level 0",
        serenity::PremiumTier::Tier1 => "Level 1",
        serenity::PremiumTier::Tier2 => "Level 2",
        serenity::PremiumTier::Tier3 => "Level 3",
        _ => "Unknown",
    };
    let boost_count = guild
        .premium_subscription_count
        .map(|count| count.to_string())
        .unwrap_or_else(|| "Not available".to_string());

    let overview = format!(
        "**Owner**\n{owner_name}\n\n**Created**\n{}\n\n**Members**\n{member_count}",
        info::discord_date(guild.id.created_at())
    );
    let server = format!(
        "**Channels**\n{}\n\n**Roles**\n{}\n\n**Boost level**\n{boost_level}\n\n**Boosts**\n{boost_count}",
        channel_count
            .map(|count| count.to_string())
            .unwrap_or_else(|| "Not available".to_string()),
        guild.roles.len()
    );

    let mut embed = info::embed(format!("Server Info • {}", guild.name))
        .description(format!("Information about {}", guild.name))
        .field("Overview", overview, true)
        .field("Server", server, true)
        .field("Technical", format!("**Server ID**\n{}", guild.id), false);
    if let Some(icon) = guild.icon_url() {
        embed = embed.thumbnail(icon);
    }
    let mut author = serenity::CreateEmbedAuthor::new(format!("Owned by {owner_name}"));
    if let Some(avatar) = owner_avatar {
        author = author.icon_url(avatar);
    }
    embed = embed.author(author);

    send_info(ctx, embed).await
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
