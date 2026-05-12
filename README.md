# Storm Almanac

A desktop companion app for [hots.lightster.ninja](https://hots.lightster.ninja). Automatically uploads Heroes of the Storm replay files and syncs talent builds to your game client.

Built with [Tauri](https://v2.tauri.app/), [SvelteKit](https://svelte.dev/), and Rust.

## Downloads

- [macOS (Apple Silicon)](https://github.com/lightster/storm-uploader/releases/latest/download/Storm-Almanac_darwin-aarch64.dmg)
- [macOS (Intel)](https://github.com/lightster/storm-uploader/releases/latest/download/Storm-Almanac_darwin-x64.dmg)
- [Windows (installer)](https://github.com/lightster/storm-uploader/releases/latest/download/Storm-Almanac_windows-x64-setup.exe)

> This app is not code-signed. I built it for my own use and it's not
> currently worth the cost of a signing certificate, but I want to make
> it available for others who find it useful. Because it's unsigned,
> your OS will warn you before running it.

**macOS:** After installing the app to your Applications folder, remove the quarantine flag before opening:

```bash
xattr -cr /Applications/Storm\ Almanac.app
```

**Windows:** If SmartScreen warns about an unrecognized app, click "More info" then "Run anyway".

## Development

### Prerequisites

- [Node.js](https://nodejs.org/)
- [Rust](https://www.rust-lang.org/tools/install)
- Tauri CLI: `cargo install tauri-cli --version "^2"`

Optional (for editor/IDE code intelligence on the Rust side, including the Claude Code rust-analyzer plugin enabled in `.claude/settings.json`):

- rust-analyzer: `rustup component add rust-analyzer` (or `brew install rust-analyzer`)

### Setup

```bash
npm install
npm run tauri dev
```

### Build

```bash
npm run tauri build
```

## License

MIT
