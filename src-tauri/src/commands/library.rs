use std::collections::HashSet;
use std::path::Path;
use std::sync::Mutex;
use rusqlite::Connection;
use tauri::ipc::Channel;
use walkdir::WalkDir;

use crate::db::library_repo;
use crate::error::AppError;
use crate::models::duplicate::DuplicateResult;
use crate::models::progress::ProgressEvent;
use crate::models::track::{is_audio_file, LibraryStats, LibraryTree, Track};
use crate::scanner::{hasher, metadata, walker};

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

#[tauri::command]
pub async fn get_incomplete_tracks(
    db: tauri::State<'_, Mutex<Connection>>,
    root: String,
) -> Result<Vec<Track>, AppError> {
    let conn = db.lock().map_err(|e| AppError::General(e.to_string()))?;
    library_repo::get_incomplete_tracks(&conn, &root)
}

#[tauri::command]
pub async fn get_library_stats(
    db: tauri::State<'_, Mutex<Connection>>,
    root: String,
) -> Result<LibraryStats, AppError> {
    let conn = db.lock().map_err(|e| AppError::General(e.to_string()))?;
    library_repo::get_library_stats(&conn, &root)
}

/// Hash all un-hashed tracks for the given library root, calling `on_event` for each
/// progress event. Returns the number of tracks hashed.
///
/// This function is extracted from `find_duplicates` so that the progress-event logic
/// can be unit-tested without a live Tauri `Channel`.
fn hash_unhashed_tracks(
    db: &Mutex<Connection>,
    root: &str,
    mut on_event: impl FnMut(ProgressEvent),
) -> Result<usize, AppError> {
    let unhashed = {
        let conn = db.lock().map_err(|e| AppError::General(e.to_string()))?;
        library_repo::get_tracks_without_hash(&conn, root)?
    };

    let total = unhashed.len();
    if total > 0 {
        on_event(ProgressEvent::HashStarted { total });
    }
    for (i, (id, file_path)) in unhashed.iter().enumerate() {
        if i % 10 == 0 || i == total - 1 {
            on_event(ProgressEvent::HashProgress {
                files_hashed: i,
                total_files: total,
                current_file: file_path
                    .rsplit('/')
                    .next()
                    .unwrap_or(file_path)
                    .to_string(),
            });
        }
        match hasher::hash_file(Path::new(file_path)) {
            Ok(hash) => {
                let conn = db.lock().map_err(|e| AppError::General(e.to_string()))?;
                library_repo::update_track_hash(&conn, *id, &hash)?;
            }
            Err(e) => {
                eprintln!("Failed to hash {}: {}", file_path, e);
            }
        }
    }
    Ok(total)
}

#[tauri::command]
pub async fn find_duplicates(
    db: tauri::State<'_, Mutex<Connection>>,
    root: String,
    on_progress: Channel<ProgressEvent>,
) -> Result<DuplicateResult, AppError> {
    // Phase 1: Hash all un-hashed tracks
    let total = hash_unhashed_tracks(&db, &root, |event| {
        let _ = on_progress.send(event);
    })?;

    // Phase 2: Query for duplicates
    let conn = db.lock().map_err(|e| AppError::General(e.to_string()))?;
    let hash_groups = library_repo::find_hash_duplicates(&conn, &root)?;
    let meta_groups = library_repo::find_metadata_duplicates(&conn, &root)?;

    let mut all_groups = hash_groups;
    all_groups.extend(meta_groups);

    let total_duplicate_tracks: usize = all_groups.iter().map(|g| g.tracks.len() - 1).sum();
    let total_wasted_bytes: u64 = all_groups
        .iter()
        .map(|g| g.tracks.iter().skip(1).map(|t| t.file_size).sum::<u64>())
        .sum();

    let _ = on_progress.send(ProgressEvent::ScanComplete {
        total_files: total,
        duration_ms: 0,
    });

    Ok(DuplicateResult {
        groups: all_groups,
        total_duplicate_tracks,
        total_wasted_bytes,
    })
}

#[tauri::command]
pub async fn delete_duplicate_tracks(
    db: tauri::State<'_, Mutex<Connection>>,
    track_ids: Vec<i64>,
    file_paths: Vec<String>,
) -> Result<usize, AppError> {
    // Delete files from disk
    for path in &file_paths {
        if let Err(e) = std::fs::remove_file(path) {
            eprintln!("Failed to delete {}: {}", path, e);
        }
    }

    // Remove from database
    let conn = db.lock().map_err(|e| AppError::General(e.to_string()))?;
    library_repo::delete_tracks_by_ids(&conn, &track_ids)
}

#[cfg(test)]
mod hash_progress_tests {
    use super::*;
    use crate::db::schema;
    use rusqlite::Connection;
    use std::sync::Mutex;
    use tempfile::TempDir;

    fn setup_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        schema::run_migrations(&conn).unwrap();
        conn
    }

    /// Insert a track whose file_path points to a real file on disk.
    fn insert_track_with_file(conn: &Connection, tmp: &TempDir, name: &str) -> String {
        let file_path = tmp.path().join(name);
        std::fs::write(&file_path, b"fake audio content").unwrap();
        let path_str = file_path.to_string_lossy().to_string();
        conn.execute(
            "INSERT INTO tracks (file_path, relative_path, library_root, format, file_size, modified_at)
             VALUES (?1, ?2, '/music', 'flac', 1000, 0)",
            rusqlite::params![path_str, name],
        )
        .unwrap();
        path_str
    }

    #[test]
    fn test_hash_progress_events_emitted() {
        let tmp = TempDir::new().unwrap();
        let conn = setup_db();

        // Insert 3 un-hashed tracks whose files exist on disk
        insert_track_with_file(&conn, &tmp, "track1.flac");
        insert_track_with_file(&conn, &tmp, "track2.flac");
        insert_track_with_file(&conn, &tmp, "track3.flac");

        let db = Mutex::new(conn);
        let mut collected: Vec<ProgressEvent> = Vec::new();
        hash_unhashed_tracks(&db, "/music", |evt| collected.push(evt)).unwrap();

        // Must have exactly one HashStarted with total = 3
        let started: Vec<_> = collected
            .iter()
            .filter(|e| matches!(e, ProgressEvent::HashStarted { total: 3 }))
            .collect();
        assert_eq!(started.len(), 1, "expected exactly one HashStarted {{ total: 3 }}");

        // Must have at least one HashProgress with total_files = 3
        let progress: Vec<_> = collected
            .iter()
            .filter(|e| matches!(e, ProgressEvent::HashProgress { total_files: 3, .. }))
            .collect();
        assert!(!progress.is_empty(), "expected at least one HashProgress event");
    }

    #[test]
    fn test_hash_progress_zero_unhashed() {
        let conn = setup_db();

        // Insert a track that already has a hash — nothing to hash
        conn.execute(
            "INSERT INTO tracks (file_path, relative_path, library_root, format, file_size, modified_at, hash)
             VALUES ('/music/prehashed.flac', 'prehashed.flac', '/music', 'flac', 1000, 0, 'abc123')",
            [],
        )
        .unwrap();

        let db = Mutex::new(conn);
        let mut collected: Vec<ProgressEvent> = Vec::new();
        let total = hash_unhashed_tracks(&db, "/music", |evt| collected.push(evt)).unwrap();

        // total == 0, no HashStarted should be emitted
        assert_eq!(total, 0);
        assert!(
            collected.is_empty(),
            "expected no events when all tracks are already hashed"
        );
    }
}
