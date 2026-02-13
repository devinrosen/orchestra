use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ProgressEvent {
    #[serde(rename = "scan_started")]
    ScanStarted { path: String },
    #[serde(rename = "scan_progress")]
    ScanProgress {
        files_found: usize,
        files_processed: usize,
        current_file: String,
    },
    #[serde(rename = "scan_complete")]
    ScanComplete {
        total_files: usize,
        duration_ms: u64,
    },
    #[serde(rename = "diff_progress")]
    DiffProgress {
        files_compared: usize,
        total_files: usize,
    },
    #[serde(rename = "diff_complete")]
    DiffComplete { total_entries: usize },
    #[serde(rename = "sync_started")]
    SyncStarted {
        total_files: usize,
        total_bytes: u64,
    },
    #[serde(rename = "sync_progress")]
    SyncProgress {
        files_completed: usize,
        total_files: usize,
        bytes_completed: u64,
        total_bytes: u64,
        current_file: String,
    },
    #[serde(rename = "sync_complete")]
    SyncComplete {
        files_synced: usize,
        duration_ms: u64,
    },
    #[serde(rename = "sync_error")]
    SyncError {
        file: String,
        error: String,
    },
}
