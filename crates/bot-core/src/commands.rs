use crate::*;
use anyhow::{Result, ensure};

pub fn module() -> RegisteredModule {
    RegisteredModule::new(
        "core",
        "Core",
        true,
        true,
        vec![
            help(),
            about(),
            modules(),
            permissions(),
            prefix(),
            config(),
            role(),
        ],
    )
}

/// Inspect or manage roles.
#[poise::command(slash_command, prefix_command, guild_only)]
async fn role(ctx: Context<'_>) -> Result<()> {
    reply(ctx, "Use /role info, add, or remove.").await
}

/// List available commands, or show a command path such as queue move.
#[poise::command(slash_command, prefix_command)]
pub async fn help(ctx: Context<'_>, #[rest] command: Option<String>) -> Result<()> {
    ctx.defer_ephemeral().await?;
    let wanted = command.map(|s| s.replace(' ', "."));
    let effective = effective_permissions(ctx).await?;
    let roles = if let Some(guild) = ctx.guild_id() {
        guild
            .member(ctx.http(), ctx.author().id)
            .await?
            .roles
            .iter()
            .map(|r| r.get())
            .collect::<Vec<_>>()
    } else {
        vec![]
    };
    let rules = if let Some(guild) = ctx.guild_id() {
        ctx.data().database.rules(guild.get()).await?
    } else {
        vec![]
    };
    let subject = permissions::Subject {
        user: ctx.author().id.get(),
        channel: ctx.channel_id().get(),
        roles: &roles,
        owner: ctx
            .data()
            .config
            .discord
            .owners
            .contains(&ctx.author().id.get()),
        administrator: effective.is_some_and(|p| p.administrator()),
    };
    let mut lines = vec![];
    fn flatten<'a>(commands: &'a [Command], out: &mut Vec<&'a Command>) {
        for c in commands {
            out.push(c);
            flatten(&c.subcommands, out);
        }
    }
    let mut all = vec![];
    flatten(&ctx.framework().options().commands, &mut all);
    for c in all {
        let path = c.qualified_name.replace(' ', ".");
        if wanted.as_ref().is_some_and(|w| w != &path) {
            continue;
        }
        let owner = ctx.data().modules.owner(&path)?;
        if crate::channel_policy::blocked_channels(ctx, owner)
            .await?
            .is_some()
        {
            continue;
        }
        if c.guild_only && ctx.guild_id().is_none() {
            continue;
        }
        if c.owners_only
            && !ctx
                .data()
                .config
                .discord
                .owners
                .contains(&ctx.author().id.get())
        {
            continue;
        }
        if !c.required_permissions.is_empty()
            && !effective.is_some_and(|p| p.contains(c.required_permissions))
        {
            continue;
        }
        if let Some(guild) = ctx.guild_id()
            && !ctx
                .data()
                .modules
                .enabled(&ctx.data().database, guild.get(), owner)
                .await?
        {
            continue;
        }
        if !permissions::evaluate(
            &rules,
            &subject,
            owner,
            &path,
            ctx.data().config.discord.administrator_bypass,
            true,
        )
        .allowed
        {
            continue;
        }
        lines.push(format!(
            "`/{}` — {} [{}]",
            c.qualified_name,
            c.description.as_deref().unwrap_or("Command group"),
            owner
        ));
    }
    lines.sort();
    reply(
        ctx,
        if lines.is_empty() {
            "No accessible command matches.".into()
        } else {
            lines.join("\n")
        },
    )
    .await
}
/// Version and service health.
#[poise::command(slash_command, prefix_command)]
async fn about(ctx: Context<'_>) -> Result<()> {
    ctx.defer_ephemeral().await?;
    let version = ctx.data().database.health().await?;
    let mut lines = vec![format!(
        "Momo v{} • SQLite migration {} • task failures {}",
        env!("CARGO_PKG_VERSION"),
        version,
        ctx.data().supervisor.failures()
    )];
    for m in &ctx.data().modules.modules {
        lines.push(format!(
            "{}: {}",
            m.descriptor.display_name,
            m.lifecycle
                .health()
                .await
                .unwrap_or_else(|_| "degraded".into())
        ));
    }
    reply(ctx, lines.join("\n")).await
}

/// Manage server modules.
#[poise::command(
    slash_command,
    prefix_command,
    guild_only,
    rename = "module",
    subcommands(
        "module_list",
        "module_info",
        "module_enable",
        "module_disable",
        "module_channels"
    )
)]
async fn modules(ctx: Context<'_>) -> Result<()> {
    reply(ctx, "Use /module list, info, enable, disable, or channels.").await
}

/// Manage module channel allowlists (slash-only configuration).
#[poise::command(
    slash_command,
    guild_only,
    rename = "channels",
    required_permissions = "MANAGE_GUILD",
    subcommands(
        "module_channels_add",
        "module_channels_remove",
        "module_channels_list",
        "module_channels_reset"
    )
)]
async fn module_channels(ctx: Context<'_>) -> Result<()> {
    reply(ctx, "Use /module channels add, remove, list, or reset.").await
}

fn validate_channel_module(ctx: Context<'_>, module: &str) -> Result<()> {
    ctx.data().modules.module(module)?;
    ensure!(
        module != "core",
        "Core commands remain available in every channel"
    );
    Ok(())
}

/// Allow a module in a channel; the first entry restricts use elsewhere.
#[poise::command(
    slash_command,
    guild_only,
    rename = "add",
    required_permissions = "MANAGE_GUILD"
)]
async fn module_channels_add(
    ctx: Context<'_>,
    module: String,
    channel: serenity::GuildChannel,
) -> Result<()> {
    validate_channel_module(ctx, &module)?;
    ensure!(
        channel.guild_id == guild_id(ctx)?,
        "Choose a channel in this server"
    );
    ensure!(
        channel.kind != serenity::ChannelType::Category,
        "Choose a channel, not a category"
    );
    ctx.data()
        .database
        .add_module_channel(guild_id(ctx)?.get(), &module, channel.id.get())
        .await?;
    reply(
        ctx,
        format!(
            "{module} is allowed in <#{}>. Administrators always bypass channel limits.",
            channel.id
        ),
    )
    .await
}

/// Remove an allowed channel; the last removal restores access everywhere.
#[poise::command(
    slash_command,
    guild_only,
    rename = "remove",
    required_permissions = "MANAGE_GUILD"
)]
async fn module_channels_remove(
    ctx: Context<'_>,
    module: String,
    channel: serenity::GuildChannel,
) -> Result<()> {
    validate_channel_module(ctx, &module)?;
    ensure!(
        channel.guild_id == guild_id(ctx)?,
        "Choose a channel in this server"
    );
    ctx.data()
        .database
        .remove_module_channel(guild_id(ctx)?.get(), &module, channel.id.get())
        .await?;
    reply_channel_list(ctx, &module).await
}

async fn reply_channel_list(ctx: Context<'_>, module: &str) -> Result<()> {
    ctx.data().modules.module(module)?;
    let channels = ctx
        .data()
        .database
        .module_channels(guild_id(ctx)?.get(), module)
        .await?;
    let status = if channels.is_empty() {
        "all channels".to_owned()
    } else {
        channels
            .iter()
            .map(|id| format!("<#{id}>"))
            .collect::<Vec<_>>()
            .join(", ")
    };
    reply(
        ctx,
        format!("{module}: {status}. Administrators always bypass channel limits."),
    )
    .await
}

/// Show a module's allowed channels.
#[poise::command(
    slash_command,
    guild_only,
    rename = "list",
    required_permissions = "MANAGE_GUILD"
)]
async fn module_channels_list(ctx: Context<'_>, module: String) -> Result<()> {
    reply_channel_list(ctx, &module).await
}

/// Restore a module's access to all channels.
#[poise::command(
    slash_command,
    guild_only,
    rename = "reset",
    required_permissions = "MANAGE_GUILD"
)]
async fn module_channels_reset(ctx: Context<'_>, module: String) -> Result<()> {
    validate_channel_module(ctx, &module)?;
    ctx.data()
        .database
        .reset_module_channels(guild_id(ctx)?.get(), &module)
        .await?;
    reply(
        ctx,
        format!("{module} commands are allowed in all channels."),
    )
    .await
}
/// List module activation.
#[poise::command(slash_command, prefix_command, guild_only, rename = "list")]
async fn module_list(ctx: Context<'_>) -> Result<()> {
    ctx.defer_ephemeral().await?;
    let guild = guild_id(ctx)?.get();
    let mut lines = vec![];
    for m in &ctx.data().modules.modules {
        lines.push(format!(
            "{} — {}{}",
            m.descriptor.id,
            if ctx
                .data()
                .modules
                .enabled(&ctx.data().database, guild, m.descriptor.id)
                .await?
            {
                "enabled"
            } else {
                "disabled"
            },
            if m.descriptor.required {
                " (required)"
            } else {
                ""
            }
        ));
    }
    reply(ctx, lines.join("\n")).await
}
/// Inspect a compiled module.
#[poise::command(slash_command, prefix_command, guild_only, rename = "info")]
async fn module_info(ctx: Context<'_>, module: String) -> Result<()> {
    let m = ctx.data().modules.module(&module)?;
    reply(
        ctx,
        format!(
            "{} v{}\n{}",
            m.descriptor.display_name,
            m.descriptor.version,
            m.lifecycle.health().await?
        ),
    )
    .await
}
/// Enable a module for this server.
#[poise::command(
    slash_command,
    prefix_command,
    guild_only,
    required_permissions = "MANAGE_GUILD",
    rename = "enable"
)]
async fn module_enable(ctx: Context<'_>, module: String) -> Result<()> {
    ctx.defer_ephemeral().await?;
    ctx.data()
        .modules
        .set_enabled(
            ctx.data(),
            guild_id(ctx)?.get(),
            &module,
            true,
            ctx.author().id.get(),
        )
        .await?;
    reply(ctx, "Module enabled.").await
}
/// Disable a module and stop its server services.
#[poise::command(
    slash_command,
    prefix_command,
    guild_only,
    required_permissions = "MANAGE_GUILD",
    rename = "disable"
)]
async fn module_disable(ctx: Context<'_>, module: String) -> Result<()> {
    ctx.defer_ephemeral().await?;
    ctx.data()
        .modules
        .set_enabled(
            ctx.data(),
            guild_id(ctx)?.get(),
            &module,
            false,
            ctx.author().id.get(),
        )
        .await?;
    reply(ctx, "Module disabled.").await
}

/// Manage command authorization rules.
#[poise::command(
    slash_command,
    prefix_command,
    guild_only,
    required_permissions = "MANAGE_GUILD",
    subcommands(
        "permission_list",
        "permission_set",
        "permission_reset",
        "permission_explain"
    )
)]
async fn permissions(ctx: Context<'_>) -> Result<()> {
    reply(ctx, "Use /permissions list, set, reset, or explain.").await
}
/// List server authorization rules.
#[poise::command(
    slash_command,
    prefix_command,
    guild_only,
    required_permissions = "MANAGE_GUILD",
    rename = "list"
)]
async fn permission_list(ctx: Context<'_>, target: Option<String>) -> Result<()> {
    let rules = ctx.data().database.rules(guild_id(ctx)?.get()).await?;
    let lines: Vec<_> = rules
        .iter()
        .filter(|r| target.as_ref().is_none_or(|t| t == &r.target))
        .map(|r| {
            format!(
                "#{} {} {:?} {} {} channel {}",
                r.id,
                r.target,
                r.effect,
                r.subject_kind,
                r.subject_id.as_deref().unwrap_or("everyone"),
                r.channel_id.as_deref().unwrap_or("all")
            )
        })
        .collect();
    reply(
        ctx,
        if lines.is_empty() {
            "No rules configured.".into()
        } else {
            lines.join("\n")
        },
    )
    .await
}
#[derive(poise::ChoiceParameter)]
enum PolicyEffect {
    Allow,
    Deny,
}
fn parse_subject(subject: &str) -> Result<(String, Option<String>)> {
    if subject == "everyone" {
        return Ok((subject.into(), None));
    }
    let (kind, id) = subject
        .split_once(':')
        .ok_or_else(|| anyhow::anyhow!("Use everyone, user:ID, role:ID, or channel:ID"))?;
    ensure!(
        ["user", "role", "channel"].contains(&kind),
        "Invalid subject kind"
    );
    let id = id.parse::<u64>()?;
    ensure!(id > 0, "Invalid subject ID");
    Ok((kind.into(), Some(id.to_string())))
}
fn validate_target(ctx: Context<'_>, target: &str) -> Result<()> {
    ensure!(
        ctx.data().modules.module(target).is_ok() || ctx.data().modules.owner(target).is_ok(),
        "Unknown module or command path"
    );
    Ok(())
}
/// Set a rule; subject is everyone, user:ID, role:ID, or channel:ID.
#[poise::command(
    slash_command,
    prefix_command,
    guild_only,
    required_permissions = "MANAGE_GUILD",
    rename = "set"
)]
async fn permission_set(
    ctx: Context<'_>,
    target: String,
    effect: PolicyEffect,
    subject: String,
    channel: Option<serenity::Channel>,
) -> Result<()> {
    validate_target(ctx, &target)?;
    let (kind, id) = parse_subject(&subject)?;
    ctx.data()
        .database
        .set_rule(
            guild_id(ctx)?.get(),
            &PermissionRule {
                id: 0,
                target,
                subject_kind: kind,
                subject_id: id,
                channel_id: channel.map(|c| c.id().to_string()),
                effect: match effect {
                    PolicyEffect::Allow => Effect::Allow,
                    PolicyEffect::Deny => Effect::Deny,
                },
            },
            ctx.author().id.get(),
            false,
        )
        .await?;
    reply(ctx, "Rule saved.").await
}
/// Remove a matching rule.
#[poise::command(
    slash_command,
    prefix_command,
    guild_only,
    required_permissions = "MANAGE_GUILD",
    rename = "reset"
)]
async fn permission_reset(
    ctx: Context<'_>,
    target: String,
    subject: String,
    channel: Option<serenity::Channel>,
) -> Result<()> {
    let (kind, id) = parse_subject(&subject)?;
    ctx.data()
        .database
        .set_rule(
            guild_id(ctx)?.get(),
            &PermissionRule {
                id: 0,
                target,
                subject_kind: kind,
                subject_id: id,
                channel_id: channel.map(|c| c.id().to_string()),
                effect: Effect::Deny,
            },
            ctx.author().id.get(),
            true,
        )
        .await?;
    reply(ctx, "Rule removed.").await
}
/// Explain the winning custom rule for a command.
#[poise::command(
    slash_command,
    prefix_command,
    guild_only,
    required_permissions = "MANAGE_GUILD",
    rename = "explain"
)]
async fn permission_explain(
    ctx: Context<'_>,
    command: String,
    user: Option<serenity::User>,
) -> Result<()> {
    let command = command.replace(' ', ".");
    let module = ctx.data().modules.owner(&command)?;
    let guild = guild_id(ctx)?;
    let user = user.unwrap_or_else(|| ctx.author().clone());
    let member = guild.member(ctx.http(), user.id).await?;
    let roles = member.roles.iter().map(|r| r.get()).collect::<Vec<_>>();
    let partial = guild.to_partial_guild(ctx.http()).await?;
    let admin = user.id == partial.owner_id
        || partial.roles.iter().any(|(id, r)| {
            (member.roles.contains(id) || id.get() == guild.get()) && r.permissions.administrator()
        });
    let result = permissions::evaluate(
        &ctx.data().database.rules(guild.get()).await?,
        &permissions::Subject {
            user: user.id.get(),
            channel: ctx.channel_id().get(),
            roles: &roles,
            owner: ctx.data().config.discord.owners.contains(&user.id.get()),
            administrator: admin,
        },
        module,
        &command,
        ctx.data().config.discord.administrator_bypass,
        true,
    );
    reply(ctx,format!("Custom policy: {} — {} (rule {:?}). Discord permissions and module activation also apply.",if result.allowed{"allow"}else{"deny"},result.reason,result.rule_id)).await
}

/// Configure optional prefix commands.
#[poise::command(
    slash_command,
    prefix_command,
    guild_only,
    subcommands("prefix_show", "prefix_set", "prefix_enable", "prefix_disable")
)]
async fn prefix(ctx: Context<'_>) -> Result<()> {
    reply(ctx, "Use /prefix show, set, enable, or disable.").await
}
/// Show the server prefix.
#[poise::command(slash_command, prefix_command, guild_only, rename = "show")]
async fn prefix_show(ctx: Context<'_>) -> Result<()> {
    let (p, e) = ctx
        .data()
        .database
        .prefix(guild_id(ctx)?.get())
        .await?
        .unwrap_or((
            ctx.data().config.discord.default_prefix.clone(),
            ctx.data().config.discord.prefix_commands,
        ));
    reply(
        ctx,
        format!(
            "Prefix: `{p}` • server enabled: {e} • process enabled: {}",
            ctx.data().config.discord.prefix_commands
        ),
    )
    .await
}
/// Set the server prefix, up to 16 characters.
#[poise::command(
    slash_command,
    prefix_command,
    guild_only,
    required_permissions = "MANAGE_GUILD",
    rename = "set"
)]
async fn prefix_set(ctx: Context<'_>, value: String) -> Result<()> {
    let guild = guild_id(ctx)?.get();
    let enabled = ctx
        .data()
        .database
        .prefix(guild)
        .await?
        .map(|(_, e)| e)
        .unwrap_or(true);
    ctx.data()
        .database
        .set_prefix(guild, &value, enabled)
        .await?;
    reply(ctx, "Prefix saved.").await
}
async fn toggle_prefix(ctx: Context<'_>, enabled: bool) -> Result<()> {
    let guild = guild_id(ctx)?.get();
    let p = ctx
        .data()
        .database
        .prefix(guild)
        .await?
        .map(|(p, _)| p)
        .unwrap_or_else(|| ctx.data().config.discord.default_prefix.clone());
    ctx.data().database.set_prefix(guild, &p, enabled).await?;
    reply(
        ctx,
        "Prefix setting saved. The process must also allow prefix commands.",
    )
    .await
}
/// Enable server prefix commands.
#[poise::command(
    slash_command,
    prefix_command,
    guild_only,
    required_permissions = "MANAGE_GUILD",
    rename = "enable"
)]
async fn prefix_enable(ctx: Context<'_>) -> Result<()> {
    toggle_prefix(ctx, true).await
}
/// Disable server prefix commands.
#[poise::command(
    slash_command,
    prefix_command,
    guild_only,
    required_permissions = "MANAGE_GUILD",
    rename = "disable"
)]
async fn prefix_disable(ctx: Context<'_>) -> Result<()> {
    toggle_prefix(ctx, false).await
}

/// Configure typed server music settings.
#[poise::command(
    slash_command,
    prefix_command,
    guild_only,
    required_permissions = "MANAGE_GUILD",
    subcommands("config_show", "config_set", "config_reset")
)]
async fn config(ctx: Context<'_>) -> Result<()> {
    reply(ctx, "Use /config show, set, or reset.").await
}
/// Show effective music settings.
#[poise::command(
    slash_command,
    prefix_command,
    guild_only,
    required_permissions = "MANAGE_GUILD",
    rename = "show"
)]
async fn config_show(ctx: Context<'_>) -> Result<()> {
    let settings = ctx
        .data()
        .database
        .music_settings(guild_id(ctx)?.get())
        .await?
        .unwrap_or_else(|| ctx.data().config.music_defaults());
    reply(
        ctx,
        format!("```json\n{}\n```", serde_json::to_string_pretty(&settings)?),
    )
    .await
}
/// Set a music key: volume, queue_limit, idle_seconds, empty_seconds, dj_role, announce.
#[poise::command(
    slash_command,
    prefix_command,
    guild_only,
    required_permissions = "MANAGE_GUILD",
    rename = "set"
)]
async fn config_set(ctx: Context<'_>, key: String, value: String) -> Result<()> {
    let guild = guild_id(ctx)?.get();
    let mut settings = ctx
        .data()
        .database
        .music_settings(guild)
        .await?
        .unwrap_or_else(|| ctx.data().config.music_defaults());
    match key.as_str() {
        "volume" => settings.default_volume_percent = value.parse()?,
        "queue_limit" => settings.max_queue_length = value.parse()?,
        "idle_seconds" => settings.idle_disconnect_seconds = value.parse()?,
        "empty_seconds" => settings.empty_channel_disconnect_seconds = value.parse()?,
        "dj_role" => {
            settings.dj_role_id = if value == "none" {
                None
            } else {
                Some(value.parse::<u64>()?.to_string())
            }
        }
        "announce" => settings.announce_now_playing = value.parse()?,
        _ => anyhow::bail!("Unknown configuration key"),
    };
    ctx.data()
        .database
        .save_music_settings(guild, &settings, ctx.data().config.music.max_queue_length)
        .await?;
    ctx.data()
        .modules
        .module("music")?
        .lifecycle
        .config_changed(ctx.data(), guild)
        .await?;
    reply(ctx, "Configuration saved.").await
}
/// Reset music settings to process defaults.
#[poise::command(
    slash_command,
    prefix_command,
    guild_only,
    required_permissions = "MANAGE_GUILD",
    rename = "reset"
)]
async fn config_reset(ctx: Context<'_>) -> Result<()> {
    let guild = guild_id(ctx)?.get();
    ctx.data()
        .database
        .save_music_settings(
            guild,
            &ctx.data().config.music_defaults(),
            ctx.data().config.music.max_queue_length,
        )
        .await?;
    ctx.data()
        .modules
        .module("music")?
        .lifecycle
        .config_changed(ctx.data(), guild)
        .await?;
    reply(ctx, "Configuration reset.").await
}
