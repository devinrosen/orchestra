use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tauri::ipc::Channel;

use crate::db::sync_state_repo::FileBaseline;
use crate::error::AppError;
use crate::models::conflict::{Conflict, ConflictResolution, ConflictType, Resolution};
use crate::models::diff::{DiffAction, DiffDirection, DiffEntry, DiffResult};
use crate::models::progress::ProgressEvent;
use crate::models::track::is_audio_file;
use crate::scanner::hasher;
use crate::sync::one_way::copy_file_safe;

struct FileState {
    hash: String,
    modified: i64,
    size: u64,
}

fn collect_file_states(
    root: &Path,
    exclude_patterns: &[String],
) -> Result<HashMap<String, FileState>, AppError> {
    let compiled: Vec<glob::Pattern> = exclude_patterns
        .iter()
        .filter_map(|p| glob::Pattern::new(p).ok())
        .collect();

    let mut map = HashMap::new();
    for entry in walkdir::WalkDir::new(root).follow_links(true) {
        let entry = entry?;
        if !entry.file_type().is_file() || !is_audio_file(entry.path()) {
            continue;
        }
        let rel = entry
            .path()
            .strip_prefix(root)
            .unwrap_or(entry.path())
            .to_string_lossy()
            .to_string();

        if compiled.iter().any(|p| p.matches(&rel)) {
            continue;
        }

        let meta = std::fs::metadata(entry.path())?;
        let modified = meta
            .modified()?
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;

        let hash = hasher::hash_file(entry.path())?;
        map.insert(
            rel,
            FileState {
                hash,
                modified,
                size: meta.len(),
            },
        );
    }
    Ok(map)
}

pub fn compute_two_way_diff(
    profile_id: &str,
    source: &Path,
    target: &Path,
    exclude_patterns: &[String],
    baselines: &HashMap<String, FileBaseline>,
) -> Result<(DiffResult, Vec<Conflict>), AppError> {
    let source_files = collect_file_states(source, exclude_patterns)?;
    let target_files = collect_file_states(target, exclude_patterns)?;

    let all_keys: HashSet<String> = source_files
        .keys()
        .chain(target_files.keys())
        .chain(baselines.keys())
        .cloned()
        .collect();

    let mut entries = Vec::new();
    let mut conflicts = Vec::new();
    let mut total_add = 0usize;
    let mut total_remove = 0usize;
    let mut total_update = 0usize;
    let mut total_unchanged = 0usize;
    let mut total_conflict = 0usize;
    let mut bytes_to_transfer = 0u64;

    for rel in all_keys {
        let src = source_files.get(&rel);
        let tgt = target_files.get(&rel);
        let baseline = baselines.get(&rel);

        let (action, direction) = match (src, tgt, baseline) {
            // Both exist, baseline exists — three-way comparison
            (Some(s), Some(t), Some(b)) => {
                let src_changed = b.source_hash.as_deref() != Some(&s.hash);
                let tgt_changed = b.target_hash.as_deref() != Some(&t.hash);

                match (src_changed, tgt_changed) {
                    (false, false) => (DiffAction::Unchanged, DiffDirection::Both),
                    (true, false) => {
                        bytes_to_transfer += s.size;
                        total_update += 1;
                        (DiffAction::Update, DiffDirection::SourceToTarget)
                    }
                    (false, true) => {
                        bytes_to_transfer += t.size;
                        total_update += 1;
                        (DiffAction::Update, DiffDirection::TargetToSource)
                    }
                    (true, true) => {
                        if s.hash == t.hash {
                            (DiffAction::Unchanged, DiffDirection::Both)
                        } else {
                            total_conflict += 1;
                            conflicts.push(Conflict {
                                relative_path: rel.clone(),
                                conflict_type: ConflictType::BothModified,
                                source_hash: Some(s.hash.clone()),
                                target_hash: Some(t.hash.clone()),
                                source_modified: Some(s.modified),
                                target_modified: Some(t.modified),
                                source_size: Some(s.size),
                                target_size: Some(t.size),
                            });
                            (DiffAction::Conflict, DiffDirection::Both)
                        }
                    }
                }
            }

            // Both exist, no baseline (first sync) — merge or conflict
            (Some(s), Some(t), None) => {
                if s.hash == t.hash {
                    (DiffAction::Unchanged, DiffDirection::Both)
                } else {
                    total_conflict += 1;
                    conflicts.push(Conflict {
                        relative_path: rel.clone(),
                        conflict_type: ConflictType::FirstSyncDiffers,
                        source_hash: Some(s.hash.clone()),
                        target_hash: Some(t.hash.clone()),
                        source_modified: Some(s.modified),
                        target_modified: Some(t.modified),
                        source_size: Some(s.size),
                        target_size: Some(t.size),
                    });
                    (DiffAction::Conflict, DiffDirection::Both)
                }
            }

            // Only in source, no baseline — new file, add to target
            (Some(s), None, None) => {
                bytes_to_transfer += s.size;
                total_add += 1;
                (DiffAction::Add, DiffDirection::SourceToTarget)
            }

            // Only in target, no baseline — new file, add to source
            (None, Some(t), None) => {
                bytes_to_transfer += t.size;
                total_add += 1;
                (DiffAction::Add, DiffDirection::TargetToSource)
            }

            // Only in source, was in baseline — deleted from target
            (Some(s), None, Some(b)) => {
                let src_changed = b.source_hash.as_deref() != Some(&s.hash);
                if src_changed {
                    total_conflict += 1;
                    conflicts.push(Conflict {
                        relative_path: rel.clone(),
                        conflict_type: ConflictType::DeletedAndModified,
                        source_hash: Some(s.hash.clone()),
                        target_hash: None,
                        source_modified: Some(s.modified),
                        target_modified: None,
                        source_size: Some(s.size),
                        target_size: None,
                    });
                    (DiffAction::Conflict, DiffDirection::Both)
                } else {
                    total_remove += 1;
                    (DiffAction::Remove, DiffDirection::SourceToTarget)
                }
            }

            // Only in target, was in baseline — deleted from source
            (None, Some(t), Some(b)) => {
                let tgt_changed = b.target_hash.as_deref() != Some(&t.hash);
                if tgt_changed {
                    total_conflict += 1;
                    conflicts.push(Conflict {
                        relative_path: rel.clone(),
                        conflict_type: ConflictType::DeletedAndModified,
                        source_hash: None,
                        target_hash: Some(t.hash.clone()),
                        source_modified: None,
                        target_modified: Some(t.modified),
                        source_size: None,
                        target_size: Some(t.size),
                    });
                    (DiffAction::Conflict, DiffDirection::Both)
                } else {
                    total_remove += 1;
                    (DiffAction::Remove, DiffDirection::TargetToSource)
                }
            }

            // Neither exists but was in baseline — both deleted
            (None, None, Some(_)) => {
                (DiffAction::Unchanged, DiffDirection::Both)
            }

            (None, None, None) => unreachable!(),
        };

        if action == DiffAction::Unchanged {
            total_unchanged += 1;
        }

        entries.push(DiffEntry {
            relative_path: rel.clone(),
            action,
            direction,
            source_size: src.map(|s| s.size),
            target_size: tgt.map(|t| t.size),
            source_hash: src.map(|s| s.hash.clone()),
            target_hash: tgt.map(|t| t.hash.clone()),
            source_modified: src.map(|s| s.modified),
            target_modified: tgt.map(|t| t.modified),
        });
    }

    entries.sort_by(|a, b| a.relative_path.cmp(&b.relative_path));

    let diff = DiffResult {
        profile_id: profile_id.to_string(),
        entries,
        total_add,
        total_remove,
        total_update,
        total_conflict,
        total_unchanged,
        bytes_to_transfer,
    };

    Ok((diff, conflicts))
}

pub fn execute_two_way_sync(
    diff: &DiffResult,
    resolutions: &[ConflictResolution],
    source: &Path,
    target: &Path,
    cancel_flag: Arc<AtomicBool>,
    channel: &Channel<ProgressEvent>,
) -> Result<usize, AppError> {
    let resolution_map: HashMap<&str, &Resolution> = resolutions
        .iter()
        .map(|r| (r.relative_path.as_str(), &r.resolution))
        .collect();

    let actionable: Vec<_> = diff
        .entries
        .iter()
        .filter(|e| !matches!(e.action, DiffAction::Unchanged))
        .collect();

    let total_files = actionable.len();
    let total_bytes: u64 = actionable
        .iter()
        .map(|e| e.source_size.or(e.target_size).unwrap_or(0))
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

        let _ = channel.send(ProgressEvent::SyncProgress {
            files_completed,
            total_files,
            bytes_completed,
            total_bytes,
            current_file: entry.relative_path.clone(),
        });

        let result = match entry.action {
            DiffAction::Conflict => {
                if let Some(resolution) = resolution_map.get(entry.relative_path.as_str()) {
                    apply_resolution(source, target, &entry.relative_path, resolution)
                } else {
                    Ok(()) // skip unresolved
                }
            }
            DiffAction::Add | DiffAction::Update => match entry.direction {
                DiffDirection::SourceToTarget => {
                    copy_file_safe(&source.join(&entry.relative_path), &target.join(&entry.relative_path))
                }
                DiffDirection::TargetToSource => {
                    copy_file_safe(&target.join(&entry.relative_path), &source.join(&entry.relative_path))
                }
                DiffDirection::Both => Ok(()),
            },
            DiffAction::Remove => match entry.direction {
                DiffDirection::SourceToTarget => {
                    // Delete was on target side, propagate: remove from source
                    remove_if_exists(&source.join(&entry.relative_path))
                }
                DiffDirection::TargetToSource => {
                    // Delete was on source side, propagate: remove from target
                    remove_if_exists(&target.join(&entry.relative_path))
                }
                DiffDirection::Both => Ok(()),
            },
            _ => Ok(()),
        };

        if let Err(e) = result {
            let _ = channel.send(ProgressEvent::SyncError {
                file: entry.relative_path.clone(),
                error: e.to_string(),
            });
        }

        bytes_completed += entry.source_size.or(entry.target_size).unwrap_or(0);
        files_completed += 1;
    }

    let duration_ms = start.elapsed().as_millis() as u64;
    let _ = channel.send(ProgressEvent::SyncComplete {
        files_synced: files_completed,
        duration_ms,
    });

    Ok(files_completed)
}

fn apply_resolution(
    source: &Path,
    target: &Path,
    relative_path: &str,
    resolution: &Resolution,
) -> Result<(), AppError> {
    let src = source.join(relative_path);
    let tgt = target.join(relative_path);

    match resolution {
        Resolution::KeepSource => {
            if src.exists() {
                copy_file_safe(&src, &tgt)
            } else {
                remove_if_exists(&tgt)
            }
        }
        Resolution::KeepTarget => {
            if tgt.exists() {
                copy_file_safe(&tgt, &src)
            } else {
                remove_if_exists(&src)
            }
        }
        Resolution::KeepBoth => {
            // Rename target with suffix, copy source to target
            if src.exists() && tgt.exists() {
                let stem = tgt
                    .file_stem()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string();
                let ext = tgt
                    .extension()
                    .map(|e| format!(".{}", e.to_string_lossy()))
                    .unwrap_or_default();
                let conflict_name = format!("{}_conflict{}", stem, ext);
                let conflict_path = tgt.with_file_name(&conflict_name);
                std::fs::rename(&tgt, &conflict_path)?;
                copy_file_safe(&src, &tgt)?;
                // Also copy conflict version to source side
                let src_conflict = source.join(
                    std::path::Path::new(relative_path)
                        .with_file_name(&conflict_name),
                );
                copy_file_safe(&conflict_path, &src_conflict)?;
            }
            Ok(())
        }
        Resolution::Skip => Ok(()),
    }
}

fn remove_if_exists(path: &Path) -> Result<(), AppError> {
    if path.exists() {
        std::fs::remove_file(path)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn write_file(dir: &Path, rel: &str, content: &[u8]) {
        let path = dir.join(rel);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(&path, content).unwrap();
    }

    fn make_baseline(rel: &str, hash: &str) -> FileBaseline {
        FileBaseline {
            relative_path: rel.to_string(),
            source_hash: Some(hash.to_string()),
            target_hash: Some(hash.to_string()),
            source_modified: Some(1000),
            target_modified: Some(1000),
            source_size: Some(100),
            target_size: Some(100),
        }
    }

    #[test]
    fn test_two_way_new_on_source_no_baseline() {
        let source = tempfile::tempdir().unwrap();
        let target = tempfile::tempdir().unwrap();
        write_file(source.path(), "new.flac", b"new track");

        let baselines = HashMap::new();
        let (diff, conflicts) = compute_two_way_diff(
            "test",
            source.path(),
            target.path(),
            &[],
            &baselines,
        )
        .unwrap();

        assert_eq!(diff.total_add, 1);
        assert_eq!(conflicts.len(), 0);
        let entry = diff.entries.iter().find(|e| e.relative_path == "new.flac").unwrap();
        assert_eq!(entry.action, DiffAction::Add);
        assert_eq!(entry.direction, DiffDirection::SourceToTarget);
    }

    #[test]
    fn test_two_way_new_on_target_no_baseline() {
        let source = tempfile::tempdir().unwrap();
        let target = tempfile::tempdir().unwrap();
        write_file(target.path(), "new.flac", b"new track on target");

        let baselines = HashMap::new();
        let (diff, conflicts) = compute_two_way_diff(
            "test",
            source.path(),
            target.path(),
            &[],
            &baselines,
        )
        .unwrap();

        assert_eq!(diff.total_add, 1);
        assert_eq!(conflicts.len(), 0);
        let entry = diff.entries.iter().find(|e| e.relative_path == "new.flac").unwrap();
        assert_eq!(entry.action, DiffAction::Add);
        assert_eq!(entry.direction, DiffDirection::TargetToSource);
    }

    #[test]
    fn test_two_way_conflict_first_sync() {
        let source = tempfile::tempdir().unwrap();
        let target = tempfile::tempdir().unwrap();
        write_file(source.path(), "track.flac", b"version A");
        write_file(target.path(), "track.flac", b"version B");

        let baselines = HashMap::new();
        let (diff, conflicts) = compute_two_way_diff(
            "test",
            source.path(),
            target.path(),
            &[],
            &baselines,
        )
        .unwrap();

        assert_eq!(diff.total_conflict, 1);
        assert_eq!(conflicts.len(), 1);
        assert_eq!(conflicts[0].conflict_type, ConflictType::FirstSyncDiffers);
    }

    #[test]
    fn test_two_way_identical_first_sync() {
        let source = tempfile::tempdir().unwrap();
        let target = tempfile::tempdir().unwrap();
        write_file(source.path(), "track.flac", b"same content");
        write_file(target.path(), "track.flac", b"same content");

        let baselines = HashMap::new();
        let (diff, conflicts) = compute_two_way_diff(
            "test",
            source.path(),
            target.path(),
            &[],
            &baselines,
        )
        .unwrap();

        assert_eq!(diff.total_unchanged, 1);
        assert_eq!(conflicts.len(), 0);
    }

    #[test]
    fn test_two_way_source_changed_with_baseline() {
        let source = tempfile::tempdir().unwrap();
        let target = tempfile::tempdir().unwrap();
        write_file(source.path(), "track.flac", b"updated source content");
        write_file(target.path(), "track.flac", b"original content");

        // Baseline had the original hash for both
        let orig_hash = {
            let mut h = blake3::Hasher::new();
            h.update(b"original content");
            h.finalize().to_hex().to_string()
        };

        let mut baselines = HashMap::new();
        baselines.insert("track.flac".to_string(), make_baseline("track.flac", &orig_hash));

        let (diff, conflicts) = compute_two_way_diff(
            "test",
            source.path(),
            target.path(),
            &[],
            &baselines,
        )
        .unwrap();

        assert_eq!(diff.total_update, 1);
        assert_eq!(conflicts.len(), 0);
        let entry = diff.entries.iter().find(|e| e.relative_path == "track.flac").unwrap();
        assert_eq!(entry.direction, DiffDirection::SourceToTarget);
    }

    #[test]
    fn test_two_way_both_changed_conflict() {
        let source = tempfile::tempdir().unwrap();
        let target = tempfile::tempdir().unwrap();
        write_file(source.path(), "track.flac", b"source changed");
        write_file(target.path(), "track.flac", b"target changed");

        let orig_hash = {
            let mut h = blake3::Hasher::new();
            h.update(b"original");
            h.finalize().to_hex().to_string()
        };

        let mut baselines = HashMap::new();
        baselines.insert("track.flac".to_string(), make_baseline("track.flac", &orig_hash));

        let (diff, conflicts) = compute_two_way_diff(
            "test",
            source.path(),
            target.path(),
            &[],
            &baselines,
        )
        .unwrap();

        assert_eq!(diff.total_conflict, 1);
        assert_eq!(conflicts.len(), 1);
        assert_eq!(conflicts[0].conflict_type, ConflictType::BothModified);
    }

    #[test]
    fn test_two_way_deleted_source_unchanged_target() {
        let source = tempfile::tempdir().unwrap();
        let target = tempfile::tempdir().unwrap();
        // Only exists in target (was deleted from source)
        write_file(target.path(), "track.flac", b"original content");

        let orig_hash = {
            let mut h = blake3::Hasher::new();
            h.update(b"original content");
            h.finalize().to_hex().to_string()
        };

        let mut baselines = HashMap::new();
        baselines.insert("track.flac".to_string(), make_baseline("track.flac", &orig_hash));

        let (diff, _conflicts) = compute_two_way_diff(
            "test",
            source.path(),
            target.path(),
            &[],
            &baselines,
        )
        .unwrap();

        // Deleted from source, target unchanged → propagate delete to target
        assert_eq!(diff.total_remove, 1);
        let entry = diff.entries.iter().find(|e| e.relative_path == "track.flac").unwrap();
        assert_eq!(entry.action, DiffAction::Remove);
        assert_eq!(entry.direction, DiffDirection::TargetToSource);
    }
}
