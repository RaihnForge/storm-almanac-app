# Overlay POC: Two Floating Windows (Interactive + Click-Through)

## Context

Storm Almanac is a tray-only Heroes of the Storm companion app (Tauri 2 + SvelteKit). A `load_overlay()` Tauri command stub already exists in `src-tauri/src/lib.rs:354-358` but is unused. The user wants to **explore what's technically possible** with overlays before committing to a real feature — specifically, see two coexisting overlay windows on screen at once: one that's **interactive** (drag, click) and one that's **click-through** (events pass through to whatever's underneath). This proves out transparency, always-on-top, and per-window cursor-event control as a foundation for any future real overlay (game-aware HUD, recording indicator, talent build hints, etc).

This is a throwaway-friendly POC, not a shipped feature. The bar is "see it work, learn the caveats, leave clean hooks for the real thing later."

## Approach

Two new dynamically-created Tauri windows, both transparent / frameless / always-on-top, opened together via a new tray menu entry. They share a single SvelteKit route distinguished by a query param so the Svelte code stays small.

**Window A — interactive** (label `overlay-interactive`)
- ~280×140, semi-transparent rounded card
- Has a `data-tauri-drag-region` so the user can grab and reposition it
- A counter button to prove click events reach the webview
- A close (×) button that closes only this overlay

**Window B — click-through** (label `overlay-clickthrough`)
- Same dimensions, visually distinct (different accent color + label)
- Calls `getCurrentWindow().setIgnoreCursorEvents(true)` on mount, so the mouse passes straight through to whatever's underneath
- Because it can't be clicked, it's only closable via the tray toggle

**Trigger:** new tray menu entry **"Toggle Overlay POC"**, placed above the existing **Quit** item. First click opens both overlays; second click closes both. Idempotent — "open" is a no-op if a window with that label already exists, and likewise for "close."

**Default positions:** offset slightly so the overlays don't stack — e.g. interactive at (200, 200), click-through at (520, 200) in screen coordinates. Hard-coded for the POC; multi-monitor sanity is out of scope.

## Files to create / modify

**Create**
- `src/routes/overlay/+page.svelte` — single Svelte route. Reads `?mode=interactive|clickthrough` from the URL. Conditionally calls `setIgnoreCursorEvents(true)`. Renders one of two visual variants.
- `src-tauri/capabilities/overlay.json` — new capability scoped to the two overlay window labels. Grants `core:default`, `core:window:default`, `core:window:allow-set-ignore-cursor-events`, `core:window:allow-start-dragging`, `core:webview:default`.

**Modify**
- `src-tauri/src/lib.rs`
  - Add two label constants: `OVERLAY_INTERACTIVE_LABEL`, `OVERLAY_CLICKTHROUGH_LABEL`.
  - Add `open_overlay_pair(app)` and `close_overlay_pair(app)` helper fns following the pattern of `open_settings_window` (`lib.rs:169-207`). Both use `WebviewWindowBuilder` with `WebviewUrl::App("/overlay?mode=...".into())`, `.transparent(true)`, `.decorations(false)`, `.always_on_top(true)`, `.skip_taskbar(true)`, `.resizable(false)`, `.shadow(false)`.
  - Add a tray menu item `toggle_overlay` and handle it in the existing tray `on_menu_event` handler — toggle based on whether either overlay window is currently present.
  - Replace the existing `load_overlay` stub with a Tauri command `toggle_overlay_pair` that does the same toggle (handy for invoking from devtools / future UI). Update the `invoke_handler` registration accordingly.
  - On macOS, do **not** flip `ActivationPolicy` for these windows — they should stay accessory-style so the dock doesn't bounce.

- `src-tauri/tauri.conf.json` — no changes needed; windows are created dynamically.

## Reused patterns / functions

- Window creation: mirror `open_settings_window` in `src-tauri/src/lib.rs:169-207`. Same `WebviewWindowBuilder` chain, just with the overlay-specific flags.
- Tray menu wiring: extend the existing `MenuBuilder` chain and the `on_menu_event` match arms in `lib.rs:run()`. The existing pattern handles items like `open_settings`, `quit`, etc.
- Frontend Tauri API: import `getCurrentWindow` from `@tauri-apps/api/window` (already a transitive dep via `@tauri-apps/api`). Use `window.startDragging()` if `data-tauri-drag-region` proves flaky.

## Caveats to surface in the POC (documented in code comments and verification notes)

- **macOS:** transparent always-on-top windows float above normal apps but **not** above fullscreen apps by default. Showing over a fullscreen game would require `setVisibleOnAllWorkspaces(true)` plus NSWindow level adjustments — explicitly out of scope for this POC.
- **Windows:** overlays render fine over borderless-windowed games (HoTS's default). Exclusive-fullscreen games may hide them. Out of scope.
- **Click-through window has no manual escape hatch.** If the tray menu is also somehow inaccessible, the user has to quit the app. Acceptable for POC; a real feature would add a hotkey or a peripheral close affordance.
- **Game detection is not wired in.** Overlays appear whenever toggled, regardless of whether HoTS is running.

## Verification

1. `npm run tauri dev` from the repo root.
2. From the tray menu, click **Toggle Overlay POC**. Confirm two distinct frameless transparent windows appear at roughly (200,200) and (520,200).
3. **Interactive window:** drag it to a new position, click the counter button several times — it should increment, and the window should retain focus / cursor behavior.
4. **Click-through window:** open Finder/Explorer or another app underneath it. Try clicking on the desktop / underlying app *through* the click-through overlay — clicks should land on the underlying surface, not the overlay.
5. Click **Toggle Overlay POC** again — both overlays close.
6. Toggle a few more times to confirm idempotence (no duplicate windows, no orphans).
7. Macros to log into `~/Library/Logs/...` via `tauri-plugin-log` — check `log::info!` calls fire on open/close.
8. macOS only: with overlays open, switch Spaces / fullscreen another app — note (don't fix) where they hide. Confirms the documented caveat.

## What this proves and what's next

**Proven:** transparent + always-on-top windows; `setIgnoreCursorEvents` on a per-window basis; two different interaction modes coexisting; a clean spawn/teardown lifecycle from tray.

**Natural follow-ups (not in this POC):** wire to existing game detection so overlays auto-show on HoTS launch; replace placeholder UI with talent build hints or recording indicator; add fullscreen-game support; configurable position / persisted across sessions; hotkey to toggle.
