use super::*;
use std::{collections::HashMap, sync::Mutex, time::Instant};

const TTL: Duration = Duration::from_secs(300);
const MAX_SESSIONS: usize = 64;

#[derive(Clone)]
struct Choice {
    id: u64,
    label: String,
    description: String,
    preview: serenity::CreateEmbed,
}

struct Session {
    owner: u64,
    guild: u64,
    channel: u64,
    kind: &'static str,
    command: &'static str,
    created: Instant,
    choices: Vec<Choice>,
}

#[derive(Default)]
pub(super) struct PickerStore(Mutex<HashMap<u64, Session>>);

impl PickerStore {
    fn register(&self, id: u64, session: Session) -> Result<()> {
        let mut sessions = self
            .0
            .lock()
            .map_err(|_| anyhow!("Media search is unavailable"))?;
        sessions.retain(|_, entry| entry.created.elapsed() < TTL);
        ensure!(
            sessions.len() < MAX_SESSIONS,
            "Media search menus are busy; try again later"
        );
        sessions.insert(id, session);
        Ok(())
    }

    fn select(
        &self,
        id: u64,
        owner: u64,
        guild: u64,
        channel: u64,
        index: usize,
    ) -> Result<(Session, Choice)> {
        let mut sessions = self
            .0
            .lock()
            .map_err(|_| anyhow!("Media search is unavailable"))?;
        sessions.retain(|_, entry| entry.created.elapsed() < TTL);
        let session = sessions.get(&id).ok_or_else(|| {
            anyhow!("This search menu expired or was already used. Run the search again.")
        })?;
        ensure!(
            session.owner == owner,
            "Only the person who searched can choose a result"
        );
        ensure!(
            session.guild == guild && session.channel == channel,
            "Use the original search menu"
        );
        let choice = session
            .choices
            .get(index)
            .cloned()
            .ok_or_else(|| anyhow!("Invalid media selection"))?;
        let session = sessions
            .remove(&id)
            .ok_or_else(|| anyhow!("Search menu expired"))?;
        Ok((session, choice))
    }
}

impl MediaService {
    async fn search(&self, kind: &str, query: &str) -> Result<Vec<Choice>> {
        let query = query.trim();
        ensure!(
            !query.is_empty() && query.chars().count() <= 200,
            "Enter a title between 1 and 200 characters"
        );
        let _permit = self
            .requests
            .try_acquire()
            .map_err(|_| anyhow!("Media lookup is busy; try again shortly"))?;
        let value = if kind == "anime" {
            let body = json!({"query": format!("query ($search: String!) {{ Page(page: 1, perPage: 10) {{ media(search: $search, type: ANIME, isAdult: false, sort: SEARCH_MATCH) {{ {ANIME_FIELDS} }} }} }}"), "variables": {"search": query}});
            let value = self
                .read(
                    self.client
                        .post("https://graphql.anilist.co")
                        .header(reqwest::header::CONTENT_TYPE, "application/json")
                        .body(body.to_string()),
                )
                .await?;
            ensure!(
                value["errors"]
                    .as_array()
                    .is_none_or(|errors| errors.is_empty()),
                "AniList could not complete the search"
            );
            value["data"]["Page"]["media"].clone()
        } else {
            self.read(
                self.tmdb_request(&format!("search/{kind}"))?
                    .query(&[("query", query), ("include_adult", "false")]),
            )
            .await?["results"]
                .clone()
        };
        choices(&value, kind)
    }

    async fn selected(&self, session: &Session, choice: Choice) -> Result<serenity::CreateEmbed> {
        if session.kind == "anime" {
            return Ok(choice.preview);
        }
        let _permit = self
            .requests
            .try_acquire()
            .map_err(|_| anyhow!("Media lookup is busy; run the search again shortly"))?;
        let value = self
            .read(
                self.tmdb_request(&format!("{}/{}", session.kind, choice.id))?
                    .query(&[("append_to_response", "videos")]),
            )
            .await?;
        Ok(tmdb_embed(&value, session.kind, choice.id))
    }
}

fn choices(value: &Value, kind: &str) -> Result<Vec<Choice>> {
    let items = value
        .as_array()
        .ok_or_else(|| anyhow!("Media provider returned invalid search results"))?;
    items
        .iter()
        .take(10)
        .filter(|item| item["isAdult"] != true && item["adult"] != true)
        .map(|item| {
            let id = item["id"]
                .as_u64()
                .ok_or_else(|| anyhow!("Media provider returned an invalid result"))?;
            let (title, year) = if kind == "anime" {
                (
                    item["title"]["english"]
                        .as_str()
                        .or_else(|| item["title"]["romaji"].as_str())
                        .unwrap_or("Unknown title"),
                    item["startDate"]["year"]
                        .as_u64()
                        .map(|year| year.to_string())
                        .unwrap_or_else(|| "Year unknown".to_owned()),
                )
            } else {
                (
                    string(item, if kind == "movie" { "title" } else { "name" }),
                    item[if kind == "movie" {
                        "release_date"
                    } else {
                        "first_air_date"
                    }]
                    .as_str()
                    .and_then(|date| date.get(..4))
                    .filter(|year| year.bytes().all(|b| b.is_ascii_digit()))
                    .unwrap_or("Year unknown")
                    .to_owned(),
                )
            };
            let description = text(
                &format!(
                    "{year} • {}",
                    if kind == "anime" {
                        string(item, "format")
                    } else {
                        kind
                    }
                ),
                100,
            );
            let preview = if kind == "anime" {
                anime_embed(item)?
            } else {
                with_artwork(
                    serenity::CreateEmbed::new()
                        .title(text(title, 256))
                        .description(text(string(item, "overview"), 1800))
                        .url(format!("https://www.themoviedb.org/{kind}/{id}")),
                    item,
                    false,
                )
            };
            Ok(Choice {
                id,
                label: text(title, 100),
                description,
                preview,
            })
        })
        .collect()
}

async fn search(
    ctx: Context<'_>,
    kind: &'static str,
    command: &'static str,
    query: String,
) -> Result<()> {
    let service = ctx.data().services.get::<MediaService>()?;
    let choices = tokio::time::timeout(Duration::from_secs(15), service.search(kind, &query))
        .await
        .map_err(|_| anyhow!("Media search timed out; try again later"))??;
    let Some(first) = choices.first() else {
        return bot_core::reply(ctx, "No matching titles were found.").await;
    };
    let options = choices
        .iter()
        .enumerate()
        .map(|(index, choice)| {
            serenity::CreateSelectMenuOption::new(&choice.label, index.to_string())
                .description(&choice.description)
        })
        .collect();
    let menu = serenity::CreateSelectMenu::new(
        format!("media:{}", ctx.id()),
        serenity::CreateSelectMenuKind::String { options },
    )
    .placeholder("Choose a title • one use • expires in 5 minutes")
    .min_values(1)
    .max_values(1);
    let preview = first.preview.clone();
    let session = Session {
        owner: ctx.author().id.get(),
        guild: bot_core::guild_id(ctx)?.get(),
        channel: ctx.channel_id().get(),
        kind,
        command,
        created: Instant::now(),
        choices,
    };
    service.pickers.register(ctx.id(), session)?;
    ctx.send(
        poise::CreateReply::default()
            .content("Choose a match below. Preview of the first result:")
            .embed(preview)
            .components(vec![serenity::CreateActionRow::SelectMenu(menu)])
            .ephemeral(true)
            .allowed_mentions(serenity::CreateAllowedMentions::new()),
    )
    .await?;
    Ok(())
}

pub(super) async fn component(
    ctx: &serenity::Context,
    c: &serenity::ComponentInteraction,
    data: &BotData,
) -> Result<()> {
    c.create_response(
        &ctx.http,
        serenity::CreateInteractionResponse::Defer(
            serenity::CreateInteractionResponseMessage::new().ephemeral(true),
        ),
    )
    .await?;
    let outcome = tokio::time::timeout(Duration::from_secs(25), async {
        ensure!(
            !data.supervisor.cancel.is_cancelled(),
            "The bot is shutting down"
        );
        let guild = c
            .guild_id
            .ok_or_else(|| anyhow!("Use media search in a server"))?;
        ensure!(
            data.modules
                .enabled(&data.database, guild.get(), "media")
                .await?,
            "Media is disabled"
        );
        let id = c
            .data
            .custom_id
            .strip_prefix("media:")
            .and_then(|id| id.parse::<u64>().ok())
            .ok_or_else(|| anyhow!("Invalid media control"))?;
        let serenity::ComponentInteractionDataKind::StringSelect { values } = &c.data.kind else {
            return Err(anyhow!("Invalid media selection"));
        };
        ensure!(values.len() == 1, "Select exactly one title");
        let index = values[0]
            .parse::<usize>()
            .map_err(|_| anyhow!("Invalid media selection"))?;
        let service = data.services.get::<MediaService>()?;
        let (session, choice) =
            service
                .pickers
                .select(id, c.user.id.get(), guild.get(), c.channel_id.get(), index)?;
        let member = guild.member(&ctx.http, c.user.id).await?;
        let partial = guild.to_partial_guild(&ctx.http).await?;
        let channel = c
            .channel_id
            .to_channel(&ctx.http)
            .await?
            .guild()
            .ok_or_else(|| anyhow!("Server channel required"))?;
        let permissions = partial.user_permissions_in(&channel, &member);
        let roles = member
            .roles
            .iter()
            .map(|role| role.get())
            .collect::<Vec<_>>();
        ensure!(
            bot_core::permissions::evaluate(
                &data.database.rules(guild.get()).await?,
                &bot_core::permissions::Subject {
                    user: c.user.id.get(),
                    channel: c.channel_id.get(),
                    roles: &roles,
                    owner: data.config.discord.owners.contains(&c.user.id.get()),
                    administrator: permissions.administrator(),
                },
                "media",
                session.command,
                data.config.discord.administrator_bypass,
                true
            )
            .allowed,
            "A server permission policy denies this search"
        );
        service.selected(&session, choice).await
    })
    .await;
    let response = match outcome {
        Ok(Ok(embed)) => serenity::EditInteractionResponse::new().embed(embed),
        // Do not expose Discord HTTP errors or provider error payloads.
        _ => serenity::EditInteractionResponse::new().content("This selection could not be opened. The menu may be expired, already used, owned by someone else, or restricted. Run the search again and retry."),
    };
    c.edit_response(
        &ctx.http,
        response.allowed_mentions(serenity::CreateAllowedMentions::new()),
    )
    .await?;
    Ok(())
}

/// Search anime titles and choose from up to ten matches.
#[poise::command(
    slash_command,
    prefix_command,
    guild_only,
    user_cooldown = 5,
    required_bot_permissions = "EMBED_LINKS"
)]
pub(super) async fn animesearch(
    ctx: Context<'_>,
    #[description = "Anime title"]
    #[rest]
    query: String,
) -> Result<()> {
    search(ctx, "anime", "animesearch", query).await
}

/// Search movie titles and choose from up to ten matches.
#[poise::command(
    slash_command,
    prefix_command,
    guild_only,
    user_cooldown = 5,
    required_bot_permissions = "EMBED_LINKS"
)]
pub(super) async fn moviesearch(
    ctx: Context<'_>,
    #[description = "Movie title"]
    #[rest]
    query: String,
) -> Result<()> {
    search(ctx, "movie", "moviesearch", query).await
}

/// Search TV titles and choose from up to ten matches.
#[poise::command(
    slash_command,
    prefix_command,
    guild_only,
    user_cooldown = 5,
    required_bot_permissions = "EMBED_LINKS"
)]
pub(super) async fn tvsearch(
    ctx: Context<'_>,
    #[description = "TV title"]
    #[rest]
    query: String,
) -> Result<()> {
    search(ctx, "tv", "tvsearch", query).await
}

#[cfg(test)]
mod tests {
    use super::*;

    fn session() -> Session {
        Session {
            owner: 1,
            guild: 2,
            channel: 3,
            kind: "movie",
            command: "moviesearch",
            created: Instant::now(),
            choices: choices(
                &json!([{"id": 10, "title": "Dune", "release_date": "2021-01-01"}]),
                "movie",
            )
            .unwrap(),
        }
    }

    #[test]
    fn menu_access_is_scoped_and_single_use() {
        let store = PickerStore::default();
        store.register(5, session()).unwrap();
        assert!(store.select(5, 9, 2, 3, 0).is_err());
        assert!(store.select(5, 1, 4, 3, 0).is_err());
        assert!(store.select(5, 1, 2, 4, 0).is_err());
        assert!(store.select(5, 1, 2, 3, 50).is_err());
        assert_eq!(store.select(5, 1, 2, 3, 0).unwrap().1.id, 10);
        assert!(store.select(5, 1, 2, 3, 0).is_err());
    }

    #[test]
    fn expired_menus_and_capacity_are_bounded() {
        let store = PickerStore::default();
        let mut expired = session();
        expired.created = Instant::now() - TTL;
        store.register(0, expired).unwrap();
        assert!(store.select(0, 1, 2, 3, 0).is_err());
        for id in 0..MAX_SESSIONS as u64 {
            store.register(id, session()).unwrap();
        }
        assert!(store.register(100, session()).is_err());
    }

    #[test]
    fn search_choices_include_years_and_exclude_adult_results() {
        let results = choices(
            &json!([
                {"id": 1, "title": "Dune", "release_date": "1984-01-01"},
                {"id": 2, "title": "Dune", "release_date": "2021-01-01"},
                {"id": 3, "title": "Excluded", "adult": true}
            ]),
            "movie",
        )
        .unwrap();
        assert_eq!(results.len(), 2);
        assert!(results[0].description.contains("1984"));
        assert!(results[1].description.contains("2021"));
    }
}
