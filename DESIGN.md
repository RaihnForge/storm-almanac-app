# Storm Almanac — Design Vision

> **Document Created:** 2026-04-03
> **Status:** Initial

## Vision

Play the game, forget the rest. Storm Almanac watches your replay directory, uploads files automatically, syncs talent builds, and stays out of the way. A tray app that does its job silently and reliably.

## Design Principles

1. **Invisible when working** — Tray app idiom. Runs in background, notification-driven. You interact with it through settings, not during gameplay.
2. **Dedup by default** — SHA256 hashing prevents duplicate uploads. Upload once, never again.
3. **Resilient delivery** — Exponential backoff retry on upload failure. If the server is down, queued replays wait.
4. **Platform-native** — Tauri 2.0 wraps SvelteKit frontend with Rust backend. macOS and Windows input recording via native APIs (CGEventTap / SetWindowsHookEx).

## UX Philosophy

### Tray App
Minimal visible UI. Icon in system tray. Right-click for status, settings, quit. Notifications for upload success/failure.

### Settings Panel
Home page shows status (watching directory, upload count, last sync). Settings for replay directory path, API endpoint, talent sync toggle, input recording toggle.

### File Watcher
Monitors replay directory with 2-second debounce. New `.StormReplay` files detected → hashed → uploaded. Status persisted in `~/.config/storm-almanac/storm-almanac.json`.

## Aesthetic Direction

- Dark theme via Tailwind CSS
- Minimal UI — status-focused, not feature-heavy
- Tray icon with state indication
- Notification-driven feedback
- Utilitarian — reliability over visual polish

## What Success Feels Like

You finish a ranked Heroes match. Before you queue the next one, a quiet notification: "Replay uploaded." You didn't think about it. You didn't click anything. Talent builds synced automatically. Three months later, every replay is on the server, no duplicates, no missed uploads. The app is the definition of boring infrastructure — and that's the highest compliment.
