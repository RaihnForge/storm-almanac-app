# Implementation Plan: Talent Build Sync (Feature 2)

## Context

Storm Uploader's webview loads hots.lightster.ninja where users save talent builds. This feature syncs those builds into the game's `TalentBuilds.txt` file so they appear as favorites in-game. The desktop app acts as a file I/O bridge — all build logic and encoding lives server-side in the hotsds API, keeping it updatable without app releases.

**Data flow:**
1. Website calls `invoke('read_talent_builds')` -> Rust reads TalentBuilds.txt from disk
2. Website POSTs contents to `/api/builds/talent-file` -> API merges user's saved builds, returns new file
3. Website calls `invoke('write_talent_builds', { contents })` -> Rust writes file back

## Step 1: Tauri commands (storm-uploader)

**File: `src-tauri/src/lib.rs`**

Replace `save_build` stub with two new commands:

### `read_talent_builds`
- Load config to get `watch_dir` (Accounts directory)
- Iterate immediate subdirectories (account ID folders like `305721663`)
- Look for `TalentBuilds.txt` in each -- pick most recently modified if multiple
- Return file contents as `String`, or empty string if not found

### `write_talent_builds(contents: String)`
- Find TalentBuilds.txt same way as read (or create in first account dir found)
- Write contents to file
- Return `Ok(())`

### Register and clean up
- Add both to `generate_handler!`
- Remove `save_build` stub (replaced by the sync flow)
- Keep `load_overlay` stub (Feature 4) and `open_uploads`

### Helper: `find_talent_builds_path(watch_dir: &str) -> Option<PathBuf>`
- Shared between read and write
- Iterates `watch_dir` subdirs, returns path to most recent TalentBuilds.txt
- For write when no file exists: returns path in first account subdir found

## Step 2: Add internal hero names to database (hotsds)

### 2a. Add `HERO_INTERNAL_NAMES` to `src/hero_data.py`

Add a dict mapping display name -> TalentBuilds.txt internal name. Only heroes where the names differ need entries (~28 heroes). Heroes not in the dict use their display name as-is.

### 2b. Add `internal_name` column via migration

**File: `migrations/20260321200000_AddInternalName.lua`**

### 2c. Populate `internal_name` in `generate_derived_tables.py`

In the `generate_heroes` function, add `internal_name` to the DataFrame alongside `portrait_key`.

## Step 3: API endpoint (hotsds)

**File: `app/src/routes/api/builds/talent-file/+server.js`**

### `POST /api/builds/talent-file`

**Request:** `{ contents: "existing TalentBuilds.txt contents" }`

**Logic:**
1. Auth check
2. Query all user's saved builds, talent tree, and hero mapping
3. Load hash JSON (`docs/talent_build_hashes.json`)
4. Group builds by hero_slug (max 3 per hero = 3 build slots)
5. Encode each talent choice sort_order to bitmask hex: 0->"01", 1->"02", 2->"04", 3->"08", null->"00"
6. Generate line: `InternalName=BuildName|slot1|slot2|slot3|hash`
7. Parse existing file contents -- preserve lines for heroes without saved builds
8. Merge: replace lines for heroes with builds, keep the rest

## Step 4: Website Tauri integration (hotsds)

### Tauri utility: `app/src/lib/tauri.js`
### Sync button: `app/src/routes/play/builds/[slug]/+page.svelte`

Add a "Sync Builds to Game" button visible only in Tauri webview.

## Files Summary

### storm-uploader (modified)
- `src-tauri/src/lib.rs` - Add `read_talent_builds`, `write_talent_builds` commands; remove `save_build` stub

### hotsds (created)
- `migrations/20260321200000_AddInternalName.lua` - Add `internal_name` column
- `app/src/routes/api/builds/talent-file/+server.js` - Merge endpoint
- `app/src/lib/tauri.js` - Tauri detection + invoke wrapper

### hotsds (modified)
- `src/hero_data.py` - Add `HERO_INTERNAL_NAMES` dict
- `scripts/generate_derived_tables.py` - Populate `internal_name` column
- `app/src/routes/play/builds/[slug]/+page.svelte` - Add sync button
