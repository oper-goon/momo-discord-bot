CREATE TABLE guild_settings (
    guild_id              TEXT PRIMARY KEY,
    prefix                TEXT NOT NULL,
    prefix_enabled        INTEGER NOT NULL,
    locale                TEXT,
    created_at_ms         INTEGER NOT NULL,
    updated_at_ms         INTEGER NOT NULL
);

CREATE TABLE guild_modules (
    guild_id              TEXT NOT NULL,
    module_id             TEXT NOT NULL,
    enabled               INTEGER NOT NULL,
    updated_by_user_id    TEXT NOT NULL,
    updated_at_ms         INTEGER NOT NULL,
    PRIMARY KEY (guild_id, module_id)
);

CREATE TABLE permission_rules (
    id                    INTEGER PRIMARY KEY,
    guild_id              TEXT NOT NULL,
    target_path           TEXT NOT NULL,
    subject_kind          TEXT NOT NULL,
    subject_id            TEXT,
    channel_id            TEXT,
    effect                TEXT NOT NULL,
    created_by_user_id    TEXT NOT NULL,
    created_at_ms         INTEGER NOT NULL
);

CREATE TABLE moderation_cases (
    guild_id              TEXT NOT NULL,
    case_number           INTEGER NOT NULL,
    action                TEXT NOT NULL,
    target_user_id        TEXT NOT NULL,
    moderator_user_id     TEXT NOT NULL,
    reason                TEXT,
    status                TEXT NOT NULL,
    created_at_ms         INTEGER NOT NULL,
    expires_at_ms         INTEGER,
    PRIMARY KEY (guild_id, case_number)
);

CREATE TABLE reminders (
    id                    INTEGER PRIMARY KEY,
    guild_id              TEXT,
    channel_id            TEXT,
    user_id               TEXT NOT NULL,
    message               TEXT NOT NULL,
    due_at_ms             INTEGER NOT NULL,
    delivered_at_ms       INTEGER,
    failure_count         INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE music_settings (
    guild_id                         TEXT PRIMARY KEY,
    dj_role_id                       TEXT,
    default_volume_percent           INTEGER NOT NULL,
    max_queue_length                 INTEGER NOT NULL,
    idle_disconnect_seconds          INTEGER NOT NULL,
    empty_channel_disconnect_seconds INTEGER NOT NULL,
    announce_now_playing             INTEGER NOT NULL
);

CREATE TABLE music_playlists (
    id                    TEXT PRIMARY KEY,
    guild_id              TEXT NOT NULL,
    owner_user_id         TEXT NOT NULL,
    name                  TEXT NOT NULL,
    created_at_ms         INTEGER NOT NULL,
    updated_at_ms         INTEGER NOT NULL,
    UNIQUE (guild_id, owner_user_id, name)
);

CREATE TABLE music_playlist_tracks (
    playlist_id           TEXT NOT NULL,
    position              INTEGER NOT NULL,
    provider              TEXT NOT NULL,
    provider_item_id      TEXT,
    canonical_url         TEXT,
    title                 TEXT NOT NULL,
    duration_ms           INTEGER,
    metadata_json         TEXT,
    PRIMARY KEY (playlist_id, position),
    FOREIGN KEY (playlist_id) REFERENCES music_playlists(id) ON DELETE CASCADE
);

CREATE INDEX permission_lookup ON permission_rules(guild_id, target_path); CREATE INDEX reminders_due ON reminders(due_at_ms) WHERE delivered_at_ms IS NULL; CREATE INDEX moderation_history ON moderation_cases(guild_id, target_user_id); CREATE TABLE command_sync (scope TEXT PRIMARY KEY, hash TEXT NOT NULL);
