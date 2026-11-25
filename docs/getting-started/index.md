# Getting started

Momokun is the public project name; the source, executable, and volume names still use Momo. You host your own bot and keep control of its credentials and SQLite database. No website or inbound HTTP port is required.

## Before you begin

You need a Discord account, permission to add a bot to your server, Git, and Docker with the Compose plugin. Docker Desktop includes Compose on Windows and macOS; on Linux install Docker Engine and its Compose plugin. Start Docker before continuing.

The current release is pre-1.0. The Linux x86_64 candidate passed isolated container startup, persistence, backup and restoration checks, plus a focused live YouTube voice trial. ARM64 support is unverified. No measured minimum RAM/CPU specification exists: source compilation needs substantially more memory and disk than running the bot. Monitor your host, especially when downloading or converting media.

Only Discord credentials are required. Easynews and TorBox need separate paid service accounts when enabled; movie/TV lookup needs TMDB credentials. YouTube, AniList, and the other credential-free integrations still need working outbound Internet access.

## Create your Discord bot

1. Open the [Discord Developer Portal](https://discord.com/developers/applications), create an application, and open its **Bot** settings.
2. Generate or reset the bot token. Save it privately; it gives control of your bot. Never paste it into a public issue, screenshot, or TOML file.
3. Enable **Message Content Intent**. The current application requests the Social Embeds module's intents at startup even when that module is disabled for a server. Prefix commands also require this intent. The application requests Guilds, Guild Messages, Message Content, and Guild Voice States; enabling prefixes adds Direct Messages. It does not request the privileged Guild Members or Presence intents. Slash commands do not require typing a prefix.
4. In the installation/OAuth2 URL generator select **bot** and **applications.commands** scopes. Select permissions appropriate to the enabled features, then open the generated URL and choose your server. You need authorization to manage that server.

Start with View Channels, Send Messages, Embed Links, and Read Message History for ordinary commands. Downloads need Attach Files; music needs Connect and Speak in the voice channel. Moderation needs the corresponding action permissions, and the bot's role must be above roles it manages. Social Embeds needs Manage Messages and Manage Webhooks as well. Avoid granting Administrator simply to hide a permission problem.

There is no authorized official hosted-bot invitation URL in this repository. The URL you generate installs your own bot.

## Install and verify

Follow the [Docker quick start](../self-hosting/index.md). After startup, run `/ping` and `/help` in your server. Core, General, and Utility are enabled by default. A server manager can enable an optional feature with `/module enable music`, for example. Optional credentials alone do not enable a module in Discord.

If commands are absent, confirm the invitation scopes, command registration mode, server ID in guild mode, and successful startup logs. Global command updates may take time to appear; close and reopen Discord before forcing command synchronization.

