ALTER TABLE reminders ADD COLUMN interval_seconds INTEGER;

CREATE TABLE bookmarks (
    id                    INTEGER PRIMARY KEY,
    user_id               TEXT NOT NULL,
    guild_id              TEXT,
    channel_id            TEXT NOT NULL,
    message_id            TEXT NOT NULL,
    author_user_id        TEXT NOT NULL,
    excerpt               TEXT NOT NULL,
    note                  TEXT,
    created_at_ms         INTEGER NOT NULL,
    UNIQUE(user_id, message_id)
);

CREATE INDEX bookmarks_user_created ON bookmarks(user_id, created_at_ms DESC);
