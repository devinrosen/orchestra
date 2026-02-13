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
    let walk_result = walker::walk_directory(root, &[]);
    let total = walk_result.audio_files.len();

    let mut tracks = Vec::new();
    for (i, file_path) in walk_result.audio_files.iter().enumerate() {
        let _ = on_progress.send(ProgressEvent::ScanProgress {
            files_found: total,
            files_processed: i + 1,
            current_file: file_path.file_name().unwrap_or_default().to_string_lossy().to_string(),
        });

        match metadata::extract_metadata(file_path, root) {
            Ok(track) => tracks.push(track),
            Err(e) => {
                eprintln!("Failed to read metadata for {}: {}", file_path.display(), e);
            }
        }
    }

    // Store in database
    {
        let conn = db.lock().map_err(|e| AppError::General(e.to_string()))?;
        let file_paths: Vec<String> = tracks.iter().map(|t| t.file_path.clone()).collect();

        for track in &tracks {
            library_repo::upsert_track(&conn, track)?;
        }

        library_repo::remove_tracks_not_in(&conn, &path, &file_paths)?;
    }

    let duration_ms = start.elapsed().as_millis() as u64;
    let _ = on_progress.send(ProgressEvent::ScanComplete {
        total_files: tracks.len(),
        duration_ms,
    });

    Ok(tracks.len())
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
