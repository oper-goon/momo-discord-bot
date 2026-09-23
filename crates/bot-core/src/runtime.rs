use crate::*;
use anyhow::{Result, ensure};
use async_trait::async_trait;
use futures_util::FutureExt;
use std::{
    any::{Any, TypeId},
    collections::{HashMap, HashSet},
    panic::AssertUnwindSafe,
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
    time::Duration,
};
use tokio::sync::{Mutex, Semaphore};
use tokio_util::{sync::CancellationToken, task::TaskTracker};

#[derive(Default)]
pub struct ServiceRegistry {
    values: HashMap<TypeId, Arc<dyn Any + Send + Sync>>,
}
impl ServiceRegistry {
    pub fn insert<T: Send + Sync + 'static>(&mut self, service: Arc<T>) -> Result<()> {
        ensure!(
            !self.values.contains_key(&TypeId::of::<T>()),
            "Duplicate service type"
        );
        self.values.insert(TypeId::of::<T>(), service);
        Ok(())
    }
    pub fn get<T: Send + Sync + 'static>(&self) -> Result<Arc<T>> {
        self.values
            .get(&TypeId::of::<T>())
            .cloned()
            .and_then(|v| v.downcast().ok())
            .ok_or_else(|| anyhow::anyhow!("Required service is unavailable"))
    }
}
pub struct Supervisor {
    handles: std::sync::Mutex<Vec<tokio::task::AbortHandle>>,
    pub cancel: CancellationToken,
    tracker: TaskTracker,
    failures: Arc<AtomicUsize>,
}
impl Default for Supervisor {
    fn default() -> Self {
        Self {
            handles: std::sync::Mutex::new(Vec::new()),
            cancel: CancellationToken::new(),
            tracker: TaskTracker::new(),
            failures: Arc::new(AtomicUsize::new(0)),
        }
    }
}
impl Supervisor {
    pub fn spawn<F>(&self, name: &'static str, future: F)
    where
        F: std::future::Future<Output = Result<()>> + Send + 'static,
    {
        if self.cancel.is_cancelled() {
            return;
        }
        let failures = self.failures.clone();
        let handle = self.tracker.spawn(async move {
            match AssertUnwindSafe(future).catch_unwind().await {
                Ok(Ok(())) => {}
                _ => {
                    failures.fetch_add(1, Ordering::Relaxed);
                    tracing::error!(task = name, "Supervised task failed");
                }
            }
        });
        if let Ok(mut handles) = self.handles.lock() {
            handles.retain(|h| !h.is_finished());
            handles.push(handle.abort_handle());
        }
    }
    pub fn failures(&self) -> usize {
        self.failures.load(Ordering::Relaxed)
    }
    pub async fn shutdown(&self) {
        self.cancel.cancel();
        self.tracker.close();
        if tokio::time::timeout(Duration::from_secs(12), self.tracker.wait())
            .await
            .is_err()
        {
            tracing::warn!("Supervised task shutdown timed out");
            if let Ok(handles) = self.handles.lock() {
                for handle in handles.iter() {
                    handle.abort();
                }
            }
            let _ = tokio::time::timeout(Duration::from_secs(1), self.tracker.wait()).await;
        }
    }
}
#[async_trait]
pub trait Lifecycle: Send + Sync {
    async fn init(&self, _data: &BotData) -> Result<()> {
        Ok(())
    }
    async fn shutdown(&self, _data: &BotData) -> Result<()> {
        Ok(())
    }
    async fn guild_changed(&self, _data: &BotData, _guild: u64, _enabled: bool) -> Result<()> {
        Ok(())
    }
    async fn config_changed(&self, _data: &BotData, _guild: u64) -> Result<()> {
        Ok(())
    }
    async fn health(&self) -> Result<String> {
        Ok("healthy".into())
    }
}
pub struct DefaultLifecycle;
#[async_trait]
impl Lifecycle for DefaultLifecycle {}
#[async_trait]
pub trait EventHandler: Send + Sync {
    async fn handle(
        &self,
        ctx: serenity::Context,
        event: serenity::FullEvent,
        data: BotData,
    ) -> Result<()>;
}
pub struct RegisteredModule {
    pub extensions: Vec<(&'static str, Command)>,
    pub descriptor: ModuleDescriptor,
    pub commands: Vec<Command>,
    pub intents: serenity::GatewayIntents,
    pub subscriptions: Vec<&'static str>,
    pub handler: Option<Arc<dyn EventHandler>>,
    pub lifecycle: Arc<dyn Lifecycle>,
}
impl RegisteredModule {
    pub fn new(
        id: &'static str,
        name: &'static str,
        required: bool,
        default_enabled: bool,
        commands: Vec<Command>,
    ) -> Self {
        Self {
            extensions: vec![],
            descriptor: ModuleDescriptor {
                id,
                display_name: name,
                version: env!("CARGO_PKG_VERSION"),
                required,
                default_enabled,
                required_services: vec![],
                provided_services: vec![],
            },
            commands,
            intents: serenity::GatewayIntents::empty(),
            subscriptions: vec![],
            handler: None,
            lifecycle: Arc::new(DefaultLifecycle),
        }
    }
}
pub struct ModuleRegistry {
    pub command_limit: Arc<Semaphore>,
    pub modules: Vec<RegisteredModule>,
    owners: HashMap<String, &'static str>,
    change_lock: Mutex<()>,
    events: Arc<Semaphore>,
}
impl ModuleRegistry {
    pub fn new(modules: Vec<RegisteredModule>) -> Result<Self> {
        let mut ids = HashSet::new();
        let mut owners = HashMap::new();
        let mut providers = HashMap::new();
        for module in &modules {
            let d = &module.descriptor;
            ensure!(
                !d.id.is_empty()
                    && d.id
                        .bytes()
                        .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-'),
                "Invalid module ID"
            );
            ensure!(ids.insert(d.id), "Duplicate module ID: {}", d.id);
            for service in &d.provided_services {
                ensure!(
                    providers.insert(*service, d.id).is_none(),
                    "Duplicate service identifier"
                );
            }
            fn register(
                commands: &[Command],
                prefix: &str,
                id: &'static str,
                map: &mut HashMap<String, &'static str>,
            ) -> Result<()> {
                for c in commands {
                    let path = if prefix.is_empty() {
                        c.name.to_string()
                    } else {
                        format!("{prefix}.{}", c.name)
                    };
                    ensure!(
                        map.insert(path.clone(), id).is_none(),
                        "Duplicate command: {path}"
                    );
                    register(&c.subcommands, &path, id, map)?;
                }
                Ok(())
            }
            register(&module.commands, "", d.id, &mut owners)?;
            for (parent, command) in &module.extensions {
                register(std::slice::from_ref(command), parent, d.id, &mut owners)?;
            }
        }
        let mut resolved = HashSet::new();
        for module in &modules {
            for (parent, _) in &module.extensions {
                ensure!(
                    owners.contains_key(*parent),
                    "Missing parent command: {parent}"
                );
            }
        }
        while resolved.len() < modules.len() {
            let before = resolved.len();
            for module in &modules {
                if module
                    .descriptor
                    .required_services
                    .iter()
                    .all(|s| providers.get(s).is_some_and(|p| resolved.contains(p)))
                {
                    resolved.insert(module.descriptor.id);
                }
            }
            ensure!(
                resolved.len() > before,
                "Missing services or module dependency cycle"
            );
        }
        Ok(Self {
            command_limit: Arc::new(Semaphore::new(32)),
            modules,
            owners,
            change_lock: Mutex::new(()),
            events: Arc::new(Semaphore::new(32)),
        })
    }
    pub fn module(&self, id: &str) -> Result<&RegisteredModule> {
        self.modules
            .iter()
            .find(|m| m.descriptor.id == id)
            .ok_or_else(|| anyhow::anyhow!("Unknown module"))
    }
    pub fn owner(&self, path: &str) -> Result<&'static str> {
        self.owners
            .get(path)
            .copied()
            .ok_or_else(|| anyhow::anyhow!("Unknown command path"))
    }
    pub fn paths(&self) -> impl Iterator<Item = (&String, &&'static str)> {
        self.owners.iter()
    }
    pub async fn enabled(&self, db: &Database, guild: u64, id: &str) -> Result<bool> {
        let m = self.module(id)?;
        if m.descriptor.required {
            return Ok(true);
        }
        Ok(db
            .module_enabled(guild, id)
            .await?
            .unwrap_or(m.descriptor.default_enabled))
    }
    pub async fn set_enabled(
        &self,
        data: &BotData,
        guild: u64,
        id: &str,
        enabled: bool,
        user: u64,
    ) -> Result<()> {
        let _guard = self.change_lock.lock().await;
        let m = self.module(id)?;
        ensure!(
            enabled || !m.descriptor.required,
            "Required modules cannot be disabled"
        );
        data.database.set_module(guild, id, enabled, user).await?;
        m.lifecycle.guild_changed(data, guild, enabled).await?;
        Ok(())
    }
    pub async fn route(
        &self,
        ctx: &serenity::Context,
        event: &serenity::FullEvent,
        data: &BotData,
    ) {
        let (kind, guild) = match event {
            serenity::FullEvent::InteractionCreate {
                interaction: serenity::Interaction::Component(component),
            } if component.data.custom_id.starts_with("music:") => {
                ("music_component", component.guild_id.map(|g| g.get()))
            }
            serenity::FullEvent::InteractionCreate {
                interaction: serenity::Interaction::Component(component),
            } if component.data.custom_id.starts_with("image:") => {
                ("image_component", component.guild_id.map(|g| g.get()))
            }
            serenity::FullEvent::InteractionCreate {
                interaction: serenity::Interaction::Component(component),
            } if component.data.custom_id.starts_with("torbox:") => {
                ("torbox_component", component.guild_id.map(|g| g.get()))
            }
            serenity::FullEvent::VoiceStateUpdate { new, .. } => {
                ("voice_state", new.guild_id.map(|g| g.get()))
            }
            serenity::FullEvent::GuildCreate { guild, .. } => {
                ("guild_create", Some(guild.id.get()))
            }
            serenity::FullEvent::Message { new_message } => {
                ("message", new_message.guild_id.map(|g| g.get()))
            }
            _ => return,
        };
        for module in &self.modules {
            if !module.subscriptions.contains(&kind) {
                continue;
            }
            if let Some(guild) = guild
                && !self
                    .enabled(&data.database, guild, module.descriptor.id)
                    .await
                    .unwrap_or(false)
            {
                if let serenity::FullEvent::InteractionCreate {
                    interaction: serenity::Interaction::Component(c),
                } = event
                {
                    let _ = c
                        .create_response(
                            &ctx.http,
                            serenity::CreateInteractionResponse::Message(
                                serenity::CreateInteractionResponseMessage::new()
                                    .content("This module is disabled.")
                                    .ephemeral(true),
                            ),
                        )
                        .await;
                }
                continue;
            }
            let Some(handler) = module.handler.clone() else {
                continue;
            };
            let Ok(permit) = self.events.clone().try_acquire_owned() else {
                tracing::warn!(module = module.descriptor.id, "Event capacity exhausted");
                continue;
            };
            let (ctx, event, data) = (ctx.clone(), event.clone(), data.clone());
            let supervisor = data.supervisor.clone();
            let cancel = supervisor.cancel.clone();
            supervisor.spawn("module-event",async move{let _permit=permit;tokio::select!{_ = cancel.cancelled()=>Ok(()),result=handler.handle(ctx,event,data)=>result}});
        }
    }
}
#[derive(Clone)]
pub struct BotData {
    pub config: Arc<AppConfig>,
    pub database: Database,
    pub modules: Arc<ModuleRegistry>,
    pub services: Arc<ServiceRegistry>,
    pub supervisor: Arc<Supervisor>,
}

pub async fn command_check(ctx: Context<'_>) -> Result<bool> {
    ctx.defer_ephemeral().await?;
    let permit = ctx
        .data()
        .modules
        .command_limit
        .clone()
        .try_acquire_owned()
        .map_err(|_| anyhow::anyhow!("The bot is busy; try again shortly"))?;
    ctx.set_invocation_data(permit).await;
    ensure!(
        !ctx.data().supervisor.cancel.is_cancelled(),
        "The bot is shutting down"
    );
    let path = ctx.command().qualified_name.replace(' ', ".");
    let module = ctx.data().modules.owner(&path)?;
    if let Some(guild) = ctx.guild_id() {
        ensure!(
            ctx.data()
                .modules
                .enabled(&ctx.data().database, guild.get(), module)
                .await?,
            "This module is disabled. Ask a server administrator to enable it"
        );
    }
    if matches!(ctx, poise::Context::Prefix(_)) {
        ensure!(
            ctx.data().config.discord.prefix_commands,
            "Prefix commands are disabled"
        );
        if let Some(guild) = ctx.guild_id()
            && let Some((_, enabled)) = ctx.data().database.prefix(guild.get()).await?
        {
            ensure!(enabled, "Prefix commands are disabled for this server");
        }
    }
    ensure!(
        permissions::decision(ctx, module, &path).await?.allowed,
        "A server permission policy denies this command"
    );
    Ok(true)
}
pub async fn on_error(error: poise::FrameworkError<'_, BotData, BotError>) {
    let id = uuid::Uuid::new_v4();
    match error {
        poise::FrameworkError::Command { ctx, error, .. }
        | poise::FrameworkError::CommandCheckFailed {
            ctx,
            error: Some(error),
            ..
        } => {
            // Errors from dependencies can contain signed URLs or sensitive payloads: do not log their Debug representation.
            tracing::warn!(correlation_id=%id,command=%ctx.command().qualified_name,"Command failed");
            let text = error.to_string();
            let safe = if text.contains("http") || text.contains("token") || text.len() > 250 {
                format!("The operation failed. Reference: {id}")
            } else {
                text
            };
            let _ = reply(ctx, safe).await;
        }
        poise::FrameworkError::CooldownHit {
            remaining_cooldown,
            ctx,
            ..
        } => {
            let seconds = remaining_cooldown.as_secs().max(1);
            let _ = reply(
                ctx,
                format!("Please wait {seconds} second(s) before using this command again."),
            )
            .await;
        }
        other => {
            if let Some(ctx) = other.ctx() {
                let _ = reply(
                    ctx,
                    "Command unavailable: check your permissions, arguments, and cooldown.",
                )
                .await;
            }
            tracing::warn!(correlation_id=%id,"Framework rejected an operation");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rejects_duplicate_ids_and_missing_dependencies() {
        assert!(
            ModuleRegistry::new(vec![
                RegisteredModule::new("core", "Core", true, true, vec![]),
                RegisteredModule::new("core", "Core", true, true, vec![])
            ])
            .is_err()
        );
        let mut m = RegisteredModule::new("music", "Music", false, false, vec![]);
        m.descriptor.required_services.push("missing");
        assert!(ModuleRegistry::new(vec![m]).is_err());
    }
    #[tokio::test]
    async fn activation_persists_and_required_stays_enabled() -> Result<()> {
        let db = Database::open("sqlite::memory:", 1).await?;
        let registry = ModuleRegistry::new(vec![
            RegisteredModule::new("core", "Core", true, true, vec![]),
            RegisteredModule::new("music", "Music", false, false, vec![]),
        ])?;
        assert!(registry.enabled(&db, 1, "core").await?);
        assert!(!registry.enabled(&db, 1, "music").await?);
        db.set_module(1, "music", true, 2).await?;
        assert!(registry.enabled(&db, 1, "music").await?);
        db.close().await;
        assert!(registry.enabled(&db, 1, "music").await.is_err());
        Ok(())
    }
}
