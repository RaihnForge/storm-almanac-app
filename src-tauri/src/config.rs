use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use tauri_plugin_store::StoreExt;

use crate::state::UploadEntry;

const STORE_FILE: &str = "storm-almanac.json";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppConfig {
    pub watch_dir: String,
    pub autostart: bool,
    #[serde(default)]
    pub start_minimized: bool,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            watch_dir: default_watch_dir(),
            autostart: false,
            start_minimized: false,
        }
    }
}

fn default_watch_dir() -> String {
    if cfg!(target_os = "macos") {
        if let Some(home) = dirs::home_dir() {
            return home
                .join("Library/Application Support/Blizzard/Heroes of the Storm/Accounts")
                .to_string_lossy()
                .to_string();
        }
    } else if cfg!(target_os = "windows") {
        if let Some(docs) = dirs::document_dir() {
            return docs
                .join("Heroes of the Storm/Accounts")
                .to_string_lossy()
                .to_string();
        }
    }
    String::new()
}

pub fn load_config(app: &tauri::AppHandle) -> AppConfig {
    let store = app.store(STORE_FILE).expect("failed to open store");

    match store.get("config") {
        Some(val) => serde_json::from_value(val).unwrap_or_default(),
        None => AppConfig::default(),
    }
}

pub fn save_config(app: &tauri::AppHandle, config: &AppConfig) {
    let store = app.store(STORE_FILE).expect("failed to open store");

    let val = serde_json::to_value(config).expect("failed to serialize config");
    store.set("config", val);
    let _ = store.save();
}

pub fn load_history(app: &tauri::AppHandle) -> Vec<UploadEntry> {
    let store = app.store(STORE_FILE).expect("failed to open store");

    match store.get("history") {
        Some(val) => serde_json::from_value(val).unwrap_or_default(),
        None => Vec::new(),
    }
}

pub fn save_history(app: &tauri::AppHandle, entries: &[UploadEntry]) {
    let store = app.store(STORE_FILE).expect("failed to open store");

    let val = serde_json::to_value(entries).expect("failed to serialize history");
    store.set("history", val);
    let _ = store.save();
}

pub fn load_known_hashes(app: &tauri::AppHandle) -> HashSet<String> {
    let store = app.store(STORE_FILE).expect("failed to open store");

    match store.get("knownHashes") {
        Some(val) => serde_json::from_value(val).unwrap_or_default(),
        None => HashSet::new(),
    }
}

pub fn save_known_hashes(app: &tauri::AppHandle, hashes: &HashSet<String>) {
    let store = app.store(STORE_FILE).expect("failed to open store");

    let val = serde_json::to_value(hashes).expect("failed to serialize known hashes");
    store.set("knownHashes", val);
    let _ = store.save();
}
