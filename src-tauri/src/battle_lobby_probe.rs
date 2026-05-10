// Watches the OS temp directory for HoTS's *.battlelobby file (written when
// you enter a draft / matchmaking lobby) and pulls the first .s2ma cache
// hash out. That hash is a stable, unique identifier for the active
// battleground — so we use it as the lookup key for per-map blocker rects.
//
// As a debug aid the watcher also copies each file it sees to
// <app_log_dir>/battlelobby-dumps/ and logs a hash-extraction summary, so
// if extraction ever breaks we have evidence to inspect.

use notify::{EventKind, RecursiveMode, Watcher};
use std::path::{Path, PathBuf};
use tauri::Manager;

const HASH_LEN: usize = 64;

pub fn start(app: tauri::AppHandle) {
    let temp_dir = std::env::temp_dir();
    let dumps_dir = match dumps_dir(&app) {
        Some(p) => p,
        None => {
            log::error!("battlelobby probe: could not resolve app log dir");
            return;
        }
    };
    if let Err(e) = std::fs::create_dir_all(&dumps_dir) {
        log::error!(
            "battlelobby probe: failed to create dumps dir {:?}: {}",
            dumps_dir,
            e
        );
        return;
    }

    log::info!(
        "battlelobby probe: watching {:?} (dumps -> {:?})",
        temp_dir,
        dumps_dir
    );

    // Pick up any existing file at startup — typically the last game's lobby.
    process_existing_files(&app, &temp_dir, &dumps_dir);

    let app_clone = app.clone();
    std::thread::spawn(move || {
        let (tx, rx) = std::sync::mpsc::channel();
        let mut watcher = match notify::recommended_watcher(
            move |res: Result<notify::Event, notify::Error>| {
                let _ = tx.send(res);
            },
        ) {
            Ok(w) => w,
            Err(e) => {
                log::error!("battlelobby probe: watcher init failed: {}", e);
                return;
            }
        };

        if let Err(e) = watcher.watch(&temp_dir, RecursiveMode::NonRecursive) {
            log::error!(
                "battlelobby probe: failed to watch {:?}: {}",
                temp_dir,
                e
            );
            return;
        }

        while let Ok(res) = rx.recv() {
            let event = match res {
                Ok(ev) => ev,
                Err(e) => {
                    log::warn!("battlelobby probe: watch error: {}", e);
                    continue;
                }
            };
            if !matches!(
                event.kind,
                EventKind::Create(_) | EventKind::Modify(_)
            ) {
                continue;
            }
            for path in event.paths {
                if is_battlelobby_path(&path) {
                    handle_file(&app_clone, &path, &dumps_dir);
                }
            }
        }
    });
}

fn process_existing_files(app: &tauri::AppHandle, temp_dir: &Path, dumps_dir: &Path) {
    let entries = match std::fs::read_dir(temp_dir) {
        Ok(e) => e,
        Err(_) => return,
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if is_battlelobby_path(&path) {
            handle_file(app, &path, dumps_dir);
        }
    }
}

fn is_battlelobby_path(path: &Path) -> bool {
    path.extension().and_then(|s| s.to_str()) == Some("battlelobby")
}

fn handle_file(app: &tauri::AppHandle, path: &Path, dumps_dir: &Path) {
    log::info!("battlelobby probe: detected {:?}", path);

    // Brief delay so HoTS has time to finish writing.
    std::thread::sleep(std::time::Duration::from_millis(500));

    let bytes = match std::fs::read(path) {
        Ok(b) => b,
        Err(e) => {
            log::error!("battlelobby probe: read {:?} failed: {}", path, e);
            return;
        }
    };
    log::info!(
        "battlelobby probe: {:?} size = {} bytes",
        path,
        bytes.len()
    );

    let hash = extract_first_map_hash(&bytes);
    match &hash {
        Some(h) => log::info!("battlelobby probe: first .s2ma hash = {}", h),
        None => log::warn!("battlelobby probe: no .s2ma hash extracted from {:?}", path),
    }

    // Save a copy with a stamped name so we can compare across games if
    // anything goes weird.
    let stamp = chrono::Utc::now().format("%Y%m%d-%H%M%S");
    let original_name = path
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("unknown");
    let dump_path = dumps_dir.join(format!("{}-{}.bin", stamp, original_name));
    if let Err(e) = std::fs::write(&dump_path, &bytes) {
        log::error!("battlelobby probe: dump save failed: {}", e);
    }

    if let Some(hash) = hash {
        crate::on_active_map_changed(app, hash);
    }
}

/// Pull the 64-hex-char hash that immediately precedes the first ".s2ma" in
/// the file. HoTS writes a series of cache paths like
/// `C:\...\Cache\1f\1b\<hash>.s2ma` and the first one is consistently the
/// active battleground.
fn extract_first_map_hash(bytes: &[u8]) -> Option<String> {
    let needle = b".s2ma";
    let pos = bytes.windows(needle.len()).position(|w| w == needle)?;
    if pos < HASH_LEN {
        return None;
    }
    let candidate = &bytes[pos - HASH_LEN..pos];
    if !candidate.iter().all(|b| b.is_ascii_hexdigit()) {
        return None;
    }
    let s = std::str::from_utf8(candidate).ok()?;
    Some(s.to_ascii_lowercase())
}

fn dumps_dir(app: &tauri::AppHandle) -> Option<PathBuf> {
    app.path()
        .app_log_dir()
        .ok()
        .map(|d| d.join("battlelobby-dumps"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_hash_from_typical_path() {
        let sample = b"\x0b\x82C:\\ProgramData\\Blizzard Entertainment\\Battle.net\\Cache\\1f\\1b\\1f1b228ddb1f72205cbfd444055287100b0f39959be816548162e4081ea85511.s2ma extra junk";
        assert_eq!(
            extract_first_map_hash(sample).as_deref(),
            Some("1f1b228ddb1f72205cbfd444055287100b0f39959be816548162e4081ea85511")
        );
    }

    #[test]
    fn returns_none_when_no_s2ma_present() {
        assert!(extract_first_map_hash(b"some random bytes with no map ref").is_none());
    }
}
