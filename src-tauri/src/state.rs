use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashSet, VecDeque};
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::ipc::Channel;
use tokio::sync::Semaphore;

const MAX_COMPLETED: usize = 100;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum UploadStatus {
    Pending,
    Uploading,
    Queued,
    Duplicate,
    Error,
}

impl UploadStatus {
    pub fn is_completed(&self) -> bool {
        matches!(self, UploadStatus::Queued | UploadStatus::Duplicate)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UploadEntry {
    pub id: String,
    pub file_name: String,
    pub file_path: String,
    pub status: UploadStatus,
    pub sha256: Option<String>,
    pub error: Option<String>,
    pub created_at: DateTime<Utc>,
    pub retry_count: u32,
    #[serde(default)]
    pub retryable: Option<bool>,
    #[serde(default)]
    pub last_attempt_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Default)]
pub struct AppState {
    pub uploads: VecDeque<UploadEntry>,
    pub known_hashes: HashSet<String>,
    pub watcher_running: bool,
}

impl AppState {
    pub fn add_entry(&mut self, entry: UploadEntry) {
        if let Some(sha256) = &entry.sha256 {
            self.known_hashes.insert(sha256.clone());
        }
        self.uploads.push_front(entry);
    }

    pub fn prune_completed(&mut self) {
        let mut completed_count = 0;
        self.uploads.retain(|e| {
            if e.status.is_completed() {
                completed_count += 1;
                completed_count <= MAX_COMPLETED
            } else {
                true
            }
        });
    }

    pub fn update_entry<F>(&mut self, id: &str, update: F) -> Option<UploadEntry>
    where
        F: FnOnce(&mut UploadEntry),
    {
        if let Some(entry) = self.uploads.iter_mut().find(|e| e.id == id) {
            update(entry);
            Some(entry.clone())
        } else {
            None
        }
    }

    pub fn has_sha256(&self, sha256: &str) -> bool {
        self.known_hashes.contains(sha256)
    }

    pub fn get_retryable(&self) -> Vec<UploadEntry> {
        self.uploads
            .iter()
            .filter(|e| e.status == UploadStatus::Error && e.retry_count < 5)
            .cloned()
            .collect()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum RecordingStatus {
    Idle,
    Recording,
    Uploading,
}

impl Default for RecordingStatus {
    fn default() -> Self {
        RecordingStatus::Idle
    }
}

#[derive(Debug, Default)]
pub struct RecordingState {
    pub status: RecordingStatus,
    pub recording_session_uuid: Option<String>,
    pub session_path: Option<PathBuf>,
}

pub type SharedState = Mutex<AppState>;
pub type SharedRecordingState = Mutex<RecordingState>;
pub type UploadSemaphore = Semaphore;
pub type UploadChannels = Mutex<Vec<Channel<Vec<UploadEntry>>>>;
