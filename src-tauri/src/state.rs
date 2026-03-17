use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::sync::Mutex;
use tokio::sync::Semaphore;

const MAX_ENTRIES: usize = 100;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum UploadStatus {
    Pending,
    Uploading,
    Queued,
    Duplicate,
    Error,
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
}

#[derive(Debug, Default)]
pub struct AppState {
    pub uploads: VecDeque<UploadEntry>,
    pub watcher_running: bool,
}

impl AppState {
    pub fn add_entry(&mut self, entry: UploadEntry) {
        self.uploads.push_front(entry);
        while self.uploads.len() > MAX_ENTRIES {
            self.uploads.pop_back();
        }
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
        self.uploads.iter().any(|e| {
            e.sha256.as_deref() == Some(sha256)
                && matches!(
                    e.status,
                    UploadStatus::Queued
                        | UploadStatus::Duplicate
                        | UploadStatus::Uploading
                        | UploadStatus::Error
                        | UploadStatus::Pending
                )
        })
    }

    pub fn get_retryable(&self) -> Vec<UploadEntry> {
        self.uploads
            .iter()
            .filter(|e| e.status == UploadStatus::Error && e.retry_count < 5)
            .cloned()
            .collect()
    }
}

pub type SharedState = Mutex<AppState>;
pub type UploadSemaphore = Semaphore;
