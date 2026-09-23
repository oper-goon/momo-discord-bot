#![forbid(unsafe_code)]

use anyhow::{Context, Result};
pub use bot_module_api;
use bot_module_api::{Effect, MusicSettings, PermissionRule, TrackRef};
use sqlx::{
    Row, SqlitePool,
    sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions},
};
use std::{
    str::FromStr,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

pub fn now_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
        .min(i64::MAX as u128) as i64
}

#[derive(Clone)]
pub struct Database {
    pool: SqlitePool,
}
#[derive(Debug)]
pub struct Reminder {
    pub id: i64,
    pub user_id: String,
    pub channel_id: Option<String>,
    pub guild_id: Option<String>,
    pub message: String,
    pub due_at_ms: i64,
    pub interval_seconds: Option<i64>,
}
#[derive(Debug)]
pub struct Bookmark {
    pub id: i64,
    pub guild_id: Option<String>,
    pub channel_id: String,
    pub message_id: String,
    pub author_user_id: String,
    pub excerpt: String,
    pub note: Option<String>,
    pub created_at_ms: i64,
}
#[derive(Debug)]
pub struct Case {
    pub number: i64,
    pub action: String,
    pub target: String,
    pub moderator: String,
    pub reason: Option<String>,
    pub status: String,
}

impl Database {
    pub async fn open(url: &str, connections: u32) -> Result<Self> {
        let options = SqliteConnectOptions::from_str(url)?
            .create_if_missing(true)
            .foreign_keys(true)
            .journal_mode(SqliteJournalMode::Wal)
            .busy_timeout(Duration::from_secs(5));
        let pool = SqlitePoolOptions::new()
            .max_connections(connections)
            .connect_with(options)
            .await?;
        sqlx::migrate!("../../migrations").run(&pool).await?;
        Ok(Self { pool })
    }
    pub async fn close(&self) {
        self.pool.close().await;
    }
    pub async fn health(&self) -> Result<i64> {
        Ok(
            sqlx::query_scalar("SELECT MAX(version) FROM _sqlx_migrations WHERE success = 1")
                .fetch_one(&self.pool)
                .await?,
        )
    }
    pub async fn module_enabled(&self, guild: u64, module: &str) -> Result<Option<bool>> {
        Ok(
            sqlx::query_scalar(
                "SELECT enabled FROM guild_modules WHERE guild_id=? AND module_id=?",
            )
            .bind(guild.to_string())
            .bind(module)
            .fetch_optional(&self.pool)
            .await?,
        )
    }
    pub async fn set_module(
        &self,
        guild: u64,
        module: &str,
        enabled: bool,
        user: u64,
    ) -> Result<()> {
        let mut tx = self.pool.begin().await?;
        sqlx::query("INSERT INTO guild_modules VALUES(?,?,?,?,?) ON CONFLICT(guild_id,module_id) DO UPDATE SET enabled=excluded.enabled, updated_by_user_id=excluded.updated_by_user_id,updated_at_ms=excluded.updated_at_ms")
            .bind(guild.to_string()).bind(module).bind(enabled).bind(user.to_string()).bind(now_ms()).execute(&mut *tx).await?;
        tx.commit().await?;
        Ok(())
    }
    pub async fn prefix(&self, guild: u64) -> Result<Option<(String, bool)>> {
        Ok(
            sqlx::query_as("SELECT prefix,prefix_enabled FROM guild_settings WHERE guild_id=?")
                .bind(guild.to_string())
                .fetch_optional(&self.pool)
                .await?,
        )
    }
    pub async fn set_prefix(&self, guild: u64, prefix: &str, enabled: bool) -> Result<()> {
        anyhow::ensure!(
            !prefix.trim().is_empty() && prefix.chars().count() <= 16,
            "Prefix must contain 1–16 nonblank characters"
        );
        sqlx::query("INSERT INTO guild_settings(guild_id,prefix,prefix_enabled,created_at_ms,updated_at_ms) VALUES(?,?,?,?,?) ON CONFLICT(guild_id) DO UPDATE SET prefix=excluded.prefix,prefix_enabled=excluded.prefix_enabled,updated_at_ms=excluded.updated_at_ms")
            .bind(guild.to_string()).bind(prefix).bind(enabled).bind(now_ms()).bind(now_ms()).execute(&self.pool).await?;
        Ok(())
    }
    pub async fn rules(&self, guild: u64) -> Result<Vec<PermissionRule>> {
        sqlx::query("SELECT * FROM permission_rules WHERE guild_id=? ORDER BY id")
            .bind(guild.to_string())
            .fetch_all(&self.pool)
            .await?
            .into_iter()
            .map(|r| {
                Ok(PermissionRule {
                    id: r.try_get("id")?,
                    target: r.try_get("target_path")?,
                    subject_kind: r.try_get("subject_kind")?,
                    subject_id: r.try_get("subject_id")?,
                    channel_id: r.try_get("channel_id")?,
                    effect: if r.try_get::<String, _>("effect")? == "allow" {
                        Effect::Allow
                    } else {
                        Effect::Deny
                    },
                })
            })
            .collect()
    }
    pub async fn set_rule(
        &self,
        guild: u64,
        rule: &PermissionRule,
        user: u64,
        reset: bool,
    ) -> Result<()> {
        let mut tx = self.pool.begin().await?;
        sqlx::query("DELETE FROM permission_rules WHERE guild_id=? AND target_path=? AND subject_kind=? AND subject_id IS ? AND channel_id IS ?").bind(guild.to_string()).bind(&rule.target).bind(&rule.subject_kind).bind(&rule.subject_id).bind(&rule.channel_id).execute(&mut *tx).await?;
        if !reset {
            sqlx::query("INSERT INTO permission_rules(guild_id,target_path,subject_kind,subject_id,channel_id,effect,created_by_user_id,created_at_ms) VALUES(?,?,?,?,?,?,?,?)").bind(guild.to_string()).bind(&rule.target).bind(&rule.subject_kind).bind(&rule.subject_id).bind(&rule.channel_id).bind(if rule.effect==Effect::Allow {"allow"} else {"deny"}).bind(user.to_string()).bind(now_ms()).execute(&mut *tx).await?;
        }
        tx.commit().await?;
        Ok(())
    }
    pub async fn begin_case(
        &self,
        guild: u64,
        action: &str,
        target: u64,
        moderator: u64,
        reason: Option<&str>,
    ) -> Result<i64> {
        // One statement takes SQLite's writer lock before allocating the next case number.
        Ok(sqlx::query_scalar("INSERT INTO moderation_cases(guild_id,case_number,action,target_user_id,moderator_user_id,reason,status,created_at_ms) SELECT ?,COALESCE(MAX(case_number),0)+1,?,?,?,?, 'pending',? FROM moderation_cases WHERE guild_id=? RETURNING case_number").bind(guild.to_string()).bind(action).bind(target.to_string()).bind(moderator.to_string()).bind(reason).bind(now_ms()).bind(guild.to_string()).fetch_one(&self.pool).await?)
    }
    pub async fn finish_case(&self, guild: u64, number: i64, success: bool) -> Result<()> {
        sqlx::query("UPDATE moderation_cases SET status=? WHERE guild_id=? AND case_number=?")
            .bind(if success { "succeeded" } else { "failed" })
            .bind(guild.to_string())
            .bind(number)
            .execute(&self.pool)
            .await?;
        Ok(())
    }
    pub async fn cases(
        &self,
        guild: u64,
        target: Option<u64>,
        number: Option<i64>,
    ) -> Result<Vec<Case>> {
        sqlx::query("SELECT * FROM moderation_cases WHERE guild_id=? AND (? IS NULL OR target_user_id=?) AND (? IS NULL OR case_number=?) ORDER BY case_number DESC LIMIT 20").bind(guild.to_string()).bind(target.map(|v|v.to_string())).bind(target.map(|v|v.to_string())).bind(number).bind(number).fetch_all(&self.pool).await?.into_iter().map(|r| Ok(Case { number:r.try_get("case_number")?,action:r.try_get("action")?,target:r.try_get("target_user_id")?,moderator:r.try_get("moderator_user_id")?,reason:r.try_get("reason")?,status:r.try_get("status")? })).collect()
    }
    pub async fn add_reminder(
        &self,
        guild: Option<u64>,
        channel: u64,
        user: u64,
        message: &str,
        due: i64,
        interval_seconds: Option<u64>,
    ) -> Result<i64> {
        Ok(sqlx::query_scalar("INSERT INTO reminders(guild_id,channel_id,user_id,message,due_at_ms,interval_seconds) VALUES(?,?,?,?,?,?) RETURNING id").bind(guild.map(|v|v.to_string())).bind(channel.to_string()).bind(user.to_string()).bind(message).bind(due).bind(interval_seconds.map(|value| value as i64)).fetch_one(&self.pool).await?)
    }
    pub async fn reminders(&self, user: Option<u64>, due: Option<i64>) -> Result<Vec<Reminder>> {
        sqlx::query("SELECT * FROM reminders WHERE delivered_at_ms IS NULL AND (? IS NULL OR user_id=?) AND (? IS NULL OR due_at_ms<=?) ORDER BY due_at_ms LIMIT 100").bind(user.map(|v|v.to_string())).bind(user.map(|v|v.to_string())).bind(due).bind(due).fetch_all(&self.pool).await?.into_iter().map(|r|Ok(Reminder { id:r.try_get("id")?,user_id:r.try_get("user_id")?,channel_id:r.try_get("channel_id")?,guild_id:r.try_get("guild_id")?,message:r.try_get("message")?,due_at_ms:r.try_get("due_at_ms")?,interval_seconds:r.try_get("interval_seconds")? })).collect()
    }
    pub async fn reminder_result(&self, id: i64, success: bool) -> Result<()> {
        if success {
            sqlx::query("UPDATE reminders SET delivered_at_ms=CASE WHEN interval_seconds IS NULL THEN ? ELSE NULL END,due_at_ms=CASE WHEN interval_seconds IS NULL THEN due_at_ms ELSE ?+(interval_seconds*1000) END,failure_count=0 WHERE id=?")
                .bind(now_ms()).bind(now_ms())
                .bind(id)
                .execute(&self.pool)
                .await?;
        } else {
            sqlx::query("UPDATE reminders SET failure_count=failure_count+1,due_at_ms=?+MIN(3600000,30000*(failure_count+1)) WHERE id=?").bind(now_ms()).bind(id).execute(&self.pool).await?;
        }
        Ok(())
    }
    pub async fn update_reminder(
        &self,
        id: i64,
        user: u64,
        due: i64,
        message: Option<&str>,
    ) -> Result<bool> {
        Ok(sqlx::query("UPDATE reminders SET due_at_ms=?,message=COALESCE(?,message),delivered_at_ms=NULL,failure_count=0 WHERE id=? AND user_id=?")
            .bind(due).bind(message).bind(id).bind(user.to_string()).execute(&self.pool).await?.rows_affected() > 0)
    }
    pub async fn cancel_reminder(&self, id: i64, user: u64) -> Result<bool> {
        Ok(
            sqlx::query("DELETE FROM reminders WHERE id=? AND user_id=?")
                .bind(id)
                .bind(user.to_string())
                .execute(&self.pool)
                .await?
                .rows_affected()
                > 0,
        )
    }
    pub async fn add_bookmark(
        &self,
        user: u64,
        guild: Option<u64>,
        channel: u64,
        message: u64,
        author: u64,
        excerpt: &str,
    ) -> Result<i64> {
        Ok(sqlx::query_scalar("INSERT INTO bookmarks(user_id,guild_id,channel_id,message_id,author_user_id,excerpt,created_at_ms) VALUES(?,?,?,?,?,?,?) ON CONFLICT(user_id,message_id) DO UPDATE SET excerpt=excluded.excerpt RETURNING id")
            .bind(user.to_string()).bind(guild.map(|value| value.to_string())).bind(channel.to_string()).bind(message.to_string()).bind(author.to_string()).bind(excerpt).bind(now_ms()).fetch_one(&self.pool).await?)
    }
    pub async fn bookmarks(&self, user: u64, search: Option<&str>) -> Result<Vec<Bookmark>> {
        let search =
            search.map(|value| format!("%{}%", value.replace('%', "\\%").replace('_', "\\_")));
        sqlx::query("SELECT * FROM bookmarks WHERE user_id=? AND (? IS NULL OR excerpt LIKE ? ESCAPE '\\' OR note LIKE ? ESCAPE '\\') ORDER BY created_at_ms DESC LIMIT 25")
            .bind(user.to_string()).bind(&search).bind(&search).bind(&search).fetch_all(&self.pool).await?.into_iter().map(|r| Ok(Bookmark { id:r.try_get("id")?,guild_id:r.try_get("guild_id")?,channel_id:r.try_get("channel_id")?,message_id:r.try_get("message_id")?,author_user_id:r.try_get("author_user_id")?,excerpt:r.try_get("excerpt")?,note:r.try_get("note")?,created_at_ms:r.try_get("created_at_ms")? })).collect()
    }
    pub async fn delete_bookmark(&self, id: i64, user: u64) -> Result<bool> {
        Ok(
            sqlx::query("DELETE FROM bookmarks WHERE id=? AND user_id=?")
                .bind(id)
                .bind(user.to_string())
                .execute(&self.pool)
                .await?
                .rows_affected()
                > 0,
        )
    }
    pub async fn music_settings(&self, guild: u64) -> Result<Option<MusicSettings>> {
        let row = sqlx::query("SELECT * FROM music_settings WHERE guild_id=?")
            .bind(guild.to_string())
            .fetch_optional(&self.pool)
            .await?;
        row.map(|r| {
            Ok(MusicSettings {
                dj_role_id: r.try_get("dj_role_id")?,
                default_volume_percent: r.try_get::<i64, _>("default_volume_percent")? as u16,
                max_queue_length: r.try_get::<i64, _>("max_queue_length")? as usize,
                idle_disconnect_seconds: r.try_get::<i64, _>("idle_disconnect_seconds")? as u64,
                empty_channel_disconnect_seconds: r
                    .try_get::<i64, _>("empty_channel_disconnect_seconds")?
                    as u64,
                announce_now_playing: r.try_get("announce_now_playing")?,
            })
        })
        .transpose()
    }
    pub async fn save_music_settings(
        &self,
        guild: u64,
        settings: &MusicSettings,
        limit: usize,
    ) -> Result<()> {
        settings.validate(limit)?;
        let mut tx = self.pool.begin().await?;
        sqlx::query("INSERT INTO music_settings VALUES(?,?,?,?,?,?,?) ON CONFLICT(guild_id) DO UPDATE SET dj_role_id=excluded.dj_role_id,default_volume_percent=excluded.default_volume_percent,max_queue_length=excluded.max_queue_length,idle_disconnect_seconds=excluded.idle_disconnect_seconds,empty_channel_disconnect_seconds=excluded.empty_channel_disconnect_seconds,announce_now_playing=excluded.announce_now_playing")
        .bind(guild.to_string()).bind(&settings.dj_role_id).bind(i64::from(settings.default_volume_percent)).bind(settings.max_queue_length as i64).bind(settings.idle_disconnect_seconds as i64).bind(settings.empty_channel_disconnect_seconds as i64).bind(settings.announce_now_playing).execute(&mut *tx).await?;
        tx.commit().await?;
        Ok(())
    }
    pub async fn save_playlist(
        &self,
        guild: u64,
        owner: u64,
        name: &str,
        tracks: &[TrackRef],
        limit: usize,
    ) -> Result<()> {
        anyhow::ensure!(
            !name.trim().is_empty() && name.len() <= 64,
            "Playlist names must be 1–64 bytes"
        );
        anyhow::ensure!(
            !tracks.is_empty() && tracks.len() <= limit,
            "Playlist is empty or exceeds import limit"
        );
        let mut tx = self.pool.begin().await?;
        let id:String=sqlx::query_scalar("INSERT INTO music_playlists VALUES(?,?,?,?,?,?) ON CONFLICT(guild_id,owner_user_id,name) DO UPDATE SET updated_at_ms=excluded.updated_at_ms RETURNING id").bind(uuid::Uuid::new_v4().to_string()).bind(guild.to_string()).bind(owner.to_string()).bind(name).bind(now_ms()).bind(now_ms()).fetch_one(&mut *tx).await?;
        sqlx::query("DELETE FROM music_playlist_tracks WHERE playlist_id=?")
            .bind(&id)
            .execute(&mut *tx)
            .await?;
        for (position, track) in tracks.iter().enumerate() {
            sqlx::query("INSERT INTO music_playlist_tracks(playlist_id,position,provider,canonical_url,title,duration_ms,metadata_json) VALUES(?,?,?,?,?,?,?)").bind(&id).bind(position as i64).bind(&track.provider).bind(&track.canonical_url).bind(&track.title).bind(track.duration_ms.map(|v|v as i64)).bind(serde_json::to_string(track)?).execute(&mut *tx).await?;
        }
        tx.commit().await?;
        Ok(())
    }
    pub async fn playlists(&self, guild: u64) -> Result<Vec<(String, String)>> {
        Ok(sqlx::query_as("SELECT name,owner_user_id FROM music_playlists WHERE guild_id=? ORDER BY name LIMIT 100").bind(guild.to_string()).fetch_all(&self.pool).await?)
    }
    pub async fn playlist(&self, guild: u64, owner: u64, name: &str) -> Result<Vec<TrackRef>> {
        let rows:Vec<String>=sqlx::query_scalar("SELECT t.metadata_json FROM music_playlist_tracks t JOIN music_playlists p ON p.id=t.playlist_id WHERE p.guild_id=? AND p.owner_user_id=? AND p.name=? ORDER BY t.position").bind(guild.to_string()).bind(owner.to_string()).bind(name).fetch_all(&self.pool).await?;
        anyhow::ensure!(!rows.is_empty(), "Playlist not found");
        rows.into_iter()
            .map(|s| serde_json::from_str(&s).context("Invalid persisted playlist track"))
            .collect()
    }
    pub async fn delete_playlist(&self, guild: u64, owner: u64, name: &str) -> Result<bool> {
        Ok(sqlx::query(
            "DELETE FROM music_playlists WHERE guild_id=? AND owner_user_id=? AND name=?",
        )
        .bind(guild.to_string())
        .bind(owner.to_string())
        .bind(name)
        .execute(&self.pool)
        .await?
        .rows_affected()
            > 0)
    }
    pub async fn note_bookmark(&self, id: i64, user: u64, note: &str) -> Result<bool> {
        Ok(
            sqlx::query("UPDATE bookmarks SET note=? WHERE id=? AND user_id=?")
                .bind(note)
                .bind(id)
                .bind(user.to_string())
                .execute(&self.pool)
                .await?
                .rows_affected()
                > 0,
        )
    }
    /// Lock the playlist for writing before reading its entries to avoid lost edits.
    pub async fn edit_playlist(
        &self,
        guild: u64,
        owner: u64,
        name: &str,
        from: usize,
        to: Option<usize>,
    ) -> Result<()> {
        let mut tx = self.pool.begin().await?;
        let id:String=sqlx::query_scalar("UPDATE music_playlists SET updated_at_ms=? WHERE guild_id=? AND owner_user_id=? AND name=? RETURNING id").bind(now_ms()).bind(guild.to_string()).bind(owner.to_string()).bind(name).fetch_optional(&mut *tx).await?.context("Playlist not found")?;
        let rows: Vec<String> = sqlx::query_scalar(
            "SELECT metadata_json FROM music_playlist_tracks WHERE playlist_id=? ORDER BY position",
        )
        .bind(&id)
        .fetch_all(&mut *tx)
        .await?;
        let mut tracks: Vec<TrackRef> = rows
            .iter()
            .map(|s| serde_json::from_str(s))
            .collect::<std::result::Result<_, _>>()?;
        anyhow::ensure!(
            from < tracks.len() && to.is_none_or(|to| to < tracks.len()),
            "Playlist position is out of range"
        );
        let track = tracks.remove(from);
        if let Some(to) = to {
            tracks.insert(to, track);
        }
        sqlx::query("DELETE FROM music_playlist_tracks WHERE playlist_id=?")
            .bind(&id)
            .execute(&mut *tx)
            .await?;
        for (position, track) in tracks.iter().enumerate() {
            sqlx::query("INSERT INTO music_playlist_tracks(playlist_id,position,provider,canonical_url,title,duration_ms,metadata_json) VALUES(?,?,?,?,?,?,?)").bind(&id).bind(position as i64).bind(&track.provider).bind(&track.canonical_url).bind(&track.title).bind(track.duration_ms.map(|v|v as i64)).bind(serde_json::to_string(track)?).execute(&mut *tx).await?;
        }
        if tracks.is_empty() {
            sqlx::query("DELETE FROM music_playlists WHERE id=?")
                .bind(&id)
                .execute(&mut *tx)
                .await?;
        }
        tx.commit().await?;
        Ok(())
    }
    pub async fn sync_hash(&self, scope: &str) -> Result<Option<String>> {
        Ok(
            sqlx::query_scalar("SELECT hash FROM command_sync WHERE scope=?")
                .bind(scope)
                .fetch_optional(&self.pool)
                .await?,
        )
    }
    pub async fn set_sync_hash(&self, scope: &str, hash: &str) -> Result<()> {
        sqlx::query("INSERT INTO command_sync VALUES(?,?) ON CONFLICT(scope) DO UPDATE SET hash=excluded.hash").bind(scope).bind(hash).execute(&self.pool).await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn migrations_repositories_and_transactions() -> Result<()> {
        let db = Database::open("sqlite::memory:", 1).await?;
        assert_eq!(db.health().await?, 2);
        assert_eq!(db.module_enabled(u64::MAX, "music").await?, None);
        db.set_module(u64::MAX, "music", true, 1).await?;
        assert_eq!(db.module_enabled(u64::MAX, "music").await?, Some(true));
        assert!(db.set_prefix(1, " ", true).await.is_err());
        db.set_prefix(1, "?", true).await?;
        assert_eq!(db.prefix(1).await?, Some(("?".into(), true)));
        assert_eq!(db.begin_case(1, "warn", 2, 3, Some("test")).await?, 1);
        assert_eq!(db.begin_case(1, "warn", 2, 3, None).await?, 2);
        let track = TrackRef {
            provider: "yt-dlp".into(),
            canonical_url: "https://www.youtube.com/watch?v=test".into(),
            title: "test".into(),
            duration_ms: None,
            requested_by: "1".into(),
        };
        db.save_playlist(1, 2, "test", std::slice::from_ref(&track), 2)
            .await?;
        assert!(
            db.save_playlist(1, 2, "test", &[track.clone(), track.clone()], 1)
                .await
                .is_err()
        );
        assert_eq!(db.playlist(1, 2, "test").await?.len(), 1);
        assert!(db.delete_playlist(1, 2, "test").await?);
        assert!(db.playlist(1, 2, "test").await.is_err());
        let reminder = db
            .add_reminder(Some(1), 2, 3, "repeat", now_ms() + 60_000, Some(60))
            .await?;
        let reminders = db.reminders(Some(3), None).await?;
        assert_eq!(reminders[0].interval_seconds, Some(60));
        assert!(
            db.update_reminder(reminder, 3, now_ms() + 120_000, Some("edited"))
                .await?
        );
        assert!(db.cancel_reminder(reminder, 3).await?);
        let bookmark = db
            .add_bookmark(3, Some(1), 2, 4, 5, "useful message")
            .await?;
        assert_eq!(db.bookmarks(3, Some("useful")).await?.len(), 1);
        assert!(db.note_bookmark(bookmark, 3, "reference").await?);
        assert_eq!(db.bookmarks(3, Some("reference")).await?.len(), 1);
        assert!(db.delete_bookmark(bookmark, 3).await?);
        db.close().await;
        Ok(())
    }
}
