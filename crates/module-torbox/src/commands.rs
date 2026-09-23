use crate::{
    TorBoxService,
    client::{CachedTorrent, StreamResult, TorrentFile},
    picker::{PageDirection, PickerAccess, PickerAction, PickerEntry, PickerPage},
};
use anyhow::{Result, ensure};
use bot_core::{BotData, Command, Context, serenity};
use std::sync::Arc;

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
    #[description = "One-based file number (optional picker bypass)"] file_number: Option<u32>,
) -> Result<()> {
    let service = service(ctx)?;
    let Some((torrent_id, files)) = resolve(&service, &magnet).await? else {
        return bot_core::reply(
            ctx,
            "This torrent is not currently cached by TorBox. Nothing was added or downloaded.",
        )
        .await;
    };

    if let Some(number) = file_number {
        let selected = select_file(&files, number)?;
        ensure!(
            !selected.infected,
            "TorBox marked the selected file as potentially infected"
        );
        let url = service.download_link(torrent_id, selected.id).await?;
        return send_buttons(
            ctx,
            "Download ready. This link expires in approximately 3 hours. Start promptly and generate a new link if it expires.",
            vec![
                serenity::CreateButton::new_link(url)
                    .label(truncate(&display_name(selected), 80)),
            ],
        )
        .await;
    }

    let entries = picker_entries(&files, |_| true);
    if entries.len() == 1 {
        let selected = &entries[0].file;
        let url = service.download_link(torrent_id, selected.id).await?;
        return send_buttons(
            ctx,
            "Download ready. This link expires in approximately 3 hours. Start promptly and generate a new link if it expires.",
            vec![
                serenity::CreateButton::new_link(url)
                    .label(truncate(&display_name(selected), 80)),
            ],
        )
        .await;
    }
    let page = service.pickers.register(
        ctx.id(),
        ctx.author().id.get(),
        torrent_id,
        entries,
        PickerAction::Download,
    )?;
    send_picker(ctx, &page).await
}

/// Create a TorBox stream for a video in a cached magnet.
#[poise::command(slash_command, guild_only, user_cooldown = 10)]
async fn stream(
    ctx: Context<'_>,
    #[description = "A BitTorrent magnet link"] magnet: String,
    #[description = "One-based video file number (optional picker bypass)"] file_number: Option<
        u32,
    >,
) -> Result<()> {
    let service = service(ctx)?;
    let Some((torrent_id, files)) = resolve(&service, &magnet).await? else {
        return bot_core::reply(
            ctx,
            "This torrent is not currently cached by TorBox. Nothing was added or streamed.",
        )
        .await;
    };
    let selected_id = if let Some(number) = file_number {
        let file = select_file(&files, number)?;
        ensure!(is_video(file), "The selected file is not a supported video");
        ensure!(
            !file.infected,
            "TorBox marked the selected file as potentially infected"
        );
        file.id
    } else {
        let entries = picker_entries(&files, is_video);
        ensure!(
            !entries.is_empty(),
            "TorBox found no safe supported video files"
        );
        if entries.len() > 1 {
            let page = service.pickers.register(
                ctx.id(),
                ctx.author().id.get(),
                torrent_id,
                entries,
                PickerAction::Stream,
            )?;
            return send_picker(ctx, &page).await;
        }
        entries[0].file.id
    };

    send_stream_result(ctx, service.create_stream(torrent_id, selected_id).await?).await
}

pub async fn component(
    ctx: &serenity::Context,
    interaction: &serenity::ComponentInteraction,
    data: &BotData,
    service: &Arc<TorBoxService>,
) -> Result<()> {
    ensure!(
        !data.supervisor.cancel.is_cancelled(),
        "The bot is shutting down"
    );
    let parts = interaction.data.custom_id.split(':').collect::<Vec<_>>();
    ensure!(
        parts.len() == 3 && parts[0] == "torbox",
        "Invalid TorBox picker control"
    );
    let session_id = parts[1].parse::<u64>()?;
    match parts[2] {
        "previous" | "next" => {
            let direction = if parts[2] == "previous" {
                PageDirection::Previous
            } else {
                PageDirection::Next
            };
            respond_to_page_access(
                ctx,
                interaction,
                service
                    .pickers
                    .navigate(session_id, interaction.user.id.get(), direction)?,
            )
            .await
        }
        "pick" => {
            let serenity::ComponentInteractionDataKind::StringSelect { values } =
                &interaction.data.kind
            else {
                anyhow::bail!("Invalid TorBox picker selection");
            };
            ensure!(values.len() == 1, "Select exactly one TorBox file");
            let number = values[0].parse::<u32>()?;
            match service
                .pickers
                .select(session_id, interaction.user.id.get(), number)?
            {
                PickerAccess::Ready(selection) => {
                    interaction.defer(&ctx.http).await?;
                    let outcome = match selection.action {
                        PickerAction::Download => service
                            .download_link(selection.torrent_id, selection.entry.file.id)
                            .await
                            .map(|url| {
                                (
                                    "Download ready. This link expires in approximately 3 hours. Start promptly and generate a new link if it expires.".to_owned(),
                                    vec![serenity::CreateButton::new_link(url).label(truncate(
                                        &display_name(&selection.entry.file),
                                        80,
                                    ))],
                                )
                            }),
                        PickerAction::Stream => service
                            .create_stream(selection.torrent_id, selection.entry.file.id)
                            .await
                            .map(stream_response),
                    };
                    let (content, buttons) = outcome.unwrap_or_else(|_| {
                        (
                            "TorBox could not complete that request. Run the command again and retry the file selection.".to_owned(),
                            Vec::new(),
                        )
                    });
                    interaction
                        .edit_response(
                            &ctx.http,
                            serenity::EditInteractionResponse::new()
                                .content(content)
                                .components(button_rows(buttons))
                                .allowed_mentions(serenity::CreateAllowedMentions::new()),
                        )
                        .await?;
                    Ok(())
                }
                PickerAccess::NotOwner => respond_not_owner(ctx, interaction).await,
                PickerAccess::Expired => respond_expired(ctx, interaction).await,
            }
        }
        _ => anyhow::bail!("Invalid TorBox picker control"),
    }
}

async fn send_stream_result(ctx: Context<'_>, result: StreamResult) -> Result<()> {
    let (content, buttons) = stream_response(result);
    send_buttons(ctx, &content, buttons).await
}

fn stream_response(result: StreamResult) -> (String, Vec<serenity::CreateButton>) {
    match result {
        StreamResult::Ready(url) => {
            (
                "TorBox stream ready. The stream link is temporary and may stop working after the session expires.".to_owned(),
                vec![serenity::CreateButton::new_link(url).label("Watch in TorBox")],
            )
        }
        StreamResult::PlanUnavailable => (
            "TorBox streaming is not available on the configured subscription. File downloads are still available with /torbox download.".to_owned(),
            Vec::new(),
        ),
        StreamResult::Unsupported => (
            "TorBox cannot stream this video. You can still request its file link with /torbox download.".to_owned(),
            Vec::new(),
        ),
    }
}

async fn respond_to_page_access(
    ctx: &serenity::Context,
    interaction: &serenity::ComponentInteraction,
    access: PickerAccess<PickerPage>,
) -> Result<()> {
    match access {
        PickerAccess::Ready(page) => {
            interaction
                .create_response(
                    &ctx.http,
                    serenity::CreateInteractionResponse::UpdateMessage(
                        serenity::CreateInteractionResponseMessage::new()
                            .content(picker_content(&page))
                            .components(picker_components(&page))
                            .allowed_mentions(serenity::CreateAllowedMentions::new()),
                    ),
                )
                .await?;
            Ok(())
        }
        PickerAccess::NotOwner => respond_not_owner(ctx, interaction).await,
        PickerAccess::Expired => respond_expired(ctx, interaction).await,
    }
}

async fn respond_not_owner(
    ctx: &serenity::Context,
    interaction: &serenity::ComponentInteraction,
) -> Result<()> {
    interaction
        .create_response(
            &ctx.http,
            serenity::CreateInteractionResponse::Message(
                serenity::CreateInteractionResponseMessage::new()
                    .content("Only the person who ran the TorBox command can use this picker.")
                    .ephemeral(true)
                    .allowed_mentions(serenity::CreateAllowedMentions::new()),
            ),
        )
        .await?;
    Ok(())
}

async fn respond_expired(
    ctx: &serenity::Context,
    interaction: &serenity::ComponentInteraction,
) -> Result<()> {
    interaction
        .create_response(
            &ctx.http,
            serenity::CreateInteractionResponse::UpdateMessage(
                serenity::CreateInteractionResponseMessage::new()
                    .content("This TorBox file picker expired. Run the command again to continue.")
                    .components(vec![])
                    .allowed_mentions(serenity::CreateAllowedMentions::new()),
            ),
        )
        .await?;
    Ok(())
}

async fn send_picker(ctx: Context<'_>, page: &PickerPage) -> Result<()> {
    ctx.send(
        poise::CreateReply::default()
            .content(picker_content(page))
            .ephemeral(true)
            .components(picker_components(page))
            .allowed_mentions(serenity::CreateAllowedMentions::new()),
    )
    .await?;
    Ok(())
}

fn picker_content(page: &PickerPage) -> String {
    let omitted = if page.omitted == 0 {
        String::new()
    } else {
        format!(
            " {} additional files were omitted; use `file_number` to select one directly.",
            page.omitted
        )
    };
    format!(
        "Choose a file to {}. Page {} of {} • {} available. This picker expires in 5 minutes.{omitted}",
        page.action.label(),
        page.page + 1,
        page.pages,
        page.total
    )
}

fn picker_components(page: &PickerPage) -> Vec<serenity::CreateActionRow> {
    let options = page
        .entries
        .iter()
        .map(|entry| {
            serenity::CreateSelectMenuOption::new(
                truncate(&display_name(&entry.file), 100),
                entry.number.to_string(),
            )
            .description(truncate(
                &format!("File #{} • {}", entry.number, human_size(entry.file.size)),
                100,
            ))
        })
        .collect();
    let select = serenity::CreateSelectMenu::new(
        format!("torbox:{}:pick", page.session_id),
        serenity::CreateSelectMenuKind::String { options },
    )
    .placeholder(format!("Select a file to {}", page.action.label()))
    .min_values(1)
    .max_values(1);
    let mut rows = vec![serenity::CreateActionRow::SelectMenu(select)];
    if page.pages > 1 {
        rows.push(serenity::CreateActionRow::Buttons(vec![
            serenity::CreateButton::new(format!("torbox:{}:previous", page.session_id))
                .label("Previous")
                .emoji('◀')
                .style(serenity::ButtonStyle::Secondary),
            serenity::CreateButton::new(format!("torbox:{}:page", page.session_id))
                .label(format!("{} / {}", page.page + 1, page.pages))
                .style(serenity::ButtonStyle::Secondary)
                .disabled(true),
            serenity::CreateButton::new(format!("torbox:{}:next", page.session_id))
                .label("Next")
                .emoji('▶')
                .style(serenity::ButtonStyle::Secondary),
        ]));
    }
    rows
}

fn picker_entries(
    files: &[TorrentFile],
    include: impl Fn(&TorrentFile) -> bool,
) -> Vec<PickerEntry> {
    files
        .iter()
        .enumerate()
        .filter(|(_, file)| !file.infected && include(file))
        .map(|(index, file)| PickerEntry {
            number: index as u32 + 1,
            file: file.clone(),
        })
        .collect()
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

async fn send_buttons(
    ctx: Context<'_>,
    content: &str,
    buttons: Vec<serenity::CreateButton>,
) -> Result<()> {
    let rows = button_rows(buttons);
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

fn button_rows(buttons: Vec<serenity::CreateButton>) -> Vec<serenity::CreateActionRow> {
    buttons
        .chunks(5)
        .map(|buttons| serenity::CreateActionRow::Buttons(buttons.to_vec()))
        .collect()
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

    #[test]
    fn picker_entries_keep_original_numbers_and_exclude_infected_files() {
        let mut files = vec![
            file("first.txt", None),
            file("blocked.mp4", Some("video/mp4")),
            file("third.mp4", Some("video/mp4")),
        ];
        files[1].infected = true;

        let entries = picker_entries(&files, is_video);
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].number, 3);
        assert_eq!(entries[0].file.name, "third.mp4");
    }
}
