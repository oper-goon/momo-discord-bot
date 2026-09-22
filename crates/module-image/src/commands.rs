use crate::DuckDuckGoImageService;
use anyhow::Result;
use bot_core::{Command, Context, serenity};
use std::sync::Arc;

pub fn commands() -> Vec<Command> {
    vec![image()]
}

fn service(ctx: Context<'_>) -> Result<Arc<DuckDuckGoImageService>> {
    ctx.data().services.get::<DuckDuckGoImageService>()
}

/// Search the web for an image using DuckDuckGo.
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
    let Some(result) = service(ctx)?.search(&query).await? else {
        return bot_core::reply(ctx, "No safe image results were found.").await;
    };

    let title = truncate(&result.title, 256);
    let mut embed = serenity::CreateEmbed::new()
        .title(title)
        .description(format!(
            "DuckDuckGo image result for **{}**",
            truncate(&query, 200)
        ))
        .image(result.image_url)
        .footer(serenity::CreateEmbedFooter::new(format!(
            "Momo • Image Search{}",
            result
                .source
                .as_deref()
                .map(|source| format!(" • {}", truncate(source, 200)))
                .unwrap_or_default()
        )));
    if let Some(page_url) = result.page_url {
        embed = embed.url(page_url);
    }
    ctx.send(
        poise::CreateReply::default()
            .embed(embed)
            .ephemeral(true)
            .allowed_mentions(serenity::CreateAllowedMentions::new()),
    )
    .await?;
    Ok(())
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
}
