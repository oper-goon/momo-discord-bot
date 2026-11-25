CREATE TABLE module_channels (
    guild_id TEXT NOT NULL,
    module_id TEXT NOT NULL,
    channel_id TEXT NOT NULL,
    PRIMARY KEY (guild_id, module_id, channel_id)
);
