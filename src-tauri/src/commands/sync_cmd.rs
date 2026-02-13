use std::path::Path;
use std::sync::Mutex;
use rusqlite::Connection;
use tauri::ipc::Channel;

use crate::db::{profile_repo, sync_state_repo};
use crate::error::AppError;
use crate::models::conflict::{Conflict, ConflictResolution};
use crate::models::diff::DiffResult;
use crate::models::progress::ProgressEvent;
use crate::models::sync_profile::SyncMode;
use crate::sync::progress::CancelToken;
use crate::sync::{diff, one_way, two_way};

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
            let result = diff::compute_one_way_diff(
                &profile_id,
                source,
                target,
                &profile.exclude_patterns,
            )?;
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
    use crate::models::track::is_audio_file;
    use crate::scanner::hasher;
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
