// One-off diagnostic: watch %TEMP% for *.battlelobby files (HoTS writes one
// per draft/matchmaking lobby), then dump the file to the app log dir and
// log a list of ASCII-printable strings so we can eyeball where map names
// live in the binary. Remove or gate this behind a flag once we know the
// extraction pattern.

use notify::{EventKind, RecursiveMode, Watcher};
use std::path::{Path, PathBuf};
use tauri::Manager;

const PRINTABLE_MIN_LEN: usize = 4;
const MAX_LOGGED_STRINGS: usize = 400;

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
        "battlelobby probe: watching {:?} for *.battlelobby (dumps -> {:?})",
        temp_dir,
        dumps_dir
    );

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
                if path.extension().and_then(|s| s.to_str()) == Some("battlelobby") {
                    handle_battlelobby_file(&path, &dumps_dir);
                }
            }
        }
    });
}

fn handle_battlelobby_file(path: &Path, dumps_dir: &Path) {
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

    let stamp = chrono::Utc::now().format("%Y%m%d-%H%M%S");
    let original_name = path
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("unknown");
    let dump_path = dumps_dir.join(format!("{}-{}.bin", stamp, original_name));
    match std::fs::write(&dump_path, &bytes) {
        Ok(_) => log::info!("battlelobby probe: saved dump -> {:?}", dump_path),
        Err(e) => log::error!("battlelobby probe: dump save failed: {}", e),
    }

    let strings = extract_printable_strings(&bytes, PRINTABLE_MIN_LEN);
    log::info!(
        "battlelobby probe: {} printable strings (>= {} chars), logging up to {}",
        strings.len(),
        PRINTABLE_MIN_LEN,
        MAX_LOGGED_STRINGS
    );
    for (offset, s) in strings.iter().take(MAX_LOGGED_STRINGS) {
        log::info!("  @{:#010x}: {}", offset, s);
    }
}

fn extract_printable_strings(bytes: &[u8], min_len: usize) -> Vec<(usize, String)> {
    let mut results = Vec::new();
    let mut start: Option<usize> = None;
    let mut current = String::new();

    for (i, &b) in bytes.iter().enumerate() {
        let printable = (0x20..=0x7E).contains(&b);
        if printable {
            if start.is_none() {
                start = Some(i);
            }
            current.push(b as char);
        } else {
            if current.len() >= min_len {
                if let Some(s) = start {
                    results.push((s, std::mem::take(&mut current)));
                }
            } else {
                current.clear();
            }
            start = None;
        }
    }
    if current.len() >= min_len {
        if let Some(s) = start {
            results.push((s, current));
        }
    }
    results
}

fn dumps_dir(app: &tauri::AppHandle) -> Option<PathBuf> {
    app.path()
        .app_log_dir()
        .ok()
        .map(|d| d.join("battlelobby-dumps"))
}
