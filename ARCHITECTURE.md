# Storm Almanac — Architecture

## Overview

Tauri 2.0 desktop app with SvelteKit frontend and Rust backend. Runs as a tray application that monitors a replay directory and uploads files to a remote API. SPA mode — no SSR.

## Component Map

```
storm-almanac-app/
├── src/                          # SvelteKit frontend (SPA)
│   ├── routes/+page.svelte       # Home — status display
│   ├── routes/settings/          # Settings UI
│   ├── app.css                   # Tailwind + custom dark theme
│   └── app.html                  # HTML shell
│
├── src-tauri/                    # Rust backend
│   ├── src/lib.rs                # App setup, tray, windows, IPC commands
│   ├── src/watcher.rs            # File system watcher + upload orchestration
│   ├── src/uploader.rs           # HTTP multipart upload client
│   ├── src/game_session.rs       # Game process polling + input session lifecycle
│   ├── src/input_recorder/       # Platform input hooks (macOS CGEventTap, Win32)
│   ├── src/config.rs             # Persistent config via plugin-store
│   ├── src/state.rs              # Shared state (Mutex<AppState>)
│   ├── tauri.conf.json           # Production config
│   └── capabilities/             # Tauri security scopes
│
├── vite.config.js                # Vite dev server on :1420 for Tauri
├── svelte.config.js              # adapter-static, SSR disabled
└── package.json                  # SvelteKit + Tailwind deps
```

## Data Flow

```
.StormReplay file created
    ↓
File watcher (notify crate, 2s debounce)
    ↓
Stability check → SHA256 hash → dedup against known_hashes
    ↓
Upload queue (semaphore: max 5 concurrent)
    ↓
POST multipart → hots.lightster.ninja/api/replays/upload
    ↓
Response: queued | duplicate | error (retry w/ exponential backoff)
    ↓
State update → emit to frontend watchers → persist to store
```

## State Management

- `AppState` — Mutex-wrapped, shared across async tasks
- `UploadEntry` — per-file status (Pending → Uploading → Queued/Duplicate/Error)
- Persistent store: `~/.config/storm-almanac/storm-almanac.json` (config, history, known hashes)

## Platform-Specific

| Feature | macOS | Windows |
|---------|-------|---------|
| Game detection | `pgrep -f Heroes.app` | `tasklist` filter |
| Input recording | CGEventTap (Accessibility) | SetWindowsHookEx |
| Autostart | Tauri plugin | Tauri plugin + winreg |
| Default replay dir | `~/Library/Application Support/Blizzard/...` | `~/Documents/Heroes of the Storm/...` |

## Ecosystem Position

- **Type:** Product (shipped desktop app, forked)
- **Port:** None — desktop app
- **Dependencies:** npm + Cargo (external, not stdlib-only)
- **Registered in:** valinor/projects.json, sanctum manifest

## Document Classification (CON/MON/EVD)

| Document | Type | Purpose |
|----------|------|---------|
| CLAUDE.md | CON | AI instructions and project rules |
| ARCHITECTURE.md | CON | System design and data flow |
| README.md | EVD | User-facing project overview |
| KPSP-Shard.md | CON+MON | Project state and backlog |
| MEMORY.md | MON | Cross-session AI memory |
| keeper.md | MON | CTO command channel |
