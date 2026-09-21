#![forbid(unsafe_code)]
use anyhow::{Context, Result};
use bot_core::{serenity, *};
use sha2::{Digest, Sha256};
use songbird::SerenityInit;
use std::{sync::Arc, time::Duration};
use tracing_subscriber::EnvFilter;

fn compose(music: Arc<module_music::MusicService>) -> Result<(ModuleRegistry, Vec<Command>)> {
    let mut registry = ModuleRegistry::new(vec![
        commands::module(),
        module_general::module(),
        module_admin::module(),
        module_moderation::module(),
        module_utility::module(),
        module_music::module(music),
        module_torbox::module(),
    ])?;
    let mut commands = Vec::new();
    for module in &mut registry.modules {
        commands.append(&mut module.commands);
    }
    for module in &mut registry.modules {
        for (parent, command) in module.extensions.drain(..) {
            commands
                .iter_mut()
                .find(|c| c.name == parent)
                .context("Missing command parent")?
                .subcommands
                .push(command);
        }
    }
    fn defaults(commands: &mut [Command]) -> Result<()> {
        for command in commands {
            command
                .cooldown_config
                .get_mut()
                .map_err(|_| anyhow::anyhow!("Cooldown configuration poisoned"))?
                .user
                .get_or_insert(Duration::from_secs(1));
            defaults(&mut command.subcommands)?;
        }
        Ok(())
    }
    defaults(&mut commands)?;
    Ok((registry, commands))
}

async fn synchronize(
    ctx: &serenity::Context,
    framework: &poise::Framework<BotData, BotError>,
    data: &BotData,
) -> Result<()> {
    let mode = &data.config.commands.mode;
    if mode == "none" {
        return Ok(());
    }
    let definitions = poise::builtins::create_application_commands(&framework.options().commands);
    let hash = format!("{:x}", Sha256::digest(serde_json::to_vec(&definitions)?));
    let force = std::env::args().any(|a| a == "--sync-commands");
    if mode == "global" {
        let scope = format!(
            "{}:global",
            ctx.http
                .application_id()
                .context("Application ID unavailable")?
        );
        if force || data.database.sync_hash(&scope).await?.as_deref() != Some(&hash) {
            poise::builtins::register_globally(ctx, &framework.options().commands).await?;
            data.database.set_sync_hash(&scope, &hash).await?;
        }
    } else {
        for guild in &data.config.commands.development_guilds {
            let scope = format!(
                "{}:guild:{guild}",
                ctx.http
                    .application_id()
                    .context("Application ID unavailable")?
            );
            if force || data.database.sync_hash(&scope).await?.as_deref() != Some(&hash) {
                poise::builtins::register_in_guild(
                    ctx,
                    &framework.options().commands,
                    serenity::GuildId::new(*guild),
                )
                .await?;
                data.database.set_sync_hash(&scope, &hash).await?;
            }
        }
    }
    Ok(())
}
#[tokio::main]
async fn main() -> Result<()> {
    let args: Vec<_> = std::env::args().collect();
    let config_path = args
        .windows(2)
        .find(|a| a[0] == "--config")
        .map(|a| a[1].as_str());
    let config = Arc::new(AppConfig::load(config_path)?);
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info,serenity=warn,songbird=warn,sqlx=warn"));
    if config.logging.format == "json" {
        tracing_subscriber::fmt()
            .with_env_filter(filter)
            .json()
            .init();
    } else {
        tracing_subscriber::fmt().with_env_filter(filter).init();
    }
    // Create the default data directory; custom database parents must be provisioned by the operator.
    std::fs::create_dir_all("data")?;
    let database = Database::open(&config.database.url, config.database.max_connections).await?;
    let manager = songbird::Songbird::serenity();
    let music = Arc::new(module_music::MusicService::new(&config, manager.clone()));
    let torbox = Arc::new(module_torbox::TorBoxService::from_env()?);
    let (registry, commands) = compose(music.clone())?;
    let mut services = ServiceRegistry::default();
    services.insert(music)?;
    services.insert(torbox)?;
    let data = BotData {
        config: config.clone(),
        database: database.clone(),
        modules: Arc::new(registry),
        services: Arc::new(services),
        supervisor: Arc::new(Supervisor::default()),
    };
    for module in &data.modules.modules {
        module.lifecycle.init(&data).await?;
    }
    if args.iter().any(|a| a == "--check") {
        tracing::info!(
            modules = data.modules.modules.len(),
            commands = commands.len(),
            migration = database.health().await?,
            "Configuration, migrations, and command registry validated"
        );
        database.close().await;
        return Ok(());
    }
    let token = std::env::var("DISCORD_TOKEN")
        .context("Set DISCORD_TOKEN in the environment before starting the bot")?;
    let mut intents = serenity::GatewayIntents::GUILDS;
    for module in &data.modules.modules {
        intents |= module.intents;
    }
    if config.discord.prefix_commands {
        intents |= serenity::GatewayIntents::GUILD_MESSAGES
            | serenity::GatewayIntents::DIRECT_MESSAGES
            | serenity::GatewayIntents::MESSAGE_CONTENT;
    }
    let startup_data = data.clone();
    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands,
            owners: config
                .discord
                .owners
                .iter()
                .copied()
                .map(serenity::UserId::new)
                .collect(),
            skip_checks_for_owners: false,
            command_check: Some(|ctx| Box::pin(bot_core::command_check(ctx))),
            on_error: |error| Box::pin(bot_core::on_error(error)),
            event_handler: |framework, event| {
                Box::pin(async move {
                    let data = framework.user_data;
                    data.modules
                        .route(framework.serenity_context, event, data)
                        .await;
                    Ok(())
                })
            },
            prefix_options: poise::PrefixFrameworkOptions {
                prefix: None,
                mention_as_prefix: config.discord.prefix_commands,
                dynamic_prefix: Some(|ctx| {
                    Box::pin(async move {
                        if !ctx.framework.user_data.config.discord.prefix_commands {
                            return Ok(None);
                        }
                        if let Some(guild) = ctx.guild_id
                            && let Some((prefix, enabled)) =
                                ctx.framework.user_data.database.prefix(guild.get()).await?
                        {
                            return Ok(enabled.then_some(prefix.into()));
                        }
                        Ok(Some(
                            ctx.framework
                                .user_data
                                .config
                                .discord
                                .default_prefix
                                .clone()
                                .into(),
                        ))
                    })
                }),
                ..Default::default()
            },
            ..Default::default()
        })
        .setup(move |ctx, _ready, framework| {
            Box::pin(async move {
                synchronize(ctx, framework, &startup_data).await?;
                module_utility::start_reminders(startup_data.clone(), ctx.http.clone());
                tracing::info!("Bot ready");
                Ok(startup_data)
            })
        })
        .build();
    let mut client = serenity::ClientBuilder::new(token, intents)
        .framework(framework)
        .register_songbird_with(manager)
        .await?;
    let shard_manager = client.shard_manager.clone();
    let result = tokio::select! {result=client.start()=>result.map_err(anyhow::Error::from),result=shutdown_signal()=>result};
    data.supervisor.cancel.cancel();
    let shutdown = async {
        for module in data.modules.modules.iter().rev() {
            if module.lifecycle.shutdown(&data).await.is_err() {
                tracing::warn!(module = module.descriptor.id, "Module shutdown failed");
            }
        }
        data.supervisor.shutdown().await;
        shard_manager.shutdown_all().await;
        database.close().await;
    };
    if tokio::time::timeout(Duration::from_secs(15), shutdown)
        .await
        .is_err()
    {
        tracing::warn!("Graceful shutdown budget exceeded");
    }
    result
}
async fn shutdown_signal() -> Result<()> {
    #[cfg(unix)]
    {
        let mut term = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())?;
        tokio::select! {r=tokio::signal::ctrl_c()=>r?,_=term.recv()=>{}}
    }
    #[cfg(not(unix))]
    tokio::signal::ctrl_c().await?;
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn full_registry_has_unique_commands() {
        let config = AppConfig::default();
        let service = Arc::new(module_music::MusicService::new(
            &config,
            songbird::Songbird::serenity(),
        ));
        let (registry, commands) = compose(service).unwrap();
        assert_eq!(registry.modules.len(), 7);
        assert!(commands.len() > 30);
        assert_eq!(registry.owner("queue.move").unwrap(), "music");
        assert_eq!(registry.owner("module.disable").unwrap(), "core");
        assert_eq!(registry.owner("role.add").unwrap(), "admin");
        assert_eq!(registry.owner("role.info").unwrap(), "general");
        assert_eq!(registry.owner("torbox.download").unwrap(), "torbox");
        let definitions = poise::builtins::create_application_commands(&commands);
        assert_eq!(definitions.len(), commands.len());
    }
    #[test]
    fn vendored_security_backport_remains_applied() {
        let source = include_str!("../../../vendor/libcrux-secrets/src/int.rs");
        assert!(!source.contains("cmp {cond:w}, 0"));
        assert_eq!(source.matches("tst {cond:w}, 0xff").count(), 4);

        let source = include_str!("../../../vendor/ringbuf/src/traits/consumer.rs");
        assert!(source.contains("struct Guard<'a, C: Consumer + ?Sized>"));
        assert!(source.contains("self.consumer.advance_read_index(self.count)"));
        assert!(source.contains("self.skip(usize::MAX)"));
        assert!(!source.contains("let actual_count = usize::min(count"));
    }
}
