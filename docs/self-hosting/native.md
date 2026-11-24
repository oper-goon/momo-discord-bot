# Native installation

Native operation is supported for development, but requires you to maintain external tools yourself. Docker is easier for most operators.

Use the Rust version pinned in `rust-toolchain.toml`. On Debian/Ubuntu, install build dependencies `cmake`, `pkg-config`, `libopus-dev`, and `libssl-dev`; the runtime needs CA certificates, OpenSSL, libopus, Python, FFmpeg/FFprobe, yt-dlp and Deno for media features. Consult the checked-in Dockerfile for the exact pinned media-tool versions. YouTube download resolution uses a PO-token provider at the internal hostname `pot-provider:4416`; a standalone native host does not automatically provide that service. Prefer Compose when using that path.

```sh
cp config/example.toml config/config.toml
# Edit command registration mode and guild IDs before continuing.
cargo build --locked --release -p bot-app
./target/release/bot-app --config config/config.toml --check
# Export DISCORD_TOKEN privately in this shell before the next command.
./target/release/bot-app --config config/config.toml
```

The program does not load `.env` itself. Set environment variables through your shell or service manager without placing secrets in command history. Unlike Compose, native operation defaults to `sqlite://data/bot.db`, relative to the working directory. The default data directory is created automatically; custom database parent directories must already exist and be writable.

On Windows the repository wrapper configures the expected local Rust/LLVM/CMake/Ninja tools when installed:

```powershell
./scripts/dev.ps1 --% build --locked -p bot-app
./scripts/dev.ps1 --% run --locked -p bot-app -- --config config/config.toml --check
```

Native dependency installation and a Windows media runtime have not been newly validated in this initiative. There is no supplied systemd service installer; if you create one, use a dedicated unprivileged account, a fixed working directory, private environment file, persistent data directory, and graceful SIGTERM shutdown.
