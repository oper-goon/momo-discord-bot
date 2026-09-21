use crate::source::{PlayableTrack, Sources};
use anyhow::{Context, Result, ensure};
use async_trait::async_trait;
use bot_core::serenity;
use bot_core::{MusicSettings, TrackRef};
use rand::seq::SliceRandom;
use std::{collections::VecDeque, sync::Arc, time::Duration};
use tokio::{
    sync::{mpsc, oneshot},
    task::JoinSet,
    time::Instant,
};
use tokio_util::sync::CancellationToken;

#[derive(Clone, Copy, Debug, PartialEq, Eq, poise::ChoiceParameter)]
pub enum RepeatMode {
    Off,
    Track,
    Queue,
}
#[derive(Clone, Debug)]
pub struct Snapshot {
    pub channel: Option<u64>,
    pub current: Option<TrackRef>,
    pub queue: Vec<TrackRef>,
    pub paused: bool,
    pub volume: u16,
    pub repeat: RepeatMode,
    pub generation: u64,
    pub halted: bool,
}
pub enum Action {
    Join {
        channel: u64,
        allow_move: bool,
    },
    Checked {
        channel: u64,
        bypass: bool,
        generation: Option<u64>,
        action: Box<Action>,
    },
    Enqueue(Vec<TrackRef>),
    Pause,
    Resume,
    Skip,
    Stop,
    Seek(u64),
    Volume(u16),
    Repeat(RepeatMode),
    Shuffle,
    Remove(usize),
    Move(usize, usize),
    Clear,
    Snapshot,
    Disconnect,
    Listeners(bool),
    VoiceDisconnected,
    Configure(MusicSettings),
}
pub struct Request {
    pub action: Action,
    pub reply: oneshot::Sender<Result<Snapshot>>,
}
#[derive(Clone)]
pub struct PlayerHandle {
    pub tx: mpsc::Sender<Request>,
    pub id: uuid::Uuid,
    pub cancel: CancellationToken,
}
impl PlayerHandle {
    pub async fn send(&self, action: Action) -> Result<Snapshot> {
        let (tx, rx) = oneshot::channel();
        self.tx
            .try_send(Request { action, reply: tx })
            .map_err(|_| anyhow::anyhow!("Player is busy or disconnected; try again"))?;
        tokio::time::timeout(Duration::from_secs(15), rx)
            .await
            .context("Player command timed out")?
            .context("Player disconnected")?
    }
}
#[derive(Debug)]
pub enum PlayerEvent {
    Started(u64),
    Ended(u64),
    Failed(u64),
}
#[async_trait]
pub trait VoiceBackend: Send + Sync {
    async fn join(&mut self, channel: u64) -> Result<()>;
    async fn play(
        &mut self,
        input: PlayableTrack,
        generation: u64,
        events: mpsc::Sender<PlayerEvent>,
        volume: u16,
    ) -> Result<()>;
    fn pause(&mut self) -> Result<()>;
    fn resume(&mut self) -> Result<()>;
    fn stop(&mut self);
    fn volume(&mut self, value: u16) -> Result<()>;
    async fn seek(&mut self, seconds: u64) -> Result<()>;
    async fn disconnect(&mut self) -> Result<()>;
}
#[async_trait]
pub trait Resolver: Send + Sync {
    async fn resolve(&self, track: &TrackRef) -> Result<PlayableTrack>;
}
#[async_trait]
impl Resolver for Sources {
    async fn resolve(&self, track: &TrackRef) -> Result<PlayableTrack> {
        self.resolve(track).await
    }
}

pub struct PlayerState {
    pub current: Option<TrackRef>,
    pub queue: VecDeque<TrackRef>,
    pub generation: u64,
    pub repeat: RepeatMode,
    pub paused: bool,
    pub volume: u16,
    pub channel: Option<u64>,
    pub failures: u8,
    pub halted: bool,
}
impl PlayerState {
    pub fn new(volume: u16) -> Self {
        Self {
            current: None,
            queue: VecDeque::new(),
            generation: 0,
            repeat: RepeatMode::Off,
            paused: false,
            volume,
            channel: None,
            failures: 0,
            halted: false,
        }
    }
    pub fn snapshot(&self) -> Snapshot {
        Snapshot {
            current: self.current.clone(),
            queue: self.queue.iter().cloned().collect(),
            generation: self.generation,
            repeat: self.repeat,
            paused: self.paused,
            volume: self.volume,
            channel: self.channel,
            halted: self.halted,
        }
    }
    fn invalidate(&mut self) {
        self.generation = self.generation.wrapping_add(1);
        self.paused = false;
    }
    pub fn advance(&mut self, natural: bool) {
        self.invalidate();
        if let Some(track) = self.current.take()
            && natural
        {
            match self.repeat {
                RepeatMode::Track => self.queue.push_front(track),
                RepeatMode::Queue => self.queue.push_back(track),
                RepeatMode::Off => {}
            }
        }
        self.current = if self.halted {
            None
        } else {
            self.queue.pop_front()
        };
    }
    pub fn move_track(&mut self, from: usize, to: usize) -> Result<()> {
        ensure!(
            from < self.queue.len() && to < self.queue.len(),
            "Queue position is out of range"
        );
        let track = self
            .queue
            .remove(from)
            .context("Queue position is out of range")?;
        self.queue.insert(to, track);
        Ok(())
    }
    pub fn failed(&mut self, generation: u64) -> bool {
        if generation != self.generation || self.current.is_none() {
            return false;
        }
        self.failures = self.failures.saturating_add(1);
        self.halted = self.failures >= 3;
        self.advance(false);
        true
    }
}

pub struct Player {
    pub state: PlayerState,
    settings: MusicSettings,
    backend: Box<dyn VoiceBackend>,
    resolver: Arc<dyn Resolver>,
    cancel: CancellationToken,
    rx: mpsc::Receiver<Request>,
    events_tx: mpsc::Sender<PlayerEvent>,
    events: mpsc::Receiver<PlayerEvent>,
    resolvers: JoinSet<(u64, Result<PlayableTrack>)>,
    idle_since: Option<Instant>,
    empty_since: Option<Instant>,
    notifier: Option<(Arc<serenity::Http>, serenity::ChannelId)>,
}
impl Player {
    pub fn new(
        settings: MusicSettings,
        backend: Box<dyn VoiceBackend>,
        resolver: Arc<dyn Resolver>,
        cancel: CancellationToken,
        notifier: Option<(Arc<serenity::Http>, serenity::ChannelId)>,
    ) -> (Self, PlayerHandle) {
        let (tx, rx) = mpsc::channel(128);
        let (events_tx, events) = mpsc::channel(32);
        let handle = PlayerHandle {
            tx,
            id: uuid::Uuid::new_v4(),
            cancel: cancel.clone(),
        };
        (
            Self {
                state: PlayerState::new(settings.default_volume_percent),
                settings,
                backend,
                resolver,
                cancel,
                rx,
                events_tx,
                events,
                resolvers: JoinSet::new(),
                idle_since: Some(Instant::now()),
                empty_since: None,
                notifier,
            },
            handle,
        )
    }
    fn resolve_current(&mut self) {
        self.resolvers.abort_all();
        if let Some(track) = self.state.current.clone() {
            let resolver = self.resolver.clone();
            let generation = self.state.generation;
            self.idle_since = None;
            self.resolvers
                .spawn(async move { (generation, resolver.resolve(&track).await) });
        } else {
            self.idle_since.get_or_insert_with(Instant::now);
        }
    }
    async fn notify(&self, message: &str) {
        if let Some((http, channel)) = &self.notifier {
            let _ = tokio::time::timeout(
                Duration::from_secs(3),
                channel.send_message(
                    http,
                    serenity::CreateMessage::new()
                        .content(message)
                        .allowed_mentions(serenity::CreateAllowedMentions::new()),
                ),
            )
            .await;
        }
    }
    async fn failed(&mut self, generation: u64) {
        if self.state.failed(generation) {
            self.backend.stop();
            self.notify(if self.state.halted{"Music stopped after three consecutive failures. Pending tracks were retained; use /skip to retry."}else{"A track could not play; advancing to the next queued track."}).await;
            self.resolve_current();
        }
    }
    async fn command(&mut self, action: Action) -> Result<bool> {
        let action = if let Action::Checked {
            channel,
            bypass,
            generation,
            action,
        } = action
        {
            ensure!(
                bypass || self.state.channel.is_none_or(|c| c == channel),
                "Join the bot's current voice channel"
            );
            ensure!(
                generation.is_none_or(|g| g == self.state.generation),
                "This control is stale; run /nowplaying again"
            );
            *action
        } else {
            action
        };
        match action {
            Action::Join {
                channel,
                allow_move,
            } => {
                ensure!(
                    allow_move || self.state.channel.is_none_or(|c| c == channel),
                    "The bot is already in another voice channel"
                );
                tokio::time::timeout(Duration::from_secs(10), self.backend.join(channel))
                    .await
                    .context("Voice connection timed out")??;
                self.state.channel = Some(channel);
            }
            Action::Enqueue(tracks) => {
                ensure!(self.state.channel.is_some(), "Join a voice channel first");
                ensure!(
                    self.state.queue.len() + tracks.len() <= self.settings.max_queue_length,
                    "Queue limit exceeded"
                );
                self.state.queue.extend(tracks);
                if self.state.current.is_none() && !self.state.halted {
                    self.state.advance(false);
                    self.resolve_current();
                }
            }
            Action::Pause => {
                self.backend.pause()?;
                self.state.paused = true;
            }
            Action::Resume => {
                self.backend.resume()?;
                self.state.paused = false;
            }
            Action::Skip => {
                self.backend.stop();
                self.state.halted = false;
                self.state.failures = 0;
                self.state.advance(false);
                self.resolve_current();
            }
            Action::Stop => {
                self.backend.stop();
                self.state.invalidate();
                self.state.current = None;
                self.state.queue.clear();
                self.state.halted = false;
                self.state.failures = 0;
                self.resolve_current();
            }
            Action::Seek(seconds) => {
                tokio::time::timeout(Duration::from_secs(5), self.backend.seek(seconds))
                    .await
                    .context("Seek timed out or is unsupported")??;
            }
            Action::Volume(value) => {
                ensure!(value <= 200, "Volume must be 0–200");
                self.backend.volume(value)?;
                self.state.volume = value;
            }
            Action::Repeat(mode) => self.state.repeat = mode,
            Action::Shuffle => self.state.queue.make_contiguous().shuffle(&mut rand::rng()),
            Action::Remove(index) => {
                self.state
                    .queue
                    .remove(index)
                    .context("Queue position is out of range")?;
            }
            Action::Move(from, to) => self.state.move_track(from, to)?,
            Action::Clear => self.state.queue.clear(),
            Action::Snapshot => {}
            Action::Disconnect | Action::VoiceDisconnected => {
                self.state.invalidate();
                self.state.current = None;
                self.state.queue.clear();
                return Ok(true);
            }
            Action::Listeners(empty) => {
                if empty {
                    self.empty_since.get_or_insert_with(Instant::now);
                } else {
                    self.empty_since = None;
                }
            }
            Action::Configure(settings) => {
                self.settings = settings;
            }
            Action::Checked { .. } => anyhow::bail!("Nested player authorization is invalid"),
        }
        Ok(false)
    }
    pub async fn run(mut self) -> Result<()> {
        let mut timer = tokio::time::interval(Duration::from_secs(1));
        loop {
            tokio::select! {
                biased;
                _=self.cancel.cancelled()=>break,
                request=self.rx.recv()=>{let Some(request)=request else{break};let result=self.command(request.action).await;let done=result.as_ref().is_ok_and(|done|*done);let _=request.reply.send(result.map(|_|self.state.snapshot()));if done{break;}},
                Some(result)=self.resolvers.join_next(),if !self.resolvers.is_empty()=>{match result{Ok((generation,result)) if generation==self.state.generation&&self.state.current.is_some()=>{match result{Ok(input)=>{if self.backend.play(input,generation,self.events_tx.clone(),self.state.volume).await.is_err(){self.failed(generation).await;}},Err(_)=>self.failed(generation).await}},_=>{}}},
                Some(event)=self.events.recv()=>{match event{PlayerEvent::Started(g)if g==self.state.generation=>{self.state.failures=0;if self.settings.announce_now_playing&& let Some(track)=&self.state.current{self.notify(&format!("Now playing: {}",track.title)).await;}},PlayerEvent::Ended(g)if g==self.state.generation&&self.state.current.is_some()=>{self.state.advance(true);self.resolve_current();},PlayerEvent::Failed(g)=>self.failed(g).await,_=>{}}},
                _=timer.tick()=>{if self.idle_since.is_some_and(|t|t.elapsed()>=Duration::from_secs(self.settings.idle_disconnect_seconds))||self.empty_since.is_some_and(|t|t.elapsed()>=Duration::from_secs(self.settings.empty_channel_disconnect_seconds)){break;}}
            }
        }
        self.cancel.cancel();
        self.resolvers.abort_all();
        while self.resolvers.join_next().await.is_some() {}
        self.backend.stop();
        let _ = tokio::time::timeout(Duration::from_secs(3), self.backend.disconnect()).await;
        Ok(())
    }
}

pub struct SongbirdBackend {
    pub manager: Arc<songbird::Songbird>,
    pub guild: serenity::GuildId,
    track: Option<songbird::tracks::TrackHandle>,
}
impl SongbirdBackend {
    pub fn new(manager: Arc<songbird::Songbird>, guild: serenity::GuildId) -> Self {
        Self {
            manager,
            guild,
            track: None,
        }
    }
}
struct TrackEvents {
    generation: u64,
    tx: mpsc::Sender<PlayerEvent>,
    kind: u8,
}
#[async_trait]
impl songbird::EventHandler for TrackEvents {
    async fn act(&self, _ctx: &songbird::EventContext<'_>) -> Option<songbird::Event> {
        let event = match self.kind {
            0 => PlayerEvent::Started(self.generation),
            1 => PlayerEvent::Ended(self.generation),
            _ => PlayerEvent::Failed(self.generation),
        };
        let _ = self.tx.try_send(event);
        None
    }
}
#[async_trait]
impl VoiceBackend for SongbirdBackend {
    async fn join(&mut self, channel: u64) -> Result<()> {
        self.manager
            .join(self.guild, serenity::ChannelId::new(channel))
            .await?;
        Ok(())
    }
    async fn play(
        &mut self,
        input: PlayableTrack,
        generation: u64,
        events: mpsc::Sender<PlayerEvent>,
        volume: u16,
    ) -> Result<()> {
        self.stop();
        let call = self
            .manager
            .get(self.guild)
            .context("Voice connection unavailable")?;
        let mut call = call.lock().await;
        let mut track = songbird::tracks::Track::new(input.input);
        track.volume = f32::from(volume) / 100.0;
        let handle = call.play_only(track);
        for (kind, event) in [
            (0, songbird::TrackEvent::Play),
            (1, songbird::TrackEvent::End),
            (2, songbird::TrackEvent::Error),
        ] {
            handle.add_event(
                songbird::Event::Track(event),
                TrackEvents {
                    generation,
                    tx: events.clone(),
                    kind,
                },
            )?;
        }
        self.track = Some(handle);
        Ok(())
    }
    fn pause(&mut self) -> Result<()> {
        self.track.as_ref().context("Nothing is playing")?.pause()?;
        Ok(())
    }
    fn resume(&mut self) -> Result<()> {
        self.track.as_ref().context("Nothing is playing")?.play()?;
        Ok(())
    }
    fn stop(&mut self) {
        if let Some(track) = self.track.take() {
            let _ = track.stop();
        }
    }
    fn volume(&mut self, value: u16) -> Result<()> {
        if let Some(track) = &self.track {
            track.set_volume(f32::from(value) / 100.0)?;
        }
        Ok(())
    }
    async fn seek(&mut self, seconds: u64) -> Result<()> {
        self.track
            .as_ref()
            .context("Nothing is playing")?
            .seek_async(Duration::from_secs(seconds))
            .await
            .map_err(|_| anyhow::anyhow!("This track cannot seek to that position"))?;
        Ok(())
    }
    async fn disconnect(&mut self) -> Result<()> {
        self.stop();
        self.manager.remove(self.guild).await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn track(title: &str) -> TrackRef {
        TrackRef {
            provider: "fake".into(),
            canonical_url: title.into(),
            title: title.into(),
            duration_ms: None,
            requested_by: "1".into(),
        }
    }
    #[test]
    fn queue_repeat_move_and_stale_failure() {
        let mut s = PlayerState::new(100);
        s.queue.extend([track("a"), track("b"), track("c")]);
        s.move_track(0, 2).unwrap();
        assert_eq!(s.queue[2].title, "a");
        assert!(s.move_track(3, 0).is_err());
        s.advance(false);
        assert_eq!(s.current.as_ref().unwrap().title, "b");
        s.repeat = RepeatMode::Queue;
        s.advance(true);
        assert_eq!(s.queue.back().unwrap().title, "b");
        let generation = s.generation;
        assert!(!s.failed(generation - 1));
        assert_eq!(s.generation, generation);
        s.repeat = RepeatMode::Track;
        let current = s.current.clone().unwrap().title;
        s.advance(true);
        assert_eq!(s.current.as_ref().unwrap().title, current);
    }
    struct FakeBackend;
    #[async_trait]
    impl VoiceBackend for FakeBackend {
        async fn join(&mut self, _: u64) -> Result<()> {
            Ok(())
        }
        async fn play(
            &mut self,
            _: PlayableTrack,
            _: u64,
            _: mpsc::Sender<PlayerEvent>,
            _: u16,
        ) -> Result<()> {
            Ok(())
        }
        fn pause(&mut self) -> Result<()> {
            Ok(())
        }
        fn resume(&mut self) -> Result<()> {
            Ok(())
        }
        fn stop(&mut self) {}
        fn volume(&mut self, _: u16) -> Result<()> {
            Ok(())
        }
        async fn seek(&mut self, _: u64) -> Result<()> {
            Ok(())
        }
        async fn disconnect(&mut self) -> Result<()> {
            Ok(())
        }
    }
    struct SlowResolver;
    #[async_trait]
    impl Resolver for SlowResolver {
        async fn resolve(&self, _: &TrackRef) -> Result<PlayableTrack> {
            tokio::time::sleep(Duration::from_secs(3600)).await;
            anyhow::bail!("fake")
        }
    }
    #[tokio::test(start_paused = true)]
    async fn responsive_during_resolution_and_idle_cleanup() {
        let cancel = CancellationToken::new();
        let (player, handle) = Player::new(
            MusicSettings::default(),
            Box::new(FakeBackend),
            Arc::new(SlowResolver),
            cancel.clone(),
            None,
        );
        let task = tokio::spawn(player.run());
        handle
            .send(Action::Join {
                channel: 1,
                allow_move: false,
            })
            .await
            .unwrap();
        handle
            .send(Action::Enqueue(vec![track("a")]))
            .await
            .unwrap();
        let snapshot = handle.send(Action::Stop).await.unwrap();
        assert!(snapshot.current.is_none() && snapshot.queue.is_empty());
        tokio::time::advance(Duration::from_secs(301)).await;
        task.await.unwrap().unwrap();
        assert!(cancel.is_cancelled());
    }
    #[tokio::test(start_paused = true)]
    async fn empty_timer_cancels_when_listener_returns() {
        let cancel = CancellationToken::new();
        let (player, handle) = Player::new(
            MusicSettings::default(),
            Box::new(FakeBackend),
            Arc::new(SlowResolver),
            cancel.clone(),
            None,
        );
        let task = tokio::spawn(player.run());
        handle
            .send(Action::Join {
                channel: 1,
                allow_move: false,
            })
            .await
            .unwrap();
        handle
            .send(Action::Enqueue(vec![track("a")]))
            .await
            .unwrap();
        handle.send(Action::Listeners(true)).await.unwrap();
        tokio::time::advance(Duration::from_secs(60)).await;
        handle.send(Action::Listeners(false)).await.unwrap();
        tokio::time::advance(Duration::from_secs(61)).await;
        assert!(!cancel.is_cancelled());
        handle.send(Action::Listeners(true)).await.unwrap();
        tokio::time::advance(Duration::from_secs(121)).await;
        task.await.unwrap().unwrap();
        assert!(cancel.is_cancelled());
    }
}
