use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tauri::ipc::Channel;

use crate::db::device_repo::CachedFileHash;
use crate::error::AppError;
use crate::models::diff::{DiffAction, DiffDirection, DiffEntry, DiffResult};
use crate::models::progress::ProgressEvent;
use crate::models::track::{is_audio_file, Track};
use crate::scanner::hasher;
use crate::sync::one_way::{copy_file_safe, remove_empty_parents};

struct FileInfo {
    size: u64,
    modified: i64,
}

fn is_hidden(name: &std::ffi::OsStr) -> bool {
    name.to_string_lossy().starts_with('.')
}

fn collect_device_files(
    root: &Path,
    channel: &Channel<ProgressEvent>,
) -> Result<HashMap<String, FileInfo>, AppError> {
    let mut files = HashMap::new();
    if !root.exists() {
        return Ok(files);
    }
    let walker = walkdir::WalkDir::new(root)
        .follow_links(true)
        .into_iter()
        .filter_entry(|e| {
            // Skip hidden/system directories (e.g. .Spotlight-V100, .Trashes, .fseventsd)
            !e.file_type().is_dir() || !is_hidden(e.file_name())
        });
    for entry in walker {
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue, // skip permission errors and other walk failures
        };
        if !entry.file_type().is_file() || !is_audio_file(entry.path()) {
            continue;
        }
        let rel = entry
            .path()
            .strip_prefix(root)
            .unwrap_or(entry.path())
            .to_string_lossy()
            .to_string();
        let meta = match std::fs::metadata(entry.path()) {
            Ok(m) => m,
            Err(_) => continue, // skip files we can't stat
        };
        let modified = meta
            .modified()
            .ok()
            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);
        files.insert(rel.clone(), FileInfo { size: meta.len(), modified });

        let _ = channel.send(ProgressEvent::DeviceScanProgress {
            files_found: files.len(),
            current_file: rel,
        });
    }
    Ok(files)
}

/// Returns (DiffResult, updated_cache_entries) so the caller can persist the cache after sync.
pub fn compute_device_diff(
    device_id: &str,
    library_tracks: &[Track],
    device_root: &Path,
    channel: &Channel<ProgressEvent>,
    hash_cache: &HashMap<String, CachedFileHash>,
) -> Result<(DiffResult, Vec<CachedFileHash>), AppError> {
    // Build map of library tracks by relative_path
    let library_map: HashMap<&str, &Track> = library_tracks
        .iter()
        .map(|t| (t.relative_path.as_str(), t))
        .collect();

    // Walk the device to find existing files
    let device_files = collect_device_files(device_root, channel)?;

    let all_keys: HashSet<String> = library_map
        .keys()
        .map(|k| k.to_string())
        .chain(device_files.keys().cloned())
        .collect();

    let total_to_compare = all_keys.len();
    let mut entries = Vec::new();
    let mut total_add = 0usize;
    let mut total_remove = 0usize;
    let mut total_update = 0usize;
    let mut total_unchanged = 0usize;
    let mut bytes_to_transfer = 0u64;
    let mut files_compared = 0usize;
    let mut new_cache: Vec<CachedFileHash> = Vec::new();

    for rel in &all_keys {
        files_compared += 1;
        let _ = channel.send(ProgressEvent::DiffProgress {
            files_compared,
            total_files: total_to_compare,
            current_file: rel.clone(),
        });

        let in_library = library_map.get(rel.as_str());
        let in_device = device_files.get(rel);

        match (in_library, in_device) {
            (Some(track), None) => {
                // File in library but not on device — Add
                bytes_to_transfer += track.file_size;
                total_add += 1;
                entries.push(DiffEntry {
                    relative_path: rel.clone(),
                    action: DiffAction::Add,
                    direction: DiffDirection::SourceToTarget,
                    source_size: Some(track.file_size),
                    target_size: None,
                    source_hash: track.hash.clone(),
                    target_hash: None,
                    source_modified: Some(track.modified_at),
                    target_modified: None,
                });
            }
            (None, Some(dev)) => {
                // File on device but not in selected library tracks — Remove
                total_remove += 1;
                entries.push(DiffEntry {
                    relative_path: rel.clone(),
                    action: DiffAction::Remove,
                    direction: DiffDirection::SourceToTarget,
                    source_size: None,
                    target_size: Some(dev.size),
                    source_hash: None,
                    target_hash: None,
                    source_modified: None,
                    target_modified: Some(dev.modified),
                });
                // Don't add to new_cache — file will be removed
            }
            (Some(track), Some(dev)) => {
                // Both exist — compare
                if track.file_size == dev.size && track.modified_at == dev.modified {
                    total_unchanged += 1;
                    // Carry forward any cached hash
                    let cached_hash = hash_cache.get(rel).map(|c| c.hash.clone());
                    new_cache.push(CachedFileHash {
                        relative_path: rel.clone(),
                        hash: cached_hash.unwrap_or_default(),
                        file_size: dev.size,
                        modified_at: dev.modified,
                    });
                    entries.push(DiffEntry {
                        relative_path: rel.clone(),
                        action: DiffAction::Unchanged,
                        direction: DiffDirection::SourceToTarget,
                        source_size: Some(track.file_size),
                        target_size: Some(dev.size),
                        source_hash: None,
                        target_hash: None,
                        source_modified: Some(track.modified_at),
                        target_modified: Some(dev.modified),
                    });
                } else {
                    // Size or mtime differ — hash to confirm
                    let src_hash = match &track.hash {
                        Some(h) => h.clone(),
                        None => hasher::hash_file(Path::new(&track.file_path))?,
                    };
                    // Check cache: if device file size+mtime match cached entry, reuse hash
                    let tgt_hash = if let Some(cached) = hash_cache.get(rel) {
                        if cached.file_size == dev.size && cached.modified_at == dev.modified {
                            cached.hash.clone()
                        } else {
                            hasher::hash_file(&device_root.join(rel))?
                        }
                    } else {
                        hasher::hash_file(&device_root.join(rel))?
                    };
                    // Cache the device hash we just resolved
                    new_cache.push(CachedFileHash {
                        relative_path: rel.clone(),
                        hash: tgt_hash.clone(),
                        file_size: dev.size,
                        modified_at: dev.modified,
                    });
                    if src_hash == tgt_hash {
                        total_unchanged += 1;
                        entries.push(DiffEntry {
                            relative_path: rel.clone(),
                            action: DiffAction::Unchanged,
                            direction: DiffDirection::SourceToTarget,
                            source_size: Some(track.file_size),
                            target_size: Some(dev.size),
                            source_hash: Some(src_hash),
                            target_hash: Some(tgt_hash),
                            source_modified: Some(track.modified_at),
                            target_modified: Some(dev.modified),
                        });
                    } else {
                        bytes_to_transfer += track.file_size;
                        total_update += 1;
                        entries.push(DiffEntry {
                            relative_path: rel.clone(),
                            action: DiffAction::Update,
                            direction: DiffDirection::SourceToTarget,
                            source_size: Some(track.file_size),
                            target_size: Some(dev.size),
                            source_hash: Some(src_hash),
                            target_hash: Some(tgt_hash),
                            source_modified: Some(track.modified_at),
                            target_modified: Some(dev.modified),
                        });
                    }
                }
            }
            (None, None) => unreachable!(),
        }
    }

    entries.sort_by(|a, b| a.relative_path.cmp(&b.relative_path));

    let _ = channel.send(ProgressEvent::DiffComplete {
        total_entries: entries.len(),
    });

    Ok((
        DiffResult {
            profile_id: device_id.to_string(),
            entries,
            total_add,
            total_remove,
            total_update,
            total_conflict: 0,
            total_unchanged,
            bytes_to_transfer,
        },
        new_cache,
    ))
}

/// Returns (files_synced, post_sync_cache) — caller should save the cache to DB.
pub fn execute_device_sync(
    diff: &DiffResult,
    library_root: &Path,
    device_root: &Path,
    cancel_flag: Arc<AtomicBool>,
    channel: &Channel<ProgressEvent>,
    mut pre_cache: Vec<CachedFileHash>,
) -> Result<(usize, Vec<CachedFileHash>), AppError> {
    let actionable: Vec<_> = diff
        .entries
        .iter()
        .filter(|e| matches!(e.action, DiffAction::Add | DiffAction::Update | DiffAction::Remove))
        .collect();

    let total_files = actionable.len();
    let total_bytes: u64 = actionable
        .iter()
        .map(|e| e.source_size.unwrap_or(0))
        .sum();

    let _ = channel.send(ProgressEvent::SyncStarted {
        total_files,
        total_bytes,
    });

    let start = std::time::Instant::now();
    let mut files_completed = 0usize;
    let mut bytes_completed = 0u64;

    for entry in &actionable {
        if cancel_flag.load(Ordering::Relaxed) {
            return Err(AppError::SyncCancelled);
        }

        // Check device is still connected
        if !device_root.exists() {
            return Err(AppError::DeviceDisconnected(
                device_root.to_string_lossy().to_string(),
            ));
        }

        let _ = channel.send(ProgressEvent::SyncProgress {
            files_completed,
            total_files,
            bytes_completed,
            total_bytes,
            current_file: entry.relative_path.clone(),
        });

        let result = match entry.action {
            DiffAction::Add | DiffAction::Update => {
                let src_path = library_root.join(&entry.relative_path);
                let tgt_path = device_root.join(&entry.relative_path);
                let copy_result = copy_file_safe(&src_path, &tgt_path);
                if copy_result.is_ok() {
                    // Update cache with the file we just wrote
                    let hash = entry.source_hash.clone().unwrap_or_default();
                    let meta = std::fs::metadata(&tgt_path).ok();
                    let (size, mtime) = meta.map(|m| {
                        let mtime = m.modified().ok()
                            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                            .map(|d| d.as_secs() as i64)
                            .unwrap_or(0);
                        (m.len(), mtime)
                    }).unwrap_or((entry.source_size.unwrap_or(0), entry.source_modified.unwrap_or(0)));

                    // Remove old entry if exists, then add new one
                    pre_cache.retain(|c| c.relative_path != entry.relative_path);
                    pre_cache.push(CachedFileHash {
                        relative_path: entry.relative_path.clone(),
                        hash,
                        file_size: size,
                        modified_at: mtime,
                    });
                }
                copy_result
            }
            DiffAction::Remove => {
                let tgt_path = device_root.join(&entry.relative_path);
                if tgt_path.exists() {
                    std::fs::remove_file(&tgt_path)?;
                    if let Some(parent) = tgt_path.parent() {
                        let _ = remove_empty_parents(parent);
                    }
                }
                // Remove from cache
                pre_cache.retain(|c| c.relative_path != entry.relative_path);
                Ok(())
            }
            _ => Ok(()),
        };

        if let Err(e) = result {
            let _ = channel.send(ProgressEvent::SyncError {
                file: entry.relative_path.clone(),
                error: e.to_string(),
            });
        }

        bytes_completed += entry.source_size.unwrap_or(0);
        files_completed += 1;
    }

    let duration_ms = start.elapsed().as_millis() as u64;
    let _ = channel.send(ProgressEvent::SyncComplete {
        files_synced: files_completed,
        duration_ms,
    });

    Ok((files_completed, pre_cache))
}
