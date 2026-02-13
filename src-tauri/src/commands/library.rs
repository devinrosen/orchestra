use std::path::Path;
use std::sync::Mutex;
use rusqlite::Connection;
use tauri::ipc::Channel;

use crate::db::library_repo;
use crate::error::AppError;
use crate::models::progress::ProgressEvent;
use crate::models::track::{LibraryTree, Track};
use crate::scanner::{metadata, walker};

#[tauri::command]
pub async fn scan_directory(
    db: tauri::State<'_, Mutex<Connection>>,
    path: String,
    on_progress: Channel<ProgressEvent>,
) -> Result<usize, AppError> {
    let root = Path::new(&path);
    if !root.exists() || !root.is_dir() {
        return Err(AppError::PathNotAccessible(path));
    }

    let _ = on_progress.send(ProgressEvent::ScanStarted { path: path.clone() });

    let start = std::time::Instant::now();

    // Load existing track fingerprints for incremental scan
    let existing = {
        let conn = db.lock().map_err(|e| AppError::General(e.to_string()))?;
        library_repo::get_track_fingerprints(&conn, &path)?
    };

    // Quick count of immediate subdirectories for progress tracking
    let subdirs: Vec<String> = std::fs::read_dir(root)
        .into_iter()
        .flatten()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().map(|ft| ft.is_dir()).unwrap_or(false))
        .filter_map(|e| e.file_name().into_string().ok())
        .collect();
    let dirs_total = subdirs.len().max(1);

    let mut new_tracks = Vec::new();
    let mut all_file_paths = Vec::new();
    let mut files_found: usize = 0;
    let mut files_skipped: usize = 0;
    let mut current_dir = String::new();
    let mut dirs_completed: usize = 0;
    for file_path in walker::walk_directory_iter(root, &[]) {
        files_found += 1;
        let path_str = file_path.to_string_lossy().to_string();
        all_file_paths.push(path_str.clone());

        // Track top-level subdirectory transitions
        let top_dir = file_path.strip_prefix(root)
            .ok()
            .and_then(|rel| rel.components().next())
            .map(|c| c.as_os_str().to_string_lossy().to_string())
            .unwrap_or_default();
        if top_dir != current_dir && !current_dir.is_empty() {
            dirs_completed += 1;
        }
        current_dir = top_dir;

        let _ = on_progress.send(ProgressEvent::ScanProgress {
            files_found,
            files_processed: new_tracks.len() + files_skipped,
            current_file: file_path.file_name().unwrap_or_default().to_string_lossy().to_string(),
            dirs_total,
            dirs_completed,
        });

        // Check if file is unchanged (same size + mtime) — skip expensive metadata extraction
        if let Ok(fs_meta) = std::fs::metadata(&file_path) {
            let fs_size = fs_meta.len();
            let fs_mtime = fs_meta
                .modified()
                .ok()
                .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                .map(|d| d.as_secs() as i64)
                .unwrap_or(0);

            if let Some(&(db_size, db_mtime)) = existing.get(&path_str) {
                if fs_size == db_size && fs_mtime == db_mtime {
                    files_skipped += 1;
                    continue;
                }
            }
        }

        match metadata::extract_metadata(&file_path, root) {
            Ok(track) => new_tracks.push(track),
            Err(e) => {
                eprintln!("Failed to read metadata for {}: {}", file_path.display(), e);
            }
        }
    }

    // Store in database
    {
        let conn = db.lock().map_err(|e| AppError::General(e.to_string()))?;

        for track in &new_tracks {
            library_repo::upsert_track(&conn, track)?;
        }

        library_repo::remove_tracks_not_in(&conn, &path, &all_file_paths)?;
    }

    let total = new_tracks.len() + files_skipped;
    let duration_ms = start.elapsed().as_millis() as u64;
    let _ = on_progress.send(ProgressEvent::ScanComplete {
        total_files: total,
        duration_ms,
    });

    Ok(total)
}

#[tauri::command]
pub async fn get_library_tree(
    db: tauri::State<'_, Mutex<Connection>>,
    root: String,
) -> Result<LibraryTree, AppError> {
    let conn = db.lock().map_err(|e| AppError::General(e.to_string()))?;
    library_repo::get_library_tree(&conn, &root)
}

#[tauri::command]
pub async fn search_library(
    db: tauri::State<'_, Mutex<Connection>>,
    query: String,
) -> Result<Vec<Track>, AppError> {
    let conn = db.lock().map_err(|e| AppError::General(e.to_string()))?;
    library_repo::search_tracks(&conn, &query)
}
