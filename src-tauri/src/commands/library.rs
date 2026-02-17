use std::collections::HashSet;
use std::path::Path;
use std::sync::Mutex;
use rusqlite::Connection;
use tauri::ipc::Channel;
use walkdir::WalkDir;

use orchestra_core::db::library_repo;
use orchestra_core::error::AppError;
use orchestra_core::models::duplicate::DuplicateResult;
use orchestra_core::models::progress::ProgressEvent;
use orchestra_core::models::track::{is_audio_file, LibraryStats, LibraryTree, Track};
use orchestra_core::scanner::{hasher, metadata, walker};

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

/// Replace filesystem-unsafe characters with underscores, trim whitespace, and ensure non-empty.
fn sanitize_folder_name(name: &str) -> String {
    let sanitized: String = name
        .chars()
        .map(|c| match c {
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '_',
            _ => c,
        })
        .collect();
    let trimmed = sanitized.trim().to_string();
    if trimmed.is_empty() {
        "_".to_string()
    } else {
        trimmed
    }
}

/// Read artist/album metadata from a source file to determine the destination folder.
/// Returns (artist_folder, album_folder), falling back to "Unknown Artist"/"Unknown Album".
fn read_folder_metadata(src: &Path) -> (String, String) {
    if let Ok(tagged) = lofty::read_from_path(src) {
        use lofty::file::TaggedFileExt;
        use lofty::tag::Accessor;
        if let Some(tag) = tagged.primary_tag().or_else(|| tagged.first_tag()) {
            let album_artist = tag
                .get_string(&lofty::tag::ItemKey::AlbumArtist)
                .map(|s| s.to_string());
            let artist = tag.artist().map(|s| s.to_string());
            let artist_folder = album_artist
                .or(artist)
                .unwrap_or_else(|| "Unknown Artist".to_string());
            let album_folder = tag
                .album()
                .map(|s| s.to_string())
                .unwrap_or_else(|| "Unknown Album".to_string());
            return (
                sanitize_folder_name(&artist_folder),
                sanitize_folder_name(&album_folder),
            );
        }
    }
    (
        "Unknown Artist".to_string(),
        "Unknown Album".to_string(),
    )
}

/// Copy audio files from `source_paths` into `library_root`, extract metadata, and upsert to DB.
/// Progress events are delivered via `on_event`. Returns the count of successfully imported tracks.
///
/// Extracted from `import_tracks` so the logic can be unit-tested without a live Tauri `Channel`.
fn do_import_tracks(
    db: &Mutex<Connection>,
    source_paths: &[String],
    library_root: &str,
    mut on_event: impl FnMut(ProgressEvent),
) -> Result<usize, AppError> {
    let root = Path::new(library_root);
    if !root.exists() || !root.is_dir() {
        return Err(AppError::PathNotAccessible(library_root.to_string()));
    }

    let total = source_paths.len();
    let mut imported = 0usize;

    for (i, source_path) in source_paths.iter().enumerate() {
        let src = Path::new(source_path);

        // Skip missing source files
        if !src.exists() {
            eprintln!("import_tracks: source path does not exist, skipping: {}", source_path);
            continue;
        }

        // Skip non-audio files
        if !is_audio_file(src) {
            eprintln!("import_tracks: not an audio file, skipping: {}", source_path);
            continue;
        }

        // Determine destination path with organized folders and collision handling
        let filename = match src.file_name() {
            Some(n) => n.to_string_lossy().to_string(),
            None => {
                eprintln!("import_tracks: cannot determine filename, skipping: {}", source_path);
                continue;
            }
        };

        // Read metadata from source to determine artist/album folders
        let (artist_folder, album_folder) = read_folder_metadata(src);
        let dest_dir = root.join(&artist_folder).join(&album_folder);
        if let Err(e) = std::fs::create_dir_all(&dest_dir) {
            eprintln!("import_tracks: failed to create directory {}: {}", dest_dir.display(), e);
            continue;
        }

        let stem = Path::new(&filename)
            .file_stem()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_default();
        let ext = Path::new(&filename)
            .extension()
            .map(|e| e.to_string_lossy().to_string());

        let dest_path = {
            let candidate = dest_dir.join(&filename);
            if !candidate.exists() {
                candidate
            } else {
                let mut found = None;
                for n in 1u32..=99 {
                    let new_name = match &ext {
                        Some(e) => format!("{}_{}.{}", stem, n, e),
                        None => format!("{}_{}", stem, n),
                    };
                    let c = dest_dir.join(&new_name);
                    if !c.exists() {
                        found = Some(c);
                        break;
                    }
                }
                match found {
                    Some(p) => p,
                    None => {
                        return Err(AppError::General(format!(
                            "Could not find a free filename for {} after 99 attempts",
                            filename
                        )));
                    }
                }
            }
        };

        // Copy the file
        if let Err(e) = std::fs::copy(src, &dest_path) {
            eprintln!("import_tracks: failed to copy {}: {}", source_path, e);
            continue;
        }

        // Send progress before metadata extraction
        let _ = on_event(ProgressEvent::ScanProgress {
            files_found: total,
            files_processed: i + 1,
            current_file: dest_path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string(),
            dirs_total: 0,
            dirs_completed: 0,
        });

        // Extract metadata from the copied file
        match metadata::extract_metadata(&dest_path, root) {
            Ok(track) => {
                let conn = db.lock().map_err(|e| AppError::General(e.to_string()))?;
                library_repo::upsert_track(&conn, &track)?;
                imported += 1;
            }
            Err(e) => {
                eprintln!(
                    "import_tracks: failed to extract metadata for {}: {}",
                    dest_path.display(),
                    e
                );
            }
        }
    }

    on_event(ProgressEvent::ScanComplete {
        total_files: imported,
        duration_ms: 0,
    });

    Ok(imported)
}

#[tauri::command]
pub async fn import_tracks(
    db: tauri::State<'_, Mutex<Connection>>,
    source_paths: Vec<String>,
    library_root: String,
    on_progress: Channel<ProgressEvent>,
) -> Result<usize, AppError> {
    do_import_tracks(&db, &source_paths, &library_root, |event| {
        let _ = on_progress.send(event);
    })
}

#[cfg(test)]
mod hash_progress_tests {
    use super::*;
    use orchestra_core::db::schema;
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

#[cfg(test)]
mod import_tracks_tests {
    use super::*;
    use orchestra_core::db::schema;
    use rusqlite::Connection;
    use std::sync::Mutex;
    use tempfile::TempDir;

    fn setup_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        schema::run_migrations(&conn).unwrap();
        conn
    }

    /// Create a fake audio file (not a real audio format — lofty will fail to parse it,
    /// but it is sufficient to test file copy and collision logic).
    fn write_fake_audio(dir: &std::path::Path, name: &str) -> String {
        let path = dir.join(name);
        std::fs::write(&path, b"fake audio content").unwrap();
        path.to_string_lossy().to_string()
    }

    /// Create a fake non-audio file.
    fn write_fake_text(dir: &std::path::Path, name: &str) -> String {
        let path = dir.join(name);
        std::fs::write(&path, b"this is a text file").unwrap();
        path.to_string_lossy().to_string()
    }

    #[test]
    fn test_import_single_file_success() {
        let library_dir = TempDir::new().unwrap();
        let source_dir = TempDir::new().unwrap();

        let source_path = write_fake_audio(source_dir.path(), "track.flac");

        let conn = setup_db();
        let db = Mutex::new(conn);

        let result = do_import_tracks(
            &db,
            &[source_path],
            library_dir.path().to_str().unwrap(),
            |_| {},
        );

        // The function must not error even when lofty cannot parse the fake binary content.
        assert!(result.is_ok(), "expected Ok, got {:?}", result);

        // The file must be copied into the organized subfolder (Unknown Artist/Unknown Album
        // because fake audio has no metadata).
        let dest = library_dir.path().join("Unknown Artist").join("Unknown Album").join("track.flac");
        assert!(dest.exists(), "expected track.flac to be copied into Unknown Artist/Unknown Album/");
    }

    #[test]
    fn test_import_collision_renamed() {
        let library_dir = TempDir::new().unwrap();
        let source_dir = TempDir::new().unwrap();

        // Pre-place a file named track.flac in the organized subfolder
        let dest_dir = library_dir.path().join("Unknown Artist").join("Unknown Album");
        std::fs::create_dir_all(&dest_dir).unwrap();
        std::fs::write(dest_dir.join("track.flac"), b"original").unwrap();

        // Import another file also named track.flac from a different source directory
        let source_path = write_fake_audio(source_dir.path(), "track.flac");

        let conn = setup_db();
        let db = Mutex::new(conn);

        let _result = do_import_tracks(
            &db,
            &[source_path],
            library_dir.path().to_str().unwrap(),
            |_| {},
        );

        // Both the original and the renamed copy must exist within the organized subfolder
        assert!(
            dest_dir.join("track.flac").exists(),
            "original track.flac must still exist"
        );
        assert!(
            dest_dir.join("track_1.flac").exists(),
            "imported file must be renamed to track_1.flac"
        );
    }

    #[test]
    fn test_import_non_audio_file_skipped() {
        let library_dir = TempDir::new().unwrap();
        let source_dir = TempDir::new().unwrap();

        let source_path = write_fake_text(source_dir.path(), "readme.txt");

        let conn = setup_db();
        let db = Mutex::new(conn);

        let result = do_import_tracks(
            &db,
            &[source_path],
            library_dir.path().to_str().unwrap(),
            |_| {},
        );

        // Non-audio files must be skipped silently — no error, count = 0
        assert!(result.is_ok(), "expected Ok for non-audio file, got {:?}", result);
        assert_eq!(result.unwrap(), 0, "expected 0 imported tracks");

        // Nothing must be copied into the library root
        let entries: Vec<_> = std::fs::read_dir(library_dir.path())
            .unwrap()
            .filter_map(|e| e.ok())
            .collect();
        assert!(entries.is_empty(), "expected library root to remain empty");
    }

    #[test]
    fn test_import_missing_source_path() {
        let library_dir = TempDir::new().unwrap();
        let source_dir = TempDir::new().unwrap();

        let missing_path = source_dir.path().join("does_not_exist.flac").to_string_lossy().to_string();
        let existing_path = write_fake_audio(source_dir.path(), "present.flac");

        let conn = setup_db();
        let db = Mutex::new(conn);

        let result = do_import_tracks(
            &db,
            &[missing_path, existing_path],
            library_dir.path().to_str().unwrap(),
            |_| {},
        );

        // Missing source path must be skipped — function must not error
        assert!(result.is_ok(), "expected Ok when source path is missing, got {:?}", result);

        // The valid file must be copied into the organized subfolder
        let dest = library_dir.path().join("Unknown Artist").join("Unknown Album").join("present.flac");
        assert!(dest.exists(), "expected present.flac to be copied despite missing sibling path");
    }

    #[test]
    fn test_import_progress_events_emitted() {
        let library_dir = TempDir::new().unwrap();
        let source_dir = TempDir::new().unwrap();

        let paths: Vec<String> = (1..=3)
            .map(|i| write_fake_audio(source_dir.path(), &format!("track{}.flac", i)))
            .collect();

        let conn = setup_db();
        let db = Mutex::new(conn);

        let mut events: Vec<ProgressEvent> = Vec::new();
        let _result = do_import_tracks(
            &db,
            &paths,
            library_dir.path().to_str().unwrap(),
            |evt| events.push(evt),
        );

        // At least one ScanProgress event must be emitted (one per successfully copied file)
        let progress_count = events
            .iter()
            .filter(|e| matches!(e, ProgressEvent::ScanProgress { .. }))
            .count();
        assert!(progress_count > 0, "expected at least one ScanProgress event");

        // Exactly one ScanComplete must be emitted at the end
        let complete_events: Vec<_> = events
            .iter()
            .filter(|e| matches!(e, ProgressEvent::ScanComplete { .. }))
            .collect();
        assert_eq!(complete_events.len(), 1, "expected exactly one ScanComplete event");

        // ScanComplete must be the last event
        assert!(
            matches!(events.last(), Some(ProgressEvent::ScanComplete { .. })),
            "ScanComplete must be the last event"
        );
    }

    #[test]
    fn test_sanitize_folder_name() {
        assert_eq!(sanitize_folder_name("AC/DC"), "AC_DC");
        assert_eq!(sanitize_folder_name("What?"), "What_");
        assert_eq!(sanitize_folder_name("  Spaces  "), "Spaces");
        assert_eq!(sanitize_folder_name("Normal Name"), "Normal Name");
        assert_eq!(sanitize_folder_name(""), "_");
        assert_eq!(sanitize_folder_name("A:B*C"), "A_B_C");
    }

    #[test]
    fn test_import_creates_organized_folders() {
        let library_dir = TempDir::new().unwrap();
        let source_dir = TempDir::new().unwrap();

        // Import two fake audio files — both will land in Unknown Artist/Unknown Album
        let p1 = write_fake_audio(source_dir.path(), "song_a.flac");
        let p2 = write_fake_audio(source_dir.path(), "song_b.mp3");

        let conn = setup_db();
        let db = Mutex::new(conn);

        let _result = do_import_tracks(
            &db,
            &[p1, p2],
            library_dir.path().to_str().unwrap(),
            |_| {},
        );

        let organized = library_dir.path().join("Unknown Artist").join("Unknown Album");
        assert!(organized.join("song_a.flac").exists(), "song_a.flac must be in organized folder");
        assert!(organized.join("song_b.mp3").exists(), "song_b.mp3 must be in organized folder");

        // Root should NOT contain the files directly
        assert!(!library_dir.path().join("song_a.flac").exists(), "files should not be in root");
        assert!(!library_dir.path().join("song_b.mp3").exists(), "files should not be in root");
    }
}
