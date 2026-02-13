use std::collections::HashSet;
use std::path::Path;
use std::sync::Mutex;
use rusqlite::Connection;
use tauri::ipc::Channel;
use walkdir::WalkDir;

use crate::db::library_repo;
use crate::error::AppError;
use crate::models::progress::ProgressEvent;
use crate::models::track::{is_audio_file, LibraryTree, Track};
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

    // Load known directories from DB
    let known_dirs = {
        let conn = db.lock().map_err(|e| AppError::General(e.to_string()))?;
        library_repo::get_known_directories(&conn, &path)?
    };

    // ── Phase 1: Directory-only walk (fast — no per-file stat) ──
    let mut disk_dirs: HashSet<String> = HashSet::new();
    for entry in WalkDir::new(root).follow_links(true).into_iter().filter_map(|e| e.ok()) {
        if entry.file_type().is_dir() {
            let rel = entry.path().strip_prefix(root).unwrap_or(entry.path());
            let dir = rel.to_string_lossy().to_string();
            if !dir.is_empty() {
                disk_dirs.insert(dir);
            }
        }
    }

    let new_dirs: Vec<&String> = disk_dirs.difference(&known_dirs).collect();
    let removed_dirs: Vec<&String> = known_dirs.difference(&disk_dirs).collect();

    let mut new_track_count: usize = 0;

    // Process new directories — walk only those dirs for audio files and extract metadata
    {
        let conn = db.lock().map_err(|e| AppError::General(e.to_string()))?;

        for (i, dir) in new_dirs.iter().enumerate() {
            let dir_path = root.join(dir);
            if !dir_path.is_dir() {
                continue;
            }
            // List audio files in this directory (non-recursive — just direct children)
            let files: Vec<std::path::PathBuf> = std::fs::read_dir(&dir_path)
                .into_iter()
                .flatten()
                .filter_map(|e| e.ok())
                .filter(|e| e.file_type().map(|ft| ft.is_file()).unwrap_or(false))
                .map(|e| e.path())
                .filter(|p| is_audio_file(p))
                .collect();

            for file_path in &files {
                let _ = on_progress.send(ProgressEvent::ScanProgress {
                    files_found: files.len(),
                    files_processed: new_track_count,
                    current_file: file_path.file_name().unwrap_or_default().to_string_lossy().to_string(),
                    dirs_total: new_dirs.len(),
                    dirs_completed: i,
                });
                match metadata::extract_metadata(file_path, root) {
                    Ok(track) => {
                        library_repo::upsert_track(&conn, &track)?;
                        new_track_count += 1;
                    }
                    Err(e) => {
                        eprintln!("Failed to read metadata for {}: {}", file_path.display(), e);
                    }
                }
            }
        }

        // Remove tracks from deleted directories
        for dir in &removed_dirs {
            library_repo::remove_tracks_by_directory(&conn, &path, dir)?;
        }
    }

    // Signal frontend to reload tree after phase 1
    if !new_dirs.is_empty() || !removed_dirs.is_empty() {
        let _ = on_progress.send(ProgressEvent::ScanTreeUpdated {
            new_dirs: new_dirs.len(),
            removed_dirs: removed_dirs.len(),
            new_tracks: new_track_count,
        });
    }

    // ── Phase 2: Per-file incremental scan of existing directories ──
    let fingerprints = {
        let conn = db.lock().map_err(|e| AppError::General(e.to_string()))?;
        library_repo::get_track_fingerprints(&conn, &path)?
    };

    let mut files_processed: usize = 0;
    let mut all_file_paths: Vec<String> = Vec::new();
    let mut phase2_tracks: Vec<Track> = Vec::new();

    for file_path in walker::walk_directory_iter(root, &[]) {
        let path_str = file_path.to_string_lossy().to_string();
        all_file_paths.push(path_str.clone());

        // Skip files in new directories (already processed in phase 1)
        let rel = file_path.strip_prefix(root).unwrap_or(&file_path);
        let dir = rel
            .parent()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_default();
        if !dir.is_empty() && new_dirs.iter().any(|d| **d == dir) {
            continue;
        }

        files_processed += 1;

        if files_processed % 50 == 0 {
            let _ = on_progress.send(ProgressEvent::ScanProgress {
                files_found: 0,
                files_processed,
                current_file: file_path.file_name().unwrap_or_default().to_string_lossy().to_string(),
                dirs_total: 0,
                dirs_completed: 0,
            });
        }

        // Check if file is unchanged
        if let Ok(fs_meta) = std::fs::metadata(&file_path) {
            let fs_size = fs_meta.len();
            let fs_mtime = fs_meta
                .modified()
                .ok()
                .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                .map(|d| d.as_secs() as i64)
                .unwrap_or(0);

            if let Some(&(db_size, db_mtime)) = fingerprints.get(&path_str) {
                if fs_size == db_size && fs_mtime == db_mtime {
                    continue;
                }
            }
        }

        // File is new or changed — extract metadata
        match metadata::extract_metadata(&file_path, root) {
            Ok(track) => {
                phase2_tracks.push(track);
            }
            Err(e) => {
                eprintln!("Failed to read metadata for {}: {}", file_path.display(), e);
            }
        }
    }

    // Write phase 2 changes to DB
    {
        let conn = db.lock().map_err(|e| AppError::General(e.to_string()))?;

        for track in &phase2_tracks {
            library_repo::upsert_track(&conn, track)?;
        }

        library_repo::remove_tracks_not_in(&conn, &path, &all_file_paths)?;
    }

    let total = new_track_count + files_processed;
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
