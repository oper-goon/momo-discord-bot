use crate::{
    MusicService,
    player::{Action, PlayerHandle, RepeatMode, Snapshot},
};
use anyhow::{Context as _, Result, ensure};
use bot_core::{serenity, *};
use std::sync::Arc;
pub fn commands() -> Vec<Command> {
    vec![
        play(),
        join(),
        leave(),
        pause(),
        resume(),
        skip(),
        stop(),
        seek(),
        volume(),
        nowplaying(),
        queue(),
        repeat(),
        shuffle(),
        playlist(),
    ]
}
fn service(ctx: Context<'_>) -> Result<Arc<MusicService>> {
    ctx.data().services.get::<MusicService>()
}

pub async fn component(
    ctx: &serenity::Context,
    c: &serenity::ComponentInteraction,
    data: &BotData,
    service: &Arc<MusicService>,
) -> Result<()> {
    c.create_response(
        &ctx.http,
        serenity::CreateInteractionResponse::Defer(
            serenity::CreateInteractionResponseMessage::new().ephemeral(true),
        ),
    )
    .await?;
    let result = async {
        ensure!(
            !data.supervisor.cancel.is_cancelled(),
            "The bot is shutting down"
        );
        let guild = c.guild_id.context("Use music controls in a server")?;
        ensure!(
            data.modules
                .enabled(&data.database, guild.get(), "music")
                .await?,
            "Music is disabled"
        );
        let parts = c.data.custom_id.split(':').collect::<Vec<_>>();
        ensure!(parts.len() == 3, "Invalid control");
        let generation = parts[2].parse::<u64>()?;
        let action = match parts[1] {
            "pause" => Action::Pause,
            "resume" => Action::Resume,
            "skip" => Action::Skip,
            "stop" => Action::Stop,
            _ => anyhow::bail!("Unknown control"),
        };
        let member = guild.member(&ctx.http, c.user.id).await?;
        let partial = guild.to_partial_guild(&ctx.http).await?;
        let channel = c
            .channel_id
            .to_channel(&ctx.http)
            .await?
            .guild()
            .context("Server channel required")?;
        let permissions = partial.user_permissions_in(&channel, &member);
        let roles = member.roles.iter().map(|r| r.get()).collect::<Vec<_>>();
        ensure!(
            bot_core::permissions::evaluate(
                &data.database.rules(guild.get()).await?,
                &bot_core::permissions::Subject {
                    user: c.user.id.get(),
                    channel: c.channel_id.get(),
                    roles: &roles,
                    owner: data.config.discord.owners.contains(&c.user.id.get()),
                    administrator: permissions.administrator()
                },
                "music",
                parts[1],
                data.config.discord.administrator_bypass,
                true
            )
            .allowed,
            "A server policy denies this control"
        );
        let settings = data
            .database
            .music_settings(guild.get())
            .await?
            .unwrap_or_else(|| data.config.music_defaults());
        let bypass = permissions.manage_guild()
            || settings
                .dj_role_id
                .as_ref()
                .is_some_and(|id| member.roles.iter().any(|r| r.to_string() == *id));
        let voice = ctx
            .cache
            .guild(guild)
            .and_then(|g| g.voice_states.get(&c.user.id).and_then(|v| v.channel_id))
            .context("Join the bot's voice channel first")?;
        let handle = service
            .existing(guild.get())
            .await
            .context("No active music player")?;
        handle
            .send(Action::Checked {
                channel: voice.get(),
                bypass,
                generation: Some(generation),
                action: Box::new(action),
            })
            .await?;
        Ok::<_, anyhow::Error>(())
    }
    .await;
    let text = match result {
        Ok(()) => "Playback updated.".into(),
        Err(error) => {
            let text = error.to_string();
            if text.len() > 200 || text.contains("http") {
                "The control failed; try /nowplaying again.".into()
            } else {
                text
            }
        }
    };
    c.edit_response(
        &ctx.http,
        serenity::EditInteractionResponse::new()
            .content(text)
            .allowed_mentions(serenity::CreateAllowedMentions::new()),
    )
    .await?;
    Ok(())
}
async fn authorized(
    ctx: Context<'_>,
    create: bool,
) -> Result<(Arc<MusicService>, PlayerHandle, u64, bool)> {
    let service = service(ctx)?;
    let guild = guild_id(ctx)?;
    let settings = ctx
        .data()
        .database
        .music_settings(guild.get())
        .await?
        .unwrap_or_else(|| ctx.data().config.music_defaults());
    let member = guild.member(ctx.http(), ctx.author().id).await?;
    let dj = effective_permissions(ctx)
        .await?
        .is_some_and(|p| p.manage_guild())
        || settings
            .dj_role_id
            .as_ref()
            .is_some_and(|id| member.roles.iter().any(|r| r.to_string() == *id));
    let caller_channel = ctx
        .guild()
        .and_then(|g| {
            g.voice_states
                .get(&ctx.author().id)
                .and_then(|v| v.channel_id)
        })
        .context("Join a voice channel first")?;
    let bot_id = ctx.serenity_context().cache.current_user().id;
    let bot_member = guild.member(ctx.http(), bot_id).await?;
    let channel = caller_channel
        .to_channel(ctx.http())
        .await?
        .guild()
        .context("Voice channel unavailable")?;
    let partial = guild.to_partial_guild(ctx.http()).await?;
    let perms = partial.user_permissions_in(&channel, &bot_member);
    ensure!(
        perms.connect() && perms.speak(),
        "The bot needs Connect and Speak in your voice channel"
    );
    let handle = if create {
        service.player(ctx).await?
    } else {
        service
            .existing(guild.get())
            .await
            .context("No active player; use /play or /join")?
    };
    let snapshot = handle.send(Action::Snapshot).await?;
    ensure!(
        snapshot.channel.is_none_or(|c| c == caller_channel.get()) || dj,
        "Join the bot's voice channel to control playback"
    );
    Ok((service, handle, caller_channel.get(), dj))
}
async fn control(ctx: Context<'_>, action: Action, message: &str) -> Result<()> {
    ctx.defer_ephemeral().await?;
    let (_, handle, channel, bypass) = authorized(ctx, false).await?;
    handle
        .send(Action::Checked {
            channel,
            bypass,
            generation: None,
            action: Box::new(action),
        })
        .await?;
    reply(ctx, message).await
}
fn describe(snapshot: &Snapshot) -> String {
    let mut lines = vec![format!(
        "Now playing: {}\nVolume: {}% • Repeat: {:?} • {}{}",
        snapshot
            .current
            .as_ref()
            .map(|t| t.title.as_str())
            .unwrap_or("nothing"),
        snapshot.volume,
        snapshot.repeat,
        if snapshot.paused { "paused" } else { "playing" },
        if snapshot.halted {
            " • halted after failures"
        } else {
            ""
        }
    )];
    for (i, t) in snapshot.queue.iter().take(15).enumerate() {
        lines.push(format!("{}. {}", i + 1, t.title));
    }
    if snapshot.queue.len() > 15 {
        lines.push(format!("… {} more tracks", snapshot.queue.len() - 15));
    }
    lines.join("\n")
}
/// Queue a supported website, public audio URL, or search query.
#[poise::command(slash_command, prefix_command, guild_only, user_cooldown = 2)]
async fn play(ctx: Context<'_>, #[rest] input: String) -> Result<()> {
    ctx.defer_ephemeral().await?;
    let (service, handle, channel, _) = authorized(ctx, true).await?;
    let tracks = service.inspect(&input, ctx.author().id.get()).await?;
    ensure!(
        ctx.data()
            .modules
            .enabled(&ctx.data().database, guild_id(ctx)?.get(), "music")
            .await?,
        "Music was disabled while resolving"
    );
    let count = tracks.len();
    if handle.send(Action::Snapshot).await?.channel.is_none() {
        handle
            .send(Action::Join {
                channel,
                allow_move: false,
            })
            .await?;
    }
    handle.send(Action::Enqueue(tracks)).await?;
    reply(ctx, format!("Queued {count} track(s).")).await
}
/// Join your voice channel; DJ or Manage Guild is required to move the bot.
#[poise::command(slash_command, prefix_command, guild_only)]
async fn join(ctx: Context<'_>) -> Result<()> {
    ctx.defer_ephemeral().await?;
    let (_, handle, channel, dj) = authorized(ctx, true).await?;
    handle
        .send(Action::Join {
            channel,
            allow_move: dj,
        })
        .await?;
    reply(ctx, "Voice connected.").await
}
/// Stop, clear the queue, and disconnect.
#[poise::command(slash_command, prefix_command, guild_only)]
async fn leave(ctx: Context<'_>) -> Result<()> {
    control(ctx, Action::Disconnect, "Disconnected.").await
}
/// Pause current playback.
#[poise::command(slash_command, prefix_command, guild_only)]
async fn pause(ctx: Context<'_>) -> Result<()> {
    control(ctx, Action::Pause, "Paused.").await
}
/// Resume current playback.
#[poise::command(slash_command, prefix_command, guild_only)]
async fn resume(ctx: Context<'_>) -> Result<()> {
    control(ctx, Action::Resume, "Resumed.").await
}
/// Skip the current track or retry the retained queue.
#[poise::command(slash_command, prefix_command, guild_only)]
async fn skip(ctx: Context<'_>) -> Result<()> {
    control(ctx, Action::Skip, "Skipped.").await
}
/// Clear all playback while staying connected.
#[poise::command(slash_command, prefix_command, guild_only)]
async fn stop(ctx: Context<'_>) -> Result<()> {
    control(ctx, Action::Stop, "Playback stopped and queue cleared.").await
}
/// Seek to a position in seconds if supported by the source.
#[poise::command(slash_command, prefix_command, guild_only)]
async fn seek(
    ctx: Context<'_>,
    #[min = 0]
    #[max = 86400]
    seconds: u32,
) -> Result<()> {
    control(ctx, Action::Seek(u64::from(seconds)), "Seek completed.").await
}
/// Set playback volume from 0 to 200 percent.
#[poise::command(slash_command, prefix_command, guild_only)]
async fn volume(
    ctx: Context<'_>,
    #[min = 0]
    #[max = 200]
    percent: u16,
) -> Result<()> {
    control(ctx, Action::Volume(percent), "Volume updated.").await
}
/// Show current playback and queue status.
#[poise::command(slash_command, prefix_command, guild_only)]
async fn nowplaying(ctx: Context<'_>) -> Result<()> {
    ctx.defer_ephemeral().await?;
    let (_, handle, _, _) = authorized(ctx, false).await?;
    let snapshot = handle.send(Action::Snapshot).await?;
    let buttons = [
        ("pause", "Pause"),
        ("resume", "Resume"),
        ("skip", "Skip"),
        ("stop", "Stop"),
    ]
    .into_iter()
    .map(|(action, label)| {
        serenity::CreateButton::new(format!("music:{action}:{}", snapshot.generation))
            .label(label)
            .style(serenity::ButtonStyle::Secondary)
    })
    .collect();
    ctx.send(
        poise::CreateReply::default()
            .content(describe(&snapshot).chars().take(1900).collect::<String>())
            .ephemeral(true)
            .components(vec![serenity::CreateActionRow::Buttons(buttons)])
            .allowed_mentions(serenity::CreateAllowedMentions::new()),
    )
    .await?;
    Ok(())
}
/// View or edit the pending queue.
#[poise::command(
    slash_command,
    prefix_command,
    guild_only,
    subcommands("queue_show", "queue_remove", "queue_move", "queue_clear"),
    subcommand_required
)]
async fn queue(ctx: Context<'_>) -> Result<()> {
    reply(ctx, "Use /queue show.").await
}
/// Show the current track and up to fifteen upcoming tracks.
#[poise::command(slash_command, prefix_command, guild_only, rename = "show")]
async fn queue_show(ctx: Context<'_>) -> Result<()> {
    ctx.defer_ephemeral().await?;
    let (_, handle, _, _) = authorized(ctx, false).await?;
    reply(ctx, describe(&handle.send(Action::Snapshot).await?)).await
}
/// Remove a pending track by one-based position.
#[poise::command(slash_command, prefix_command, guild_only, rename = "remove")]
async fn queue_remove(ctx: Context<'_>, #[min = 1] position: u32) -> Result<()> {
    ensure!(position > 0, "Position starts at 1");
    control(ctx, Action::Remove(position as usize - 1), "Track removed.").await
}
/// Move a pending track; positions start at one.
#[poise::command(slash_command, prefix_command, guild_only, rename = "move")]
async fn queue_move(ctx: Context<'_>, #[min = 1] from: u32, #[min = 1] to: u32) -> Result<()> {
    ensure!(from > 0 && to > 0, "Positions start at 1");
    control(
        ctx,
        Action::Move(from as usize - 1, to as usize - 1),
        "Queue updated.",
    )
    .await
}
/// Clear pending tracks without stopping the current track.
#[poise::command(slash_command, prefix_command, guild_only, rename = "clear")]
async fn queue_clear(ctx: Context<'_>) -> Result<()> {
    control(ctx, Action::Clear, "Pending queue cleared.").await
}
/// Set repeat mode to off, track, or queue.
#[poise::command(slash_command, prefix_command, guild_only)]
async fn repeat(ctx: Context<'_>, mode: RepeatMode) -> Result<()> {
    control(ctx, Action::Repeat(mode), "Repeat mode updated.").await
}
/// Shuffle pending tracks only.
#[poise::command(slash_command, prefix_command, guild_only)]
async fn shuffle(ctx: Context<'_>) -> Result<()> {
    control(ctx, Action::Shuffle, "Pending tracks shuffled.").await
}
/// Manage server-local, user-owned playlists.
#[poise::command(
    slash_command,
    prefix_command,
    guild_only,
    subcommands(
        "playlist_save",
        "playlist_list",
        "playlist_play",
        "playlist_delete",
        "playlist_move",
        "playlist_remove"
    )
)]
async fn playlist(ctx: Context<'_>) -> Result<()> {
    reply(
        ctx,
        "Use /playlist list, save, play, move, remove, or delete.",
    )
    .await
}
/// Save current and pending tracks under your playlist name.
#[poise::command(slash_command, prefix_command, guild_only, rename = "save")]
async fn playlist_save(ctx: Context<'_>, name: String) -> Result<()> {
    ctx.defer_ephemeral().await?;
    let (_, handle, _, _) = authorized(ctx, false).await?;
    let snapshot = handle.send(Action::Snapshot).await?;
    let tracks: Vec<_> = snapshot.current.into_iter().chain(snapshot.queue).collect();
    ctx.data()
        .database
        .save_playlist(
            guild_id(ctx)?.get(),
            ctx.author().id.get(),
            &name,
            &tracks,
            ctx.data().config.music.max_playlist_import,
        )
        .await?;
    reply(ctx, "Playlist saved.").await
}
/// List saved playlists and their owners.
#[poise::command(slash_command, prefix_command, guild_only, rename = "list")]
async fn playlist_list(ctx: Context<'_>) -> Result<()> {
    let rows = ctx.data().database.playlists(guild_id(ctx)?.get()).await?;
    let lines: Vec<_> = rows
        .iter()
        .map(|(name, owner)| format!("{name} — <@{owner}>"))
        .collect();
    reply(
        ctx,
        if lines.is_empty() {
            "No saved playlists.".into()
        } else {
            lines.join("\n")
        },
    )
    .await
}
/// Queue a saved playlist, optionally from another owner.
#[poise::command(slash_command, prefix_command, guild_only, rename = "play")]
async fn playlist_play(
    ctx: Context<'_>,
    name: String,
    owner: Option<serenity::User>,
) -> Result<()> {
    ctx.defer_ephemeral().await?;
    let (_, handle, channel, _) = authorized(ctx, true).await?;
    let owner = owner.map(|u| u.id).unwrap_or(ctx.author().id);
    let mut tracks = ctx
        .data()
        .database
        .playlist(guild_id(ctx)?.get(), owner.get(), &name)
        .await?;
    ensure!(
        tracks.len() <= ctx.data().config.music.max_playlist_import,
        "Playlist exceeds current import limit"
    );
    for track in &mut tracks {
        track.requested_by = ctx.author().id.to_string();
    }
    if handle.send(Action::Snapshot).await?.channel.is_none() {
        handle
            .send(Action::Join {
                channel,
                allow_move: false,
            })
            .await?;
    }
    handle.send(Action::Enqueue(tracks)).await?;
    reply(ctx, "Playlist queued.").await
}
async fn playlist_owner(ctx: Context<'_>, owner: Option<serenity::User>) -> Result<u64> {
    let owner = owner.map(|u| u.id.get()).unwrap_or(ctx.author().id.get());
    ensure!(
        owner == ctx.author().id.get()
            || ctx
                .data()
                .config
                .discord
                .owners
                .contains(&ctx.author().id.get())
            || effective_permissions(ctx)
                .await?
                .is_some_and(|p| p.administrator()),
        "Only the playlist owner or an administrator may modify it"
    );
    Ok(owner)
}
/// Delete a playlist you own, or administer.
#[poise::command(slash_command, prefix_command, guild_only, rename = "delete")]
async fn playlist_delete(
    ctx: Context<'_>,
    name: String,
    owner: Option<serenity::User>,
) -> Result<()> {
    let owner = playlist_owner(ctx, owner).await?;
    ensure!(
        ctx.data()
            .database
            .delete_playlist(guild_id(ctx)?.get(), owner, &name)
            .await?,
        "Playlist not found"
    );
    reply(ctx, "Playlist deleted.").await
}
/// Reorder a saved playlist transactionally.
#[poise::command(slash_command, prefix_command, guild_only, rename = "move")]
async fn playlist_move(
    ctx: Context<'_>,
    name: String,
    #[min = 1] from: u32,
    #[min = 1] to: u32,
    owner: Option<serenity::User>,
) -> Result<()> {
    let owner = playlist_owner(ctx, owner).await?;
    let guild = guild_id(ctx)?.get();
    ensure!(from > 0 && to > 0, "Playlist positions start at one");
    ctx.data()
        .database
        .edit_playlist(
            guild,
            owner,
            &name,
            from as usize - 1,
            Some(to as usize - 1),
        )
        .await?;
    reply(ctx, "Playlist reordered.").await
}
/// Remove a saved playlist entry by one-based position.
#[poise::command(slash_command, prefix_command, guild_only, rename = "remove")]
async fn playlist_remove(
    ctx: Context<'_>,
    name: String,
    #[min = 1] position: u32,
    owner: Option<serenity::User>,
) -> Result<()> {
    let owner = playlist_owner(ctx, owner).await?;
    let guild = guild_id(ctx)?.get();
    ensure!(position > 0, "Playlist positions start at one");
    ctx.data()
        .database
        .edit_playlist(guild, owner, &name, position as usize - 1, None)
        .await?;
    reply(ctx, "Playlist entry removed.").await
}
