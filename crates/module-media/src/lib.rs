#![forbid(unsafe_code)]

use anyhow::{Result, anyhow, ensure};
use bot_core::{BotData, Context, EventHandler, RegisteredModule, serenity};
use serde_json::{Value, json};
use std::{sync::Arc, time::Duration};
use tokio::sync::Semaphore;

mod search;

const ANIME_FIELDS: &str = "id title { english romaji } description(asHtml: false) averageScore status genres studios(isMain: true) { nodes { name } } trailer { id site } coverImage { large } startDate { year } format";

pub struct MediaService {
    client: reqwest::Client,
    token: Option<String>,
    api_key: Option<String>,
    requests: Semaphore,
    pickers: search::PickerStore,
}

impl MediaService {
    pub fn new() -> Result<Self> {
        Ok(Self {
            client: reqwest::Client::builder()
                .no_proxy()
                .redirect(reqwest::redirect::Policy::none())
                .connect_timeout(Duration::from_secs(5))
                .timeout(Duration::from_secs(12))
                .user_agent("Momo media lookup")
                .build()
                .map_err(|_| anyhow!("Could not initialize media lookup"))?,
            token: std::env::var("TMDB_READ_ACCESS_TOKEN")
                .ok()
                .filter(|token| !token.trim().is_empty()),
            api_key: std::env::var("TMDB_API_KEY")
                .ok()
                .filter(|key| !key.trim().is_empty()),
            requests: Semaphore::new(2),
            pickers: search::PickerStore::default(),
        })
    }

    async fn read(&self, request: reqwest::RequestBuilder) -> Result<Value> {
        let mut response = request
            .send()
            .await
            .map_err(|_| anyhow!("Media provider is unavailable; try again later"))?;
        ensure!(
            response.status() != reqwest::StatusCode::TOO_MANY_REQUESTS,
            "Media provider rate limit reached; try again later"
        );
        ensure!(
            response.status().is_success(),
            "Media provider could not complete the lookup"
        );
        let mut bytes = Vec::new();
        while let Some(chunk) = response
            .chunk()
            .await
            .map_err(|_| anyhow!("Media response could not be read"))?
        {
            ensure!(
                bytes.len() + chunk.len() <= 512 * 1024,
                "Media response exceeded the size limit"
            );
            bytes.extend_from_slice(&chunk);
        }
        serde_json::from_slice(&bytes)
            .map_err(|_| anyhow!("Media provider returned an invalid response"))
    }

    async fn lookup(&self, kind: &str, query: &str) -> Result<Option<serenity::CreateEmbed>> {
        let query = query.trim();
        ensure!(
            !query.is_empty() && query.chars().count() <= 200,
            "Enter a title between 1 and 200 characters"
        );
        let _permit = self
            .requests
            .try_acquire()
            .map_err(|_| anyhow!("Media lookup is busy; try again shortly"))?;
        if kind == "anime" {
            let body = json!({"query": format!("query ($search: String!) {{ Media(search: $search, type: ANIME, isAdult: false) {{ {ANIME_FIELDS} }} }}"), "variables": {"search": query}});
            let value = self
                .read(
                    self.client
                        .post("https://graphql.anilist.co")
                        .header(reqwest::header::CONTENT_TYPE, "application/json")
                        .body(body.to_string()),
                )
                .await?;
            if value["errors"]
                .as_array()
                .is_some_and(|errors| !errors.is_empty())
            {
                if value["errors"].as_array().is_some_and(|errors| {
                    errors
                        .iter()
                        .all(|error| error["status"].as_u64() == Some(404))
                }) {
                    return Ok(None);
                }
                return Err(anyhow!(
                    "AniList could not complete the lookup; try again later"
                ));
            }
            let media = &value["data"]["Media"];
            if media.is_null() {
                return Ok(None);
            }
            return Ok(Some(anime_embed(media)?));
        }
        let search = self
            .read(
                self.tmdb_request(&format!("search/{kind}"))?
                    .query(&[("query", query), ("include_adult", "false")]),
            )
            .await?;
        let Some(id) = search["results"]
            .as_array()
            .and_then(|results| results.first())
            .and_then(|item| item["id"].as_u64())
        else {
            return Ok(None);
        };
        let details = self
            .read(
                self.tmdb_request(&format!("{kind}/{id}"))?
                    .query(&[("append_to_response", "videos")]),
            )
            .await?;
        Ok(Some(tmdb_embed(&details, kind, id)))
    }

    fn tmdb_request(&self, path: &str) -> Result<reqwest::RequestBuilder> {
        let request = self
            .client
            .get(format!("https://api.themoviedb.org/3/{path}"));
        if let Some(token) = &self.token {
            Ok(request.bearer_auth(token))
        } else if let Some(key) = &self.api_key {
            Ok(request.query(&[("api_key", key)]))
        } else {
            Err(anyhow!(
                "Movie and TV lookup requires the operator to set TMDB_API_KEY or TMDB_READ_ACCESS_TOKEN"
            ))
        }
    }
}

pub fn module() -> RegisteredModule {
    let mut module = RegisteredModule::new(
        "media",
        "Media",
        false,
        false,
        vec![
            anime(),
            movie(),
            tv(),
            search::animesearch(),
            search::moviesearch(),
            search::tvsearch(),
        ],
    );
    module.descriptor.provided_services.push("media-lookup");
    module.subscriptions.push("media_component");
    module.handler = Some(Arc::new(MediaEvents));
    module
}

struct MediaEvents;

#[async_trait::async_trait]
impl EventHandler for MediaEvents {
    async fn handle(
        &self,
        ctx: serenity::Context,
        event: serenity::FullEvent,
        data: BotData,
    ) -> Result<()> {
        if let serenity::FullEvent::InteractionCreate {
            interaction: serenity::Interaction::Component(component),
        } = event
        {
            search::component(&ctx, &component, &data).await?;
        }
        Ok(())
    }
}

fn artwork(media: &Value, anime: bool) -> Option<String> {
    if anime {
        let raw = media["coverImage"]["large"].as_str()?;
        let url = url::Url::parse(raw).ok()?;
        if url.scheme() == "https"
            && url.host_str() == Some("s4.anilist.co")
            && url.username().is_empty()
            && url.password().is_none()
            && url.port().is_none()
            && url
                .path()
                .starts_with("/file/anilistcdn/media/anime/cover/")
            && url.query().is_none()
            && url.fragment().is_none()
            && raw.len() <= 1024
        {
            Some(url.to_string())
        } else {
            None
        }
    } else {
        let path = media["poster_path"].as_str()?.strip_prefix('/')?;
        if !path.is_empty()
            && path.len() <= 200
            && path
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'.' | b'_' | b'-'))
        {
            Some(format!("https://image.tmdb.org/t/p/w500/{path}"))
        } else {
            None
        }
    }
}

fn with_artwork(embed: serenity::CreateEmbed, media: &Value, anime: bool) -> serenity::CreateEmbed {
    match artwork(media, anime) {
        Some(url) => embed.image(url),
        None => embed,
    }
}

// Strip provider HTML and escape Discord markdown; cap by UTF-16 units for Discord limits.
fn text(value: &str, limit: usize) -> String {
    let mut output = String::new();
    let mut tag = false;
    let mut units = 0;
    for character in value.chars() {
        if character == '<' {
            tag = true;
            continue;
        }
        if character == '>' && tag {
            tag = false;
            continue;
        }
        if tag {
            continue;
        }
        let escaped = matches!(character, '\\' | '*' | '_' | '~' | '`' | '[' | ']' | '|');
        let count = character.len_utf16() + usize::from(escaped);
        if units + count > limit {
            break;
        }
        if escaped {
            output.push('\\');
        }
        output.push(character);
        units += count;
    }
    if output.trim().is_empty() {
        "Not available".to_owned()
    } else {
        output
    }
}

fn string<'a>(value: &'a Value, key: &str) -> &'a str {
    value[key]
        .as_str()
        .filter(|s| !s.trim().is_empty())
        .unwrap_or("Not available")
}

fn names(value: &Value, objects: bool) -> String {
    let names = value
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|item| {
            if objects {
                item["name"].as_str()
            } else {
                item.as_str()
            }
        })
        .take(20)
        .collect::<Vec<_>>()
        .join(", ");
    text(&names, 700)
}

fn trailer(site: &str, id: &str) -> Option<String> {
    if id.is_empty()
        || id.len() > 100
        || !id
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-')
    {
        return None;
    }
    match site {
        "youtube" | "YouTube" => Some(format!("https://www.youtube.com/watch?v={id}")),
        "dailymotion" => Some(format!("https://www.dailymotion.com/video/{id}")),
        _ => None,
    }
}

fn base(
    title: &str,
    description: &str,
    score: String,
    studios: String,
    genres: String,
    status: &str,
    trailer: Option<String>,
) -> serenity::CreateEmbed {
    serenity::CreateEmbed::new()
        .color(0x9ba4b5)
        .title(text(title, 256))
        .description(text(description, 2200))
        .field("Score", score, true)
        .field("Status", text(status, 100), true)
        .field("Studio / Production", studios, false)
        .field("Genres", genres, false)
        .field(
            "Trailer",
            trailer.unwrap_or_else(|| "Not available".to_owned()),
            false,
        )
}

fn anime_embed(media: &Value) -> Result<serenity::CreateEmbed> {
    let id = media["id"]
        .as_u64()
        .ok_or_else(|| anyhow!("AniList returned an invalid result"))?;
    let title = media["title"]["english"]
        .as_str()
        .or_else(|| media["title"]["romaji"].as_str())
        .unwrap_or("Unknown title");
    Ok(with_artwork(
        base(
            title,
            string(media, "description"),
            media["averageScore"]
                .as_u64()
                .filter(|n| *n > 0)
                .map(|n| format!("{n}/100"))
                .unwrap_or_else(|| "Not rated".to_owned()),
            names(&media["studios"]["nodes"], true),
            names(&media["genres"], false),
            &string(media, "status").replace('_', " "),
            trailer(
                string(&media["trailer"], "site"),
                string(&media["trailer"], "id"),
            ),
        )
        .url(format!("https://anilist.co/anime/{id}"))
        .footer(serenity::CreateEmbedFooter::new("Source: AniList")),
        media,
        true,
    ))
}

fn tmdb_embed(media: &Value, kind: &str, id: u64) -> serenity::CreateEmbed {
    let video = media["videos"]["results"]
        .as_array()
        .into_iter()
        .flatten()
        .filter(|video| video["type"] == "Trailer")
        .find_map(|video| trailer(string(video, "site"), string(video, "key")));
    let score = if media["vote_count"].as_u64().unwrap_or(0) > 0 {
        media["vote_average"].as_f64().map(|n| format!("{n:.1}/10"))
    } else {
        None
    }
    .unwrap_or_else(|| "Not rated".to_owned());
    with_artwork(base(string(media, if kind == "movie" { "title" } else { "name" }), string(media, "overview"), score, names(&media["production_companies"], true), names(&media["genres"], true), string(media, "status"), video)
        .url(format!("https://www.themoviedb.org/{kind}/{id}"))
        .footer(serenity::CreateEmbedFooter::new("Source: TMDB • This product uses the TMDB API but is not endorsed or certified by TMDB.")), media, false)
}

async fn lookup(ctx: Context<'_>, kind: &str, query: String) -> Result<()> {
    let service = ctx.data().services.get::<MediaService>()?;
    let result = tokio::time::timeout(Duration::from_secs(25), service.lookup(kind, &query))
        .await
        .map_err(|_| anyhow!("Media lookup timed out; try again later"))??;
    let Some(embed) = result else {
        return bot_core::reply(ctx, "No matching title was found.").await;
    };
    ctx.send(
        poise::CreateReply::default()
            .embed(embed)
            .ephemeral(true)
            .allowed_mentions(serenity::CreateAllowedMentions::new()),
    )
    .await?;
    Ok(())
}

/// Look up an anime's synopsis, score, studio, genres, status, and trailer.
#[poise::command(
    slash_command,
    prefix_command,
    guild_only,
    user_cooldown = 5,
    required_bot_permissions = "EMBED_LINKS"
)]
async fn anime(
    ctx: Context<'_>,
    #[description = "Anime title"]
    #[rest]
    query: String,
) -> Result<()> {
    lookup(ctx, "anime", query).await
}

/// Look up a movie's synopsis, score, production company, genres, status, and trailer.
#[poise::command(
    slash_command,
    prefix_command,
    guild_only,
    user_cooldown = 5,
    required_bot_permissions = "EMBED_LINKS"
)]
async fn movie(
    ctx: Context<'_>,
    #[description = "Movie title"]
    #[rest]
    query: String,
) -> Result<()> {
    lookup(ctx, "movie", query).await
}

/// Look up a TV show's synopsis, score, production company, genres, status, and trailer.
#[poise::command(
    slash_command,
    prefix_command,
    guild_only,
    user_cooldown = 5,
    required_bot_permissions = "EMBED_LINKS"
)]
async fn tv(
    ctx: Context<'_>,
    #[description = "TV show title"]
    #[rest]
    query: String,
) -> Result<()> {
    lookup(ctx, "tv", query).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn remote_text_and_links_are_bounded() {
        assert_eq!(text("<b>Hello</b> *world*", 100), "Hello \\*world\\*");
        assert!(text(&"😀".repeat(3000), 2200).encode_utf16().count() <= 2200);
        assert!(trailer("youtube", "evil&url=https://example.com").is_none());
        assert!(trailer("unknown", "abc").is_none());
        assert_eq!(
            trailer("youtube", "abc-_"),
            Some("https://www.youtube.com/watch?v=abc-_".to_owned())
        );
    }

    #[test]
    fn sparse_results_degrade_gracefully() {
        let anime = anime_embed(&json!({"id": 1, "title": {"romaji": "Test"}})).unwrap();
        let value = serde_json::to_value(anime).unwrap();
        assert_eq!(value["title"], "Test");
        assert_eq!(value["fields"][0]["value"], "Not rated");
        let movie = serde_json::to_value(tmdb_embed(
            &json!({"title": "Test", "vote_average": 0}),
            "movie",
            1,
        ))
        .unwrap();
        assert_eq!(movie["fields"][0]["value"], "Not rated");
    }

    #[test]
    fn posters_use_only_provider_image_hosts() {
        let anime = json!({"id": 1, "title": {"english": "Test"}, "coverImage": {"large": "https://s4.anilist.co/file/anilistcdn/media/anime/cover/large/test.jpg"}});
        let embed = serde_json::to_value(anime_embed(&anime).unwrap()).unwrap();
        assert_eq!(embed["image"]["url"], anime["coverImage"]["large"]);
        let movie = json!({"title": "Test", "poster_path": "/test.jpg"});
        let embed = serde_json::to_value(tmdb_embed(&movie, "movie", 1)).unwrap();
        assert_eq!(
            embed["image"]["url"],
            "https://image.tmdb.org/t/p/w500/test.jpg"
        );
        for path in [
            "//evil.example/a.jpg",
            "/../../a.jpg",
            "/a.jpg?token=secret",
            "/a/b.jpg",
        ] {
            assert!(artwork(&json!({"poster_path": path}), false).is_none());
        }
        for url in [
            "https://evil.example/a.jpg",
            "http://s4.anilist.co/file/anilistcdn/media/anime/cover/a.jpg",
            "https://s4.anilist.co@evil.example/file/anilistcdn/media/anime/cover/a.jpg",
        ] {
            assert!(artwork(&json!({"coverImage": {"large": url}}), true).is_none());
        }
        assert!(artwork(&json!({}), true).is_none());
        assert!(artwork(&json!({}), false).is_none());
    }
}
