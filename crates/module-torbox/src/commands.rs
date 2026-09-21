use crate::{
    TorBoxService,
    client::{CachedTorrent, StreamResult, TorrentFile},
};
use anyhow::{Result, ensure};
use bot_core::{Command, Context, serenity};
use std::sync::Arc;

const MAX_LINKS_PER_RESPONSE: usize = 5;
const MAX_LISTED_FILES: usize = 25;
const MAX_FILE_LIST_BYTES: usize = 1_400;

pub fn commands() -> Vec<Command> {
    vec![torbox()]
}

fn service(ctx: Context<'_>) -> Result<Arc<TorBoxService>> {
    ctx.data().services.get::<TorBoxService>()
}

/// Resolve cached TorBox downloads and video streams.
#[poise::command(
    slash_command,
    guild_only,
    subcommands("download", "stream"),
    subcommand_required
)]
async fn torbox(ctx: Context<'_>) -> Result<()> {
    bot_core::reply(ctx, "Use /torbox download or /torbox stream.").await
}

/// Generate temporary download links for files in a cached magnet.
#[poise::command(slash_command, guild_only, user_cooldown = 10)]
async fn download(
    ctx: Context<'_>,
    #[description = "A BitTorrent magnet link"] magnet: String,
    #[description = "One-based file number from a previous file listing"] file_number: Option<u32>,
) -> Result<()> {
    let service = service(ctx)?;
    let Some((torrent_id, files)) = resolve(&service, &magnet).await? else {
        return bot_core::reply(
            ctx,
            "This torrent is not currently cached by TorBox. Nothing was added or downloaded.",
        )
        .await;
    };

    let selected = if let Some(number) = file_number {
        vec![select_file(&files, number)?]
    } else if files.len() <= MAX_LINKS_PER_RESPONSE {
        files.iter().collect()
    } else {
        return list_files(ctx, &files, false).await;
    };
    ensure!(
        selected.iter().all(|file| !file.infected),
        "TorBox marked the selected file as potentially infected"
    );

    let mut buttons = Vec::with_capacity(selected.len());
    for file in selected {
        let url = service.download_link(torrent_id, file.id).await?;
        buttons
            .push(serenity::CreateButton::new_link(url).label(truncate(&display_name(file), 80)));
    }
    send_buttons(
        ctx,
        "Download ready. These links expire in approximately 3 hours. Start promptly and generate new links if they expire.",
        buttons,
    )
    .await
}

/// Create a TorBox stream for a video in a cached magnet.
#[poise::command(slash_command, guild_only, user_cooldown = 10)]
async fn stream(
    ctx: Context<'_>,
    #[description = "A BitTorrent magnet link"] magnet: String,
    #[description = "One-based file number from a previous video listing"] file_number: Option<u32>,
) -> Result<()> {
    let service = service(ctx)?;
    let Some((torrent_id, files)) = resolve(&service, &magnet).await? else {
        return bot_core::reply(
            ctx,
            "This torrent is not currently cached by TorBox. Nothing was added or streamed.",
        )
        .await;
    };
    let videos: Vec<_> = files.iter().filter(|file| is_video(file)).collect();
    ensure!(!videos.is_empty(), "TorBox found no supported video files");
    let selected = if let Some(number) = file_number {
        let file = select_file(&files, number)?;
        ensure!(is_video(file), "The selected file is not a supported video");
        file
    } else if videos.len() == 1 {
        videos[0]
    } else {
        return list_files(ctx, &files, true).await;
    };
    ensure!(
        !selected.infected,
        "TorBox marked the selected file as potentially infected"
    );

    match service.create_stream(torrent_id, selected.id).await? {
        StreamResult::Ready(url) => {
            send_buttons(
                ctx,
                "TorBox stream ready. The stream link is temporary and may stop working after the session expires.",
                vec![serenity::CreateButton::new_link(url).label("Watch in TorBox")],
            )
            .await
        }
        StreamResult::PlanUnavailable => {
            bot_core::reply(
                ctx,
                "TorBox streaming is not available on the configured subscription. File downloads are still available with /torbox download.",
            )
            .await
        }
        StreamResult::Unsupported => {
            bot_core::reply(
                ctx,
                "TorBox cannot stream this video. You can still request its file link with /torbox download.",
            )
            .await
        }
    }
}

async fn resolve(service: &TorBoxService, magnet: &str) -> Result<Option<(u64, Vec<TorrentFile>)>> {
    let torrent_id = match service.add_cached(magnet).await? {
        CachedTorrent::Added(id) => id,
        CachedTorrent::NotCached => return Ok(None),
    };
    Ok(Some((torrent_id, service.files(torrent_id).await?)))
}

fn select_file(files: &[TorrentFile], number: u32) -> Result<&TorrentFile> {
    ensure!(number > 0, "File number must be at least 1");
    files
        .get(number as usize - 1)
        .ok_or_else(|| anyhow::anyhow!("File number is outside the available range"))
}

async fn list_files(ctx: Context<'_>, files: &[TorrentFile], videos_only: bool) -> Result<()> {
    let mut lines = Vec::new();
    let mut listed_bytes = 0;
    for (index, file) in files.iter().enumerate() {
        if videos_only && !is_video(file) {
            continue;
        }
        if lines.len() == MAX_LISTED_FILES {
            break;
        }
        let status = if file.infected { " — blocked" } else { "" };
        let line = format!(
            "{}. {} ({}){}",
            index + 1,
            truncate(&display_name(file), 70),
            human_size(file.size),
            status
        );
        if listed_bytes + line.len() > MAX_FILE_LIST_BYTES {
            break;
        }
        listed_bytes += line.len();
        lines.push(line);
    }
    let action = if videos_only {
        "Run `/torbox stream` again with the desired `file_number`."
    } else {
        "Run `/torbox download` again with the desired `file_number`."
    };
    bot_core::reply(
        ctx,
        format!("Available files:\n{}\n\n{action}", lines.join("\n")),
    )
    .await
}

async fn send_buttons(
    ctx: Context<'_>,
    content: &str,
    buttons: Vec<serenity::CreateButton>,
) -> Result<()> {
    let rows = buttons
        .chunks(5)
        .map(|buttons| serenity::CreateActionRow::Buttons(buttons.to_vec()))
        .collect();
    ctx.send(
        poise::CreateReply::default()
            .content(content)
            .ephemeral(true)
            .components(rows)
            .allowed_mentions(serenity::CreateAllowedMentions::new()),
    )
    .await?;
    Ok(())
}

fn display_name(file: &TorrentFile) -> String {
    file.short_name
        .as_deref()
        .filter(|name| !name.is_empty())
        .unwrap_or(&file.name)
        .to_owned()
}

fn is_video(file: &TorrentFile) -> bool {
    if file
        .mimetype
        .as_deref()
        .is_some_and(|mime| mime.starts_with("video/"))
    {
        return true;
    }
    let name = display_name(file).to_ascii_lowercase();
    [".mp4", ".mkv", ".webm", ".mov", ".m4v", ".avi"]
        .iter()
        .any(|extension| name.ends_with(extension))
}

fn truncate(value: &str, max: usize) -> String {
    let chars = value.chars().count();
    if chars <= max {
        return value.to_owned();
    }
    let mut value: String = value.chars().take(max).collect();
    value.pop();
    value.push('…');
    value
}

fn human_size(bytes: u64) -> String {
    const UNITS: [&str; 5] = ["B", "KiB", "MiB", "GiB", "TiB"];
    let mut size = bytes as f64;
    let mut unit = 0;
    while size >= 1024.0 && unit < UNITS.len() - 1 {
        size /= 1024.0;
        unit += 1;
    }
    if unit == 0 {
        format!("{bytes} {}", UNITS[unit])
    } else {
        format!("{size:.1} {}", UNITS[unit])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn file(name: &str, mime: Option<&str>) -> TorrentFile {
        TorrentFile {
            id: 1,
            name: name.into(),
            short_name: None,
            size: 1024,
            infected: false,
            mimetype: mime.map(str::to_owned),
        }
    }

    #[test]
    fn recognizes_video_files_conservatively() {
        assert!(is_video(&file("movie.bin", Some("video/mp4"))));
        assert!(is_video(&file("movie.MKV", None)));
        assert!(!is_video(&file("movie.exe", None)));
    }

    #[test]
    fn formats_file_sizes() {
        assert_eq!(human_size(1024), "1.0 KiB");
        assert_eq!(human_size(5), "5 B");
    }
}
