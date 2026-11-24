# Frequently asked questions

## Does the documentation need my VPS?

No. GitHub Pages serves static documentation independently of the bot and its
database. It requires no production token or additional VPS service.

## Do I need every integration?

No. Discord credentials are needed to connect the bot. Paid providers and
feature-specific credentials are optional; enable only the modules you need.
Easynews and TorBox require their respective accounts. Missing optional media
credentials must not be treated as a reason to share someone else's account.

## Can I run it without Docker?

Yes, native Rust builds are supported, but Docker Compose is the recommended
self-hosting path because it supplies media tools and a consistent runtime.

## Is this already licensed as noncommercial source-available software?

Yes. This release candidate uses PolyForm Noncommercial 1.0.0 after the owner's
confirmation of rights. Earlier MIT grants and third-party terms remain valid. Read
[the licensing status](license.md) before redistributing.

## Are public images and the official bot invite available?

Publication requires owner approval. Follow the documented source-build method
until a tested image is actually published. No official invitation URL has been
provided; create and invite your own Discord application.
