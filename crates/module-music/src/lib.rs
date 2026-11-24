#![forbid(unsafe_code)]
mod commands;
pub mod matching;
mod providers;
pub mod player;
pub mod source;
use anyhow::Result;
use async_trait::async_trait;
use bot_core::{serenity, *};
use futures_util::FutureExt;
use player::{Action, Player, PlayerHandle, SongbirdBackend};
use source::Sources;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::{Mutex, Semaphore};

pub struct MusicService {
    players: Mutex<HashMap<u64, PlayerHandle>>,
    pub sources: Arc<Sources>,
    pub manager: Arc<songbird::Songbird>,
    inspect: Arc<Semaphore>,
}
impl MusicService {
    pub fn new(config: &AppConfig, manager: Arc<songbird::Songbird>) -> Self {
        Self {
            players: Mutex::new(HashMap::new()),
            sources: Arc::new(Sources::new(config.music.clone())),
            manager,
            inspect: Arc::new(Semaphore::new(4)),
        }
    }
    pub async fn existing(&self, guild: u64) -> Option<PlayerHandle> {
        self.players
            .lock()
            .await
            .get(&guild)
            .filter(|h| !h.cancel.is_cancelled() && !h.tx.is_closed())
            .cloned()
    }
    pub async fn player(self: &Arc<Self>, ctx: Context<'_>) -> Result<PlayerHandle> {
        let guild = guild_id(ctx)?;
        let mut players = self.players.lock().await;
        if let Some(handle) = players.get(&guild.get()) {
            anyhow::ensure!(
                !handle.cancel.is_cancelled() && !handle.tx.is_closed(),
                "The previous player is disconnecting; try again shortly"
            );
            return Ok(handle.clone());
        }
        let settings = ctx
            .data()
            .database
            .music_settings(guild.get())
            .await?
            .unwrap_or_else(|| ctx.data().config.music_defaults());
        let (player, handle) = Player::new(
            settings,
            Box::new(SongbirdBackend::new(self.manager.clone(), guild)),
            self.sources.clone(),
            ctx.data().supervisor.cancel.child_token(),
            Some((ctx.serenity_context().http.clone(), ctx.channel_id())),
        );
        players.insert(guild.get(), handle.clone());
        let service = self.clone();
        let id = handle.id;
        ctx.data().supervisor.spawn("guild-player", async move {
            let result = std::panic::AssertUnwindSafe(player.run())
                .catch_unwind()
                .await;
            if result.is_err() {
                let _ = service.manager.remove(guild).await;
            }
            let mut players = service.players.lock().await;
            if players.get(&guild.get()).is_some_and(|h| h.id == id) {
                players.remove(&guild.get());
            }
            result.unwrap_or_else(|_| {
                Err(anyhow::anyhow!("Guild player failed and was disconnected"))
            })
        });
        Ok(handle)
    }
    pub async fn stop(&self, guild: u64) {
        let handle = self.players.lock().await.get(&guild).cloned();
        if let Some(handle) = handle {
            handle.cancel.cancel();
            let _ =
                tokio::time::timeout(std::time::Duration::from_secs(14), handle.tx.closed()).await;
        }
    }
    pub async fn count(&self) -> usize {
        self.players
            .lock()
            .await
            .values()
            .filter(|h| !h.cancel.is_cancelled() && !h.tx.is_closed())
            .count()
    }
    pub async fn inspect(&self, input: &str, user: u64) -> Result<Vec<TrackRef>> {
        let _permit = self
            .inspect
            .clone()
            .try_acquire_owned()
            .map_err(|_| anyhow::anyhow!("Media resolver busy; try again"))?;
        self.sources.inspect(input, user).await
    }
}
struct MusicLifecycle {
    service: Arc<MusicService>,
}
#[async_trait]
impl Lifecycle for MusicLifecycle {
    async fn guild_changed(&self, _: &BotData, guild: u64, enabled: bool) -> Result<()> {
        if !enabled {
            self.service.stop(guild).await;
        }
        Ok(())
    }
    async fn config_changed(&self, data: &BotData, guild: u64) -> Result<()> {
        if let Some(handle) = self.service.existing(guild).await {
            let settings = data
                .database
                .music_settings(guild)
                .await?
                .unwrap_or_else(|| data.config.music_defaults());
            handle.send(Action::Configure(settings)).await?;
        }
        Ok(())
    }
    async fn shutdown(&self, _: &BotData) -> Result<()> {
        for handle in self.service.players.lock().await.values() {
            handle.cancel.cancel();
        }
        Ok(())
    }
    async fn health(&self) -> Result<String> {
        Ok(format!(
            "{}; {} active players",
            if self.service.sources.health().await {
                "healthy"
            } else {
                "degraded: music providers unavailable"
            },
            self.service.count().await
        ))
    }
}
struct VoiceEvents {
    service: Arc<MusicService>,
}
#[async_trait]
impl EventHandler for VoiceEvents {
    async fn handle(
        &self,
        ctx: serenity::Context,
        event: serenity::FullEvent,
        data: BotData,
    ) -> Result<()> {
        if let serenity::FullEvent::InteractionCreate {
            interaction: serenity::Interaction::Component(component),
        } = &event
        {
            return commands::component(&ctx, component, &data, &self.service).await;
        }
        if let serenity::FullEvent::VoiceStateUpdate { new, .. } = event
            && let Some(guild) = new.guild_id
            && let Some(handle) = self.service.existing(guild.get()).await
        {
            let bot = ctx.cache.current_user().id;
            if new.user_id == bot && new.channel_id.is_none() {
                let _ = handle.send(Action::VoiceDisconnected).await;
                return Ok(());
            }
            let snapshot = handle.send(Action::Snapshot).await?;
            if let Some(channel) = snapshot.channel {
                let empty = ctx.cache.guild(guild).map(|g| {
                    !g.voice_states.iter().any(|(id, state)| {
                        id != &bot
                            && state.channel_id.is_some_and(|c| c.get() == channel)
                            && !g.members.get(id).is_some_and(|m| m.user.bot)
                    })
                });
                if let Some(empty) = empty {
                    handle.send(Action::Listeners(empty)).await?;
                }
            }
        }
        Ok(())
    }
}
pub fn module(service: Arc<MusicService>) -> RegisteredModule {
    let mut module = RegisteredModule::new("music", "Music", false, false, commands::commands());
    module.intents = serenity::GatewayIntents::GUILD_VOICE_STATES;
    module.descriptor.provided_services.push("music");
    module.subscriptions.push("voice_state");
    module.subscriptions.push("music_component");
    module.handler = Some(Arc::new(VoiceEvents {
        service: service.clone(),
    }));
    module.lifecycle = Arc::new(MusicLifecycle { service });
    module
}
