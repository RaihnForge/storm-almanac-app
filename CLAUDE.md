# Storm Almanac — HotS Replay Uploader & Talent Sync

## What It Does

Desktop companion app for Heroes of the Storm. Automatically uploads replay files to hots.lightster.ninja and syncs talent builds to the game client. Includes optional input session recording during gameplay.

Forked from lightster/storm-almanac-app.

## Stack

- **Frontend:** SvelteKit 5 + Svelte 5, Tailwind CSS 4.2, adapter-static (SPA mode)
- **Backend:** Rust (Tauri 2.0), Tokio async runtime
- **Build:** Vite 6, Cargo (Rust)
- **Desktop:** Tauri 2.0 — tray app, deep links, auto-update
- **No port** — desktop app, not a web service

**Note:** This project uses a different stack from the ecosystem standard (JS/HTML/CSS). It has external dependencies (npm + Cargo).

## Running

```sh
npm install
npm run tauri dev
```

Requires Node.js 24+ and Rust toolchain.

## Architecture

| Component | File(s) | Role |
|-----------|---------|------|
| App setup | `src-tauri/src/lib.rs` | Tauri app, tray menu, window management, IPC commands |
| File watcher | `src-tauri/src/watcher.rs` | Monitors replay directory, orchestrates uploads |
| Uploader | `src-tauri/src/uploader.rs` | HTTP multipart POST to backend API |
| Game session | `src-tauri/src/game_session.rs` | Polls for game process, manages input recording |
| Input recorder | `src-tauri/src/input_recorder/` | Platform-specific keyboard/mouse capture (macOS/Windows) |
| Config | `src-tauri/src/config.rs` | Settings persistence via Tauri plugin-store |
| State | `src-tauri/src/state.rs` | Shared app state (uploads, hashes, status) |
| Home page | `src/routes/+page.svelte` | Minimal status display |
| Settings UI | `src/routes/settings/+page.svelte` | Watch folder, autostart, input recording toggles |

## Key Data Flows

1. **Replay upload:** File watcher detects `.StormReplay` → SHA256 dedup → multipart POST → retry on failure (exponential backoff, max 5)
2. **Talent sync:** Read/write `TalentBuilds.txt` in game directory (backup on first write)
3. **Input recording:** Game detected → OS-level hooks capture events → JSONL → gzip → upload on game close

## External APIs

| Endpoint | Method | Purpose |
|----------|--------|---------|
| `{API_URL}/api/replays/upload` | POST | Upload .StormReplay file |
| `{API_URL}/api/input-sessions/upload` | POST | Upload compressed input session |

Default API_URL: `https://hots.lightster.ninja`

## Ownership

Sovereign (forked). Original author: lightster. Improvement candidates welcome.

## Workflow Rules

- Complete ALL tiers/phases before moving on. Do not skip ahead or leave phases partially done.
- Always update KPSP-Shard.md (Backlog section) and ARCHITECTURE.md after implementing features.
- Verify you are editing files in the CORRECT project directory before making changes.
