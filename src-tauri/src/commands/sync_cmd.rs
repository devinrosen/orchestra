use rusqlite::Connection;
use std::path::Path;
use std::sync::Mutex;
use tauri::ipc::Channel;

use crate::sync::progress::CancelToken;
use crate::sync::{diff, one_way, two_way};
use orchestra_core::db::{profile_repo, sync_state_repo};
use orchestra_core::error::AppError;
use orchestra_core::models::conflict::{Conflict, ConflictResolution};
use orchestra_core::models::diff::DiffResult;
use orchestra_core::models::progress::ProgressEvent;
use orchestra_core::models::sync_profile::SyncMode;

#[tauri::command]
pub async fn compute_diff(
    db: tauri::State<'_, Mutex<Connection>>,
    profile_id: String,
) -> Result<(DiffResult, Vec<Conflict>), AppError> {
    let (profile, baselines) = {
        let conn = db.lock().map_err(|e| AppError::General(e.to_string()))?;
        let profile = profile_repo::get_profile(&conn, &profile_id)?;
        let baselines = sync_state_repo::get_baselines(&conn, &profile_id)?;
        (profile, baselines)
    };

    let source = Path::new(&profile.source_path);
    let target = Path::new(&profile.target_path);

    if !source.exists() {
        return Err(AppError::PathNotAccessible(profile.source_path.clone()));
    }
    if !target.exists() {
        return Err(AppError::PathNotAccessible(profile.target_path.clone()));
    }

    match profile.sync_mode {
        SyncMode::OneWay => {
            let result =
                diff::compute_one_way_diff(&profile_id, source, target, &profile.exclude_patterns)?;
            Ok((result, vec![]))
        }
        SyncMode::TwoWay => two_way::compute_two_way_diff(
            &profile_id,
            source,
            target,
            &profile.exclude_patterns,
            &baselines,
        ),
    }
}

#[tauri::command]
pub async fn execute_sync(
    db: tauri::State<'_, Mutex<Connection>>,
    cancel_token: tauri::State<'_, Mutex<CancelToken>>,
    profile_id: String,
    diff_result: DiffResult,
    conflict_resolutions: Vec<ConflictResolution>,
    on_progress: Channel<ProgressEvent>,
) -> Result<usize, AppError> {
    let profile = {
        let conn = db.lock().map_err(|e| AppError::General(e.to_string()))?;
        profile_repo::get_profile(&conn, &profile_id)?
    };

    let source = Path::new(&profile.source_path);
    let target = Path::new(&profile.target_path);

    // Reset cancel token
    let flag = {
        let token = cancel_token
            .lock()
            .map_err(|e| AppError::General(e.to_string()))?;
        token.flag()
    };

    let count = match profile.sync_mode {
        SyncMode::OneWay => {
            one_way::execute_one_way_sync(&diff_result, source, target, flag, &on_progress)?
        }
        SyncMode::TwoWay => two_way::execute_two_way_sync(
            &diff_result,
            &conflict_resolutions,
            source,
            target,
            flag,
            &on_progress,
        )?,
    };

    // Save baselines and update last_synced
    {
        let conn = db.lock().map_err(|e| AppError::General(e.to_string()))?;
        let now = chrono::Utc::now().timestamp();
        profile_repo::update_last_synced(&conn, &profile_id, now)?;

        // Build baselines from current state of source and target
        let baselines = build_post_sync_baselines(source, target, &profile.exclude_patterns)?;
        sync_state_repo::save_baselines(&conn, &profile_id, &baselines)?;
    }

    Ok(count)
}

#[tauri::command]
pub async fn cancel_sync(
    cancel_token: tauri::State<'_, Mutex<CancelToken>>,
) -> Result<(), AppError> {
    let token = cancel_token
        .lock()
        .map_err(|e| AppError::General(e.to_string()))?;
    token.cancel();
    Ok(())
}

fn build_post_sync_baselines(
    source: &Path,
    target: &Path,
    exclude_patterns: &[String],
) -> Result<Vec<sync_state_repo::FileBaseline>, AppError> {
    use orchestra_core::models::track::is_audio_file;
    use orchestra_core::scanner::hasher;
    use std::collections::HashSet;

    let compiled: Vec<glob::Pattern> = exclude_patterns
        .iter()
        .filter_map(|p| glob::Pattern::new(p).ok())
        .collect();

    let mut all_rels = HashSet::new();

    // Collect from source
    let mut source_info: std::collections::HashMap<String, (String, i64, u64)> =
        std::collections::HashMap::new();
    for entry in walkdir::WalkDir::new(source).follow_links(true) {
        let entry = entry?;
        if !entry.file_type().is_file() || !is_audio_file(entry.path()) {
            continue;
        }
        let rel = entry
            .path()
            .strip_prefix(source)
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
        all_rels.insert(rel.clone());
        source_info.insert(rel, (hash, modified, meta.len()));
    }

    // Collect from target
    let mut target_info: std::collections::HashMap<String, (String, i64, u64)> =
        std::collections::HashMap::new();
    for entry in walkdir::WalkDir::new(target).follow_links(true) {
        let entry = entry?;
        if !entry.file_type().is_file() || !is_audio_file(entry.path()) {
            continue;
        }
        let rel = entry
            .path()
            .strip_prefix(target)
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
        all_rels.insert(rel.clone());
        target_info.insert(rel, (hash, modified, meta.len()));
    }

    let baselines: Vec<sync_state_repo::FileBaseline> = all_rels
        .into_iter()
        .map(|rel| {
            let src = source_info.get(&rel);
            let tgt = target_info.get(&rel);
            sync_state_repo::FileBaseline {
                relative_path: rel,
                source_hash: src.map(|(h, _, _)| h.clone()),
                target_hash: tgt.map(|(h, _, _)| h.clone()),
                source_modified: src.map(|(_, m, _)| *m),
                target_modified: tgt.map(|(_, m, _)| *m),
                source_size: src.map(|(_, _, s)| *s),
                target_size: tgt.map(|(_, _, s)| *s),
            }
        })
        .collect();

    Ok(baselines)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn write_fake_audio(dir: &TempDir, rel: &str, content: &[u8]) {
        let path = dir.path().join(rel);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(&path, content).unwrap();
    }

    #[test]
    fn test_build_post_sync_baselines_with_matching_files() {
        let src = TempDir::new().unwrap();
        let tgt = TempDir::new().unwrap();

        write_fake_audio(&src, "artist/album/track01.flac", b"source audio data 1");
        write_fake_audio(&src, "artist/album/track02.flac", b"source audio data 2");
        write_fake_audio(&tgt, "artist/album/track01.flac", b"source audio data 1");

        let baselines = build_post_sync_baselines(src.path(), tgt.path(), &[]).unwrap();

        assert_eq!(baselines.len(), 2);

        let b1 = baselines
            .iter()
            .find(|b| b.relative_path == "artist/album/track01.flac")
            .expect("track01 baseline missing");
        assert!(b1.source_hash.is_some());
        assert!(b1.target_hash.is_some());
        assert_eq!(b1.source_hash, b1.target_hash, "same content => same hash");

        let b2 = baselines
            .iter()
            .find(|b| b.relative_path == "artist/album/track02.flac")
            .expect("track02 baseline missing");
        assert!(b2.source_hash.is_some());
        assert!(b2.target_hash.is_none(), "track02 only in source");
    }

    #[test]
    fn test_build_post_sync_baselines_empty_dirs() {
        let src = TempDir::new().unwrap();
        let tgt = TempDir::new().unwrap();

        let baselines = build_post_sync_baselines(src.path(), tgt.path(), &[]).unwrap();
        assert!(baselines.is_empty());
    }

    #[test]
    fn test_build_post_sync_baselines_respects_exclude_patterns() {
        let src = TempDir::new().unwrap();
        let tgt = TempDir::new().unwrap();

        write_fake_audio(&src, "keep/track.flac", b"keep me");
        write_fake_audio(&src, "skip/track.flac", b"exclude me");

        let baselines =
            build_post_sync_baselines(src.path(), tgt.path(), &["skip/**".to_string()]).unwrap();

        assert_eq!(baselines.len(), 1);
        assert_eq!(baselines[0].relative_path, "keep/track.flac");
    }
}
