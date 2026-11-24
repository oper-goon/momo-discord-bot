---
hide:
  - navigation
  - toc
---

<div class="momo-hero" markdown>
<span class="momo-eyebrow">Your server. Your rules. Your bot.</span>

# Momokun

<p class="momo-lead">Music, moderation, reminders, and media tools. A modular Discord bot that runs on hardware you control.</p>

<span class="momo-status">0.9.0 · pre-1.0 · public release preparation</span>

<div class="momo-actions" markdown>
[Quick start](getting-started/index.md){ .md-button .md-button--primary }
[Self-host with Docker](self-hosting/index.md){ .md-button }
[Command reference](reference/commands.md){ .md-button }
</div>
</div>

## A useful server starts small

Core controls, general information, and utilities are available by default.
Enable additional modules for each server when you need them. You do not need
an Easynews, TorBox, or TMDB account to run the bot's core features.

<div class="momo-grid" markdown>
<div class="momo-card" markdown>
### Music, together
Voice playback, queues, saved playlists, and DJ controls. YouTube, SoundCloud,
Bandcamp, and public audio links; optional account-backed providers.

[Explore music](features/music.md)
</div>
<div class="momo-card" markdown>
### Everyday tools
Persistent reminders, private bookmarks, polls, timestamps, and useful server information.

[Explore utilities](features/utility.md)
</div>
<div class="momo-card" markdown>
### Community controls
Moderation cases, role-aware administration, module activation, and command policies.

[Explore moderation](features/moderation.md)
</div>
<div class="momo-card" markdown>
### Media, made useful
Bounded downloads and conversions, title lookup, safe image search, and repaired social embeds.

[Explore all modules](features/index.md)
</div>
</div>

## Host it on your terms

Docker Compose runs the bot, a private YouTube token helper, and a persistent
SQLite volume. No public web port or separate database server is required.
This documentation is a static website hosted separately from the bot.

[Start the installation guide](getting-started/index.md) ·
[View the source on GitHub](https://github.com/oper-goon/momo-discord-bot)

!!! info "Release and visual identity"
    This is a pre-release project. External services and live Discord interactions
    have validation limits; read the feature pages before enabling optional modules.
    The owner has supplied the monochrome horned mascot and glitch wordmark as
    the approved visual reference. The preview uses text and a console icon until
    the original image files are added to the branding asset directory; no
    replacement character is being created. See [branding](project/branding.md).

No official shared-bot invitation is configured. [Create and invite your own bot](getting-started/index.md).
