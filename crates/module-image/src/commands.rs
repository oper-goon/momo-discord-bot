use crate::{
    DuckDuckGoImageService,
    client::{ImagePage, PageDirection, SessionAdvance},
};
use anyhow::{Result, ensure};
use bot_core::{BotData, Command, Context, serenity};
use std::sync::Arc;

pub fn commands() -> Vec<Command> {
    vec![image()]
}

fn service(ctx: Context<'_>) -> Result<Arc<DuckDuckGoImageService>> {
    ctx.data().services.get::<DuckDuckGoImageService>()
}

/// Search the web for images using DuckDuckGo with a Bing fallback.
#[poise::command(
    slash_command,
    prefix_command,
    guild_only,
    user_cooldown = 5,
    required_bot_permissions = "EMBED_LINKS"
)]
async fn image(
    ctx: Context<'_>,
    #[description = "What to search for"]
    #[rest]
    query: String,
) -> Result<()> {
    let service = service(ctx)?;
    let results = service.search(&query).await?;
    if results.is_empty() {
        return bot_core::reply(ctx, "No safe image results were found.").await;
    }

    let session_id = ctx.id();
    let page = service.register_session(
        session_id,
        ctx.author().id.get(),
        query.trim().to_owned(),
        results,
    )?;
    ctx.send(reply(&page, session_id, false)).await?;
    Ok(())
}

pub async fn component(
    ctx: &serenity::Context,
    interaction: &serenity::ComponentInteraction,
    data: &BotData,
    service: &Arc<DuckDuckGoImageService>,
) -> Result<()> {
    ensure!(
        !data.supervisor.cancel.is_cancelled(),
        "The bot is shutting down"
    );
    let parts = interaction.data.custom_id.split(':').collect::<Vec<_>>();
    ensure!(
        parts.len() == 3 && parts[0] == "image",
        "Invalid image control"
    );
    let session_id = parts[1].parse::<u64>()?;
    let direction = match parts[2] {
        "previous" => PageDirection::Previous,
        "next" => PageDirection::Next,
        _ => anyhow::bail!("Invalid image control"),
    };

    match service.advance_session(session_id, interaction.user.id.get(), direction)? {
        SessionAdvance::Updated(page) => {
            interaction
                .create_response(
                    &ctx.http,
                    serenity::CreateInteractionResponse::UpdateMessage(
                        serenity::CreateInteractionResponseMessage::new()
                            .embed(embed(&page))
                            .components(controls(session_id, page.index, page.total, false))
                            .allowed_mentions(serenity::CreateAllowedMentions::new()),
                    ),
                )
                .await?;
        }
        SessionAdvance::NotOwner => {
            interaction
                .create_response(
                    &ctx.http,
                    serenity::CreateInteractionResponse::Message(
                        serenity::CreateInteractionResponseMessage::new()
                            .content(
                                "Only the person who ran the image search can use these controls.",
                            )
                            .ephemeral(true)
                            .allowed_mentions(serenity::CreateAllowedMentions::new()),
                    ),
                )
                .await?;
        }
        SessionAdvance::Expired => {
            interaction
                .create_response(
                    &ctx.http,
                    serenity::CreateInteractionResponse::UpdateMessage(
                        serenity::CreateInteractionResponseMessage::new()
                            .content(
                                "These image controls expired. Run the search again to continue.",
                            )
                            .components(vec![])
                            .allowed_mentions(serenity::CreateAllowedMentions::new()),
                    ),
                )
                .await?;
        }
    }
    Ok(())
}

fn reply(page: &ImagePage, session_id: u64, disabled: bool) -> poise::CreateReply {
    poise::CreateReply::default()
        .embed(embed(page))
        .ephemeral(true)
        .components(controls(session_id, page.index, page.total, disabled))
        .allowed_mentions(serenity::CreateAllowedMentions::new())
}

fn embed(page: &ImagePage) -> serenity::CreateEmbed {
    let result = &page.result;
    let mut embed = serenity::CreateEmbed::new()
        .title(truncate(&result.title, 256))
        .description(format!(
            "{} image result for **{}**",
            result.provider,
            truncate(&page.query, 200)
        ))
        .image(result.image_url.clone())
        .footer(serenity::CreateEmbedFooter::new(format!(
            "Momo • Image Search • {}/{}{}",
            page.index + 1,
            page.total,
            result
                .source
                .as_deref()
                .map(|source| format!(" • {}", truncate(source, 200)))
                .unwrap_or_default()
        )));
    if let Some(page_url) = &result.page_url {
        embed = embed.url(page_url);
    }
    embed
}

fn controls(
    session_id: u64,
    index: usize,
    total: usize,
    disabled: bool,
) -> Vec<serenity::CreateActionRow> {
    let disabled = disabled || total <= 1;
    vec![serenity::CreateActionRow::Buttons(vec![
        serenity::CreateButton::new(format!("image:{session_id}:previous"))
            .label("Previous")
            .emoji('◀')
            .style(serenity::ButtonStyle::Secondary)
            .disabled(disabled),
        serenity::CreateButton::new(format!("image:{session_id}:page"))
            .label(format!("{} / {total}", index + 1))
            .style(serenity::ButtonStyle::Secondary)
            .disabled(true),
        serenity::CreateButton::new(format!("image:{session_id}:next"))
            .label("Next")
            .emoji('▶')
            .style(serenity::ButtonStyle::Secondary)
            .disabled(disabled),
    ])]
}

fn truncate(value: &str, max: usize) -> String {
    if value.chars().count() <= max {
        return value.to_owned();
    }
    let mut value: String = value.chars().take(max).collect();
    value.pop();
    value.push('…');
    value
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn truncates_embed_text_on_character_boundaries() {
        assert_eq!(truncate("kitten", 10), "kitten");
        assert_eq!(truncate("🐱🐱🐱", 2), "🐱…");
    }

    #[test]
    fn controls_are_owner_session_specific() {
        let rows = controls(42, 1, 5, false);
        let serenity::CreateActionRow::Buttons(buttons) = &rows[0] else {
            panic!("expected buttons");
        };
        assert_eq!(buttons.len(), 3);
    }
}
