use std::collections::{HashMap, HashSet};
use std::path::Path;

use crate::error::AppError;
use crate::models::diff::{DiffAction, DiffDirection, DiffEntry, DiffResult};
use crate::models::track::is_audio_file;
use crate::scanner::hasher;

struct FileInfo {
    size: u64,
    modified: i64,
    hash: Option<String>,
}

fn collect_files(root: &Path, exclude_patterns: &[String]) -> Result<HashMap<String, FileInfo>, AppError> {
    let compiled: Vec<glob::Pattern> = exclude_patterns
        .iter()
        .filter_map(|p| glob::Pattern::new(p).ok())
        .collect();

    let mut files = HashMap::new();
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

        files.insert(
            rel,
            FileInfo {
                size: meta.len(),
                modified,
                hash: None,
            },
        );
    }
    Ok(files)
}

fn compute_hash_if_needed(root: &Path, rel: &str, info: &mut FileInfo) -> Result<String, AppError> {
    if let Some(ref h) = info.hash {
        return Ok(h.clone());
    }
    let hash = hasher::hash_file(&root.join(rel))?;
    info.hash = Some(hash.clone());
    Ok(hash)
}

pub fn compute_one_way_diff(
    profile_id: &str,
    source: &Path,
    target: &Path,
    exclude_patterns: &[String],
) -> Result<DiffResult, AppError> {
    let mut source_files = collect_files(source, exclude_patterns)?;
    let mut target_files = collect_files(target, exclude_patterns)?;

    let all_keys: HashSet<String> = source_files
        .keys()
        .chain(target_files.keys())
        .cloned()
        .collect();

    let mut entries = Vec::new();
    let mut total_add = 0usize;
    let mut total_remove = 0usize;
    let mut total_update = 0usize;
    let mut total_unchanged = 0usize;
    let mut bytes_to_transfer = 0u64;

    for rel in all_keys {
        match (source_files.get_mut(&rel), target_files.get_mut(&rel)) {
            (Some(src), None) => {
                let hash = compute_hash_if_needed(source, &rel, src)?;
                bytes_to_transfer += src.size;
                total_add += 1;
                entries.push(DiffEntry {
                    relative_path: rel,
                    action: DiffAction::Add,
                    direction: DiffDirection::SourceToTarget,
                    source_size: Some(src.size),
                    target_size: None,
                    source_hash: Some(hash),
                    target_hash: None,
                    source_modified: Some(src.modified),
                    target_modified: None,
                });
            }
            (None, Some(tgt)) => {
                total_remove += 1;
                entries.push(DiffEntry {
                    relative_path: rel,
                    action: DiffAction::Remove,
                    direction: DiffDirection::SourceToTarget,
                    source_size: None,
                    target_size: Some(tgt.size),
                    source_hash: None,
                    target_hash: None,
                    source_modified: None,
                    target_modified: Some(tgt.modified),
                });
            }
            (Some(src), Some(tgt)) => {
                if src.size == tgt.size && src.modified == tgt.modified {
                    total_unchanged += 1;
                    entries.push(DiffEntry {
                        relative_path: rel,
                        action: DiffAction::Unchanged,
                        direction: DiffDirection::SourceToTarget,
                        source_size: Some(src.size),
                        target_size: Some(tgt.size),
                        source_hash: None,
                        target_hash: None,
                        source_modified: Some(src.modified),
                        target_modified: Some(tgt.modified),
                    });
                } else {
                    let src_hash = compute_hash_if_needed(source, &rel, src)?;
                    let tgt_hash = compute_hash_if_needed(target, &rel, tgt)?;
                    if src_hash == tgt_hash {
                        total_unchanged += 1;
                        entries.push(DiffEntry {
                            relative_path: rel,
                            action: DiffAction::Unchanged,
                            direction: DiffDirection::SourceToTarget,
                            source_size: Some(src.size),
                            target_size: Some(tgt.size),
                            source_hash: Some(src_hash),
                            target_hash: Some(tgt_hash),
                            source_modified: Some(src.modified),
                            target_modified: Some(tgt.modified),
                        });
                    } else {
                        bytes_to_transfer += src.size;
                        total_update += 1;
                        entries.push(DiffEntry {
                            relative_path: rel,
                            action: DiffAction::Update,
                            direction: DiffDirection::SourceToTarget,
                            source_size: Some(src.size),
                            target_size: Some(tgt.size),
                            source_hash: Some(src_hash),
                            target_hash: Some(tgt_hash),
                            source_modified: Some(src.modified),
                            target_modified: Some(tgt.modified),
                        });
                    }
                }
            }
            (None, None) => unreachable!(),
        }
    }

    entries.sort_by(|a, b| a.relative_path.cmp(&b.relative_path));

    Ok(DiffResult {
        profile_id: profile_id.to_string(),
        entries,
        total_add,
        total_remove,
        total_update,
        total_conflict: 0,
        total_unchanged,
        bytes_to_transfer,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn setup_test_dirs() -> (tempfile::TempDir, tempfile::TempDir) {
        let source = tempfile::tempdir().unwrap();
        let target = tempfile::tempdir().unwrap();
        (source, target)
    }

    fn write_fake_audio(dir: &Path, rel: &str, content: &[u8]) {
        let path = dir.join(rel);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(&path, content).unwrap();
    }

    #[test]
    fn test_one_way_add() {
        let (source, target) = setup_test_dirs();
        write_fake_audio(source.path(), "artist/album/track.flac", b"source audio data");

        let result =
            compute_one_way_diff("test", source.path(), target.path(), &[]).unwrap();

        assert_eq!(result.total_add, 1);
        assert_eq!(result.total_remove, 0);
        assert_eq!(result.total_update, 0);
        assert_eq!(result.entries.len(), 1);
        assert_eq!(result.entries[0].action, DiffAction::Add);
    }

    #[test]
    fn test_one_way_remove() {
        let (source, target) = setup_test_dirs();
        write_fake_audio(target.path(), "old/track.mp3", b"old data");

        let result =
            compute_one_way_diff("test", source.path(), target.path(), &[]).unwrap();

        assert_eq!(result.total_add, 0);
        assert_eq!(result.total_remove, 1);
        assert_eq!(result.entries[0].action, DiffAction::Remove);
    }

    #[test]
    fn test_one_way_unchanged() {
        let (source, target) = setup_test_dirs();
        write_fake_audio(source.path(), "track.flac", b"same data");
        write_fake_audio(target.path(), "track.flac", b"same data");

        let result =
            compute_one_way_diff("test", source.path(), target.path(), &[]).unwrap();

        // Even though mtimes differ, content hashes match so it's unchanged
        assert_eq!(result.total_unchanged, 1);
    }

    #[test]
    fn test_one_way_update() {
        let (source, target) = setup_test_dirs();
        write_fake_audio(source.path(), "track.flac", b"new version of the track");
        write_fake_audio(target.path(), "track.flac", b"old version");

        let result =
            compute_one_way_diff("test", source.path(), target.path(), &[]).unwrap();

        assert_eq!(result.total_update, 1);
        assert_eq!(result.entries[0].action, DiffAction::Update);
    }

    #[test]
    fn test_non_audio_files_ignored() {
        let (source, target) = setup_test_dirs();
        write_fake_audio(source.path(), "readme.txt", b"not audio");
        write_fake_audio(source.path(), "track.flac", b"audio");

        let result =
            compute_one_way_diff("test", source.path(), target.path(), &[]).unwrap();

        assert_eq!(result.total_add, 1); // only the .flac
    }

    #[test]
    fn test_exclude_patterns() {
        let (source, target) = setup_test_dirs();
        write_fake_audio(source.path(), "good.flac", b"keep");
        write_fake_audio(source.path(), "skip.flac", b"exclude");

        let result = compute_one_way_diff(
            "test",
            source.path(),
            target.path(),
            &["skip.*".to_string()],
        )
        .unwrap();

        assert_eq!(result.total_add, 1);
        assert_eq!(result.entries[0].relative_path, "good.flac");
    }
}
