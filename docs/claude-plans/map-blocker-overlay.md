# Map Blocker Overlay (game-foreground-aware, hotkey-toggled mode)

## Context

The overlay POC (interactive + click-through cards) shipped in v0.1.32 proved that transparent always-on-top windows over HoTS work fine in Windowed Full Screen mode. Now we want the first **real** overlay feature: a transparent window the user positions over the HoTS minimap that silently absorbs misclicks (so accidentally clicking the map can't issue a "move camera" / "right-click move command" by mistake). The mechanism is just "another window in front of the game catches the click before the game sees it" — no `SendInput`, no hooks, no DLL injection. This puts the feature in the same risk category as the existing POC (i.e. essentially zero anti-cheat exposure).

Two pieces of polish make this usable:
1. **Auto show/hide tied to HoTS focus.** Show the blocker only when HoTS is the foreground window; hide it when the user alt-tabs to a browser, Discord, etc., so the blocker isn't in the way during non-game time.
2. **Hotkey-toggled "interactable mode."** A global hotkey switches the window between blocking mode (transparent, locked, absorbs clicks) and interactable mode (decorated, resizable, draggable) so the user can position/resize the rect to fit their minimap. Geometry persists across runs.

The existing POC pair (`overlay-interactive`, `overlay-clickthrough`) stays in place — it's a useful demo and reference. The blocker is a sibling, not a replacement.

**Forward-looking note (not in scope for this task):** the same window-plumbing decisions made here will apply to a future "web content overlay" — a translucent always-on-top window whose webview points at a Storm Almanac page (e.g. `WebviewUrl::External("https://hots.lightster.ninja/overlay/talents?...")` or a local Svelte route that pulls live data). The Tauri side (`transparent: true`, `macos-private-api` already enabled) is already in place; the only thing that pattern needs that this task doesn't is for the rendered page to ship CSS with `body { background: transparent }` and opaque content cards. Frosted-glass backdrop blur of the desktop behind the window is a separate, additive piece (needs the `window-vibrancy` crate). Calling this out so the blocker's design doesn't accidentally box us out of that direction.

## Approach

Add a third overlay variant `mode=blocker` to the existing `/overlay` SvelteKit route. A new tray entry **"Enable Map Blocker"** turns the feature on/off. When enabled, a foreground-aware poller shows the blocker window when HoTS is the focused window and hides it otherwise. A global hotkey (default `CmdOrCtrl+Shift+B`) toggles the visible blocker between two visual/interaction modes.

### State model

The blocker window has three logical states:

| State | When | Window visibility | Decorations | Resizable | Catches clicks |
|---|---|---|---|---|---|
| Disabled | User toggled feature off | Closed | n/a | n/a | n/a |
| Blocking (game focused) | Enabled + HoTS is foreground + not in interactable mode | Visible, transparent | None | No | Yes |
| Blocking (game not focused) | Enabled + HoTS not foreground + not in interactable mode | Hidden | n/a | n/a | n/a |
| Interactable | Hotkey toggled it on (overrides foreground check) | Visible, opaque-ish, with title bar | Yes | Yes | Yes (irrelevant — game isn't focused) |

Pressing the hotkey while in interactable mode → back to blocking, foreground-driven visibility resumes.
Pressing the hotkey while in blocking mode → switch to interactable, force-show window even if HoTS isn't foreground (so the user can configure when alt-tabbed out).

### Window plumbing

Single window labeled `overlay-blocker`, created on demand when the feature is enabled. Reuses the existing `WebviewWindowBuilder` pattern from `open_overlay_window` in `src-tauri/src/lib.rs:214-235`, but:
- `.transparent(true)`, `.always_on_top(true)`, `.skip_taskbar(true)`, `.shadow(false)` — same as existing overlays.
- `.decorations(false)`, `.resizable(false)` at creation (blocking mode is the default).
- `.position(x, y)` and `.inner_size(w, h)` from persisted store (with sensible defaults — bottom-left quadrant of primary monitor at ~250×200, where HoTS minimap typically sits).
- `WebviewUrl::App("/overlay?mode=blocker".into())`.

Runtime mutation via `WebviewWindow::set_decorations(bool)` and `set_resizable(bool)` — methods exist on Tauri 2 webview windows but aren't currently used anywhere in this codebase, so I'll start with the simplest approach (toggle decorations + resizable on hotkey) and fall back to custom HTML/CSS resize handles only if runtime decoration toggling proves flaky on macOS.

Listen for `WindowEvent::Moved` and `WindowEvent::Resized` via `on_window_event` (same closure pattern used in `open_settings_window` at `src-tauri/src/lib.rs:194-205`) to persist geometry on change. Debounce saves with a short timer (~500ms) so a drag doesn't hammer the store.

### Foreground detection

New module `src-tauri/src/game_focus.rs`. Polls every 500ms via a dedicated `tokio::spawn` (separate from the existing 5s `start_game_session_polling` in `game_session.rs`, since 5s is too slow for UI-feeling auto-show/hide).

- **Windows:** `windows` crate (already a dep). `GetForegroundWindow()` → HWND. `GetWindowThreadProcessId(hwnd, &mut pid)` → PID. Compare against `HeroesOfTheStorm_x64.exe`'s PID, which we get by walking the process list (or, simpler and matching the existing pattern at `src-tauri/src/lib.rs:241-244`, by calling `tasklist` with the image filter and parsing the PID column). Cache the PID and re-resolve only when foreground PID isn't a match.
- **macOS:** use `NSWorkspace.sharedWorkspace.frontmostApplication.bundleIdentifier` via the `objc2`/`objc2-app-kit` bridge — or, since macOS HoTS isn't realistically in scope for this feature, accept "always treat HoTS as foreground when running" as the macOS fallback (matches what `is_game_running()` already does on macOS).

The poller publishes a `bool is_game_focused` to a `tokio::sync::watch` channel. A subscriber task in `lib.rs` reacts to changes:
- Transition to focused + blocker enabled + not in interactable mode → `window.show()`
- Transition to not-focused + not in interactable mode → `window.hide()`

### Global hotkey

New deps:
- `src-tauri/Cargo.toml`: add `tauri-plugin-global-shortcut = "2"`.
- `package.json`: add `@tauri-apps/plugin-global-shortcut@^2`.
- Initialize the plugin in the `tauri::Builder` chain in `lib.rs:run()` next to the other plugins (around line 432–443).

Register `CmdOrCtrl+Shift+B` at startup. Handler toggles a `BlockerMode` enum (`Blocking | Interactable`) held in a shared `Arc<Mutex<BlockerState>>` and applies the visual change to the window:
- → `Interactable`: `set_decorations(true)`, `set_resizable(true)`, `show()`, set body class so CSS shows a visible fill + label "Map Blocker — drag/resize, Ctrl+Shift+B to lock"; emit a Tauri event `blocker://mode-changed` so the frontend can swap classes.
- → `Blocking`: `set_decorations(false)`, `set_resizable(false)`, body class flips to transparent; foreground-driven show/hide resumes.

The hotkey only does anything when the blocker is enabled (i.e. window exists). When disabled, hotkey presses are ignored.

### Persistence

Store namespace: `storm-almanac.json` (existing file used by `config::load_config`/`save_config` at `src-tauri/src/config.rs:50-65` via `app.store(STORE_FILE).get/set/save`). Use the existing `StoreExt` pattern.

New key `"map_blocker"`, value shape:
```json
{
  "enabled": true,
  "x": 1620,
  "y": 760,
  "width": 280,
  "height": 220
}
```

Loaded on startup. If `enabled` is true AND HoTS is foreground at startup, the blocker window is created and shown. If `enabled` is true but HoTS isn't foreground, the window is created but immediately hidden (or — to save resources — created lazily on first focus transition). I'll go lazy: only create the window when first needed, since this is also how the POC works.

### Tray menu

Add a checkable menu item **"Enable Map Blocker"** (Tauri's `CheckMenuItemBuilder`) above the existing `Toggle Overlay POC` entry. Its checked state mirrors the persisted `enabled` flag. Toggling it:
- On → set `enabled=true`, save to store, register hotkey, start foreground poller subscriber.
- Off → set `enabled=false`, save to store, unregister hotkey, close blocker window if open.

### Frontend

Extend `src/routes/overlay/+page.svelte`:
- Recognize `mode=blocker` as a third value alongside the existing two.
- In blocker mode, listen for `blocker://mode-changed` Tauri events to swap a CSS class on the root element between `.blocking` and `.interactable`.
  - **`.blocking`** (default visible state): transparent fill **+ a faint dashed reddish border** (~1px, `border: 1px dashed rgba(255, 100, 100, 0.35)`). Visible enough on close inspection that the user remembers the blocker is there; not so visible it obscures the minimap. **On `mousedown`**, pulse a soft red fill (`background: rgba(255, 80, 80, 0.15)` for ~200ms via a CSS keyframe animation triggered by toggling a `.flash` class). This is the killer "why isn't my click working" cue — every absorbed click flashes back at the user, immediately distinguishing "blocker ate it" from "game broken."
  - **`.interactable`**: semi-opaque dark fill, dashed pink border (more prominent than blocking-mode border), small label "Map Blocker — Ctrl+Shift+B to lock" rendered in a corner. The OS title bar (decorations on) handles drag/resize.
- No `setIgnoreCursorEvents` call — the blocker always wants clicks to land on it.
- No `data-tauri-drag-region` in the DOM by default; the OS title bar handles drag/resize when decorations are on in interactable mode.

### Capabilities

Update `src-tauri/capabilities/overlay.json`:
- Add `"overlay-blocker"` to the `windows` array.
- Add permissions: `core:window:allow-set-decorations`, `core:window:allow-set-resizable`, `core:window:allow-set-position`, `core:window:allow-set-size`, `core:window:allow-show`, `core:window:allow-hide` (only those that aren't already covered by `core:window:default`).
- New plugin permission: `global-shortcut:default` (the global-shortcut plugin's default permission).

## Files to create / modify

**Create**
- `src-tauri/src/game_focus.rs` — foreground-window poller, exposes `start_foreground_poller(app) -> watch::Receiver<bool>`.

**Modify**
- `src-tauri/src/lib.rs`
  - Add `mod game_focus;`.
  - Add `BLOCKER_LABEL`, default-geometry constants near existing `OVERLAY_*` constants (lines 77–80).
  - Add `BlockerMode` enum + `BlockerState { mode, enabled, geometry }` shared via `Arc<Mutex<…>>`, registered with `app.manage()` in `setup`.
  - Add `open_blocker_window`, `close_blocker_window`, `apply_blocker_mode`, `set_blocker_enabled` helpers. Reuse the `WebviewWindowBuilder` pattern from `open_overlay_window` (lines 214–235); reuse the window-event closure pattern from `open_settings_window` (lines 194–205) for geometry-change persistence.
  - Add tray menu entry "Enable Map Blocker" (`CheckMenuItemBuilder`) with handler that flips `enabled`.
  - In `setup`, kick off `game_focus::start_foreground_poller`, subscribe a task that responds to focus changes by showing/hiding the blocker.
  - Wire `tauri-plugin-global-shortcut` plugin and register `CmdOrCtrl+Shift+B` once the blocker is enabled (and unregister when disabled).
- `src-tauri/Cargo.toml` — add `tauri-plugin-global-shortcut = "2"`.
- `package.json` — add `@tauri-apps/plugin-global-shortcut`.
- `src-tauri/capabilities/overlay.json` — add `overlay-blocker` to windows, add the runtime mutation permissions and `global-shortcut:default`.
- `src/routes/overlay/+page.svelte` — add `mode === 'blocker'` branch with the two-state CSS and event listener.

## Reused patterns / functions

- Store read/write via `StoreExt`: pattern at `src-tauri/src/config.rs:50-65`.
- Window builder fluent chain: `open_overlay_window` at `src-tauri/src/lib.rs:214-235`.
- `on_window_event` move-closure pattern: `open_settings_window` at `src-tauri/src/lib.rs:194-205`.
- HoTS process detection by image name: `is_game_running` at `src-tauri/src/lib.rs:228-249` (the new foreground module piggybacks on the same `tasklist` invocation to resolve PID).
- Tauri command registration in `invoke_handler`: existing `toggle_overlay_pair` at the end of `lib.rs::run()`.

## Caveats and known unknowns

- **Runtime `set_decorations` toggling on macOS** is sometimes flaky — on some Tauri/wry versions the window has to be recreated. Will verify during development; if flaky, fallback is custom CSS resize handles + `window.startResizeDragging('SouthEast')` etc. Windows is the primary target and works reliably there.
- **Hotkey conflict**: `Ctrl+Shift+B` is unused by default HoTS keybindings, but the player may have rebound something. Not configurable in v1; if it conflicts the user can rebind in HoTS or we'll add a config in a follow-up.
- **Foreground polling at 500ms** introduces a half-second-or-less delay between alt-tabbing into HoTS and the blocker appearing. Snappy enough for this use case; if it ever feels laggy we can move to `SetWinEventHook` with `EVENT_SYSTEM_FOREGROUND` for event-driven detection on Windows.
- **Multi-monitor / DPI:** persisted x/y are absolute screen coordinates. If the user moves HoTS to a different monitor, the blocker stays at the saved coords until they reposition it. Acceptable; a future enhancement could anchor it relative to the game window's `GetWindowRect`.

## Verification

1. `npm install` (for new JS plugin), `cargo check` from `src-tauri/` (for new Rust plugin).
2. `npm run tauri:dev` from repo root.
3. From the tray menu, click **Enable Map Blocker**. The menu item should show as checked. Nothing visible yet (HoTS isn't running).
4. Launch HoTS (or any app with image name `HeroesOfTheStorm_x64.exe`; for testing, can rename a dummy exe). Once HoTS is the foreground window, the blocker should appear at its default position within ~500ms.
5. Alt-tab to a browser. Blocker should hide within ~500ms. Alt-tab back to HoTS — blocker reappears.
6. With HoTS focused and the blocker visible, click in the blocker rect. Confirm (a) the in-game cursor doesn't issue a move command (the click was absorbed), and (b) the blocker briefly flashes a soft red fill on the click — this is the visual confirmation that the blocker ate it. Click just outside the rect — the click reaches the game and the blocker doesn't flash.
7. Press `Ctrl+Shift+B`. Blocker should switch to interactable mode: title bar appears, body fills with a visible color, you can drag/resize via the OS chrome.
8. Drag and resize. Press `Ctrl+Shift+B` again. Window should return to transparent locked mode at the new position/size.
9. Quit the app and relaunch. Blocker should remember `enabled=true` and the new geometry; behavior in step 4 repeats with the new rect.
10. Toggle **Enable Map Blocker** off. Window closes; hotkey becomes a no-op. Toggle on — restored.
11. Confirm `~/Library/Logs/Storm Almanac/` (macOS) or the equivalent Windows log dir shows `log::info!` lines for show/hide/mode-change events (useful for debugging if anything goes sideways).
