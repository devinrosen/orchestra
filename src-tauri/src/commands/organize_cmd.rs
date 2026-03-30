use rusqlite::Connection;
use std::sync::atomic::Ordering;
use std::sync::Mutex;
use tauri::ipc::Channel;

use orchestra_core::db::organization_repo;
use orchestra_core::error::AppError;
use orchestra_core::models::organization::{
    OrganizeApplyItem, OrganizeApplyResult, OrganizeItemStatus, OrganizePreviewItem,
    OrganizePreviewResult,
};
use orchestra_core::models::progress::ProgressEvent;
use orchestra_core::organizer;

use crate::sync::progress::CancelToken;

/// Computes proposed rename paths for every track in the library and returns a preview.
#[tauri::command]
pub async fn preview_organize(
    db: tauri::State<'_, Mutex<Connection>>,
    library_root: String,
    pattern: String,
) -> Result<OrganizePreviewResult, AppError> {
    let tracks = {
        let conn = db.lock().map_err(|e| AppError::General(e.to_string()))?;
        organization_repo::get_all_tracks_for_organize(&conn, &library_root)?
    };

    // Build map: proposed_relative_path -> list of track_ids that would land there
    let mut proposed_map: std::collections::HashMap<String, Vec<i64>> =
        std::collections::HashMap::new();

    // First pass: compute all proposed paths
    let mut items: Vec<OrganizePreviewItem> = Vec::with_capacity(tracks.len());
    for track in &tracks {
        let track_id = track.id.unwrap_or(0);
        let proposed = organizer::apply_pattern(&pattern, track);
        proposed_map
            .entry(proposed.clone())
            .or_default()
            .push(track_id);
        items.push(OrganizePreviewItem {
            track_id,
            current_relative_path: track.relative_path.clone(),
            proposed_relative_path: proposed,
            status: OrganizeItemStatus::Ok,
        });
    }

    // Second pass: assign statuses
    let mut already_correct = 0usize;
    let mut collisions = 0usize;

    for item in &mut items {
        let bucket_size = proposed_map
            .get(&item.proposed_relative_path)
            .map(|v| v.len())
            .unwrap_or(0);

        if item.proposed_relative_path == item.current_relative_path {
            item.status = OrganizeItemStatus::AlreadyCorrect;
            already_correct += 1;
        } else if bucket_size > 1 {
            let conflicting = proposed_map
                .get(&item.proposed_relative_path)
                .and_then(|v| v.iter().find(|&&id| id != item.track_id))
                .copied();
            item.status = OrganizeItemStatus::Collision {
                conflicting_track_id: conflicting,
            };
            collisions += 1;
        }
    }

    let total = items.len();

    Ok(OrganizePreviewResult {
        items,
        total,
        already_correct,
        collisions,
        errors: 0,
    })
}

/// Applies a batch of file moves, updates the database, and streams progress events.
#[tauri::command]
pub async fn apply_organize(
    db: tauri::State<'_, Mutex<Connection>>,
    cancel_token: tauri::State<'_, Mutex<CancelToken>>,
    library_root: String,
    items: Vec<OrganizeApplyItem>,
    on_progress: Channel<ProgressEvent>,
) -> Result<OrganizeApplyResult, AppError> {
    let flag = {
        let token = cancel_token
            .lock()
            .map_err(|e| AppError::General(e.to_string()))?;
        token.flag()
    };

    let total = items.len();
    let mut moved = 0usize;
    let mut skipped = 0usize;
    let mut errors: Vec<String> = Vec::new();
    let mut db_updates: Vec<(i64, String, String)> = Vec::new();

    let start = std::time::Instant::now();
    let root = library_root.trim_end_matches('/');

    for (i, item) in items.iter().enumerate() {
        if flag.load(Ordering::Relaxed) {
            break;
        }

        let new_file_path = format!("{}/{}", root, item.proposed_relative_path);
        let new_path = std::path::Path::new(&new_file_path);
        let src = std::path::Path::new(&item.current_file_path);

        // Skip if source equals destination (already correct)
        if item.current_file_path == new_file_path {
            skipped += 1;
            continue;
        }

        // Ensure destination directory exists
        if let Some(parent) = new_path.parent() {
            if let Err(e) = std::fs::create_dir_all(parent) {
                errors.push(format!("{}: {}", item.current_file_path, e));
                skipped += 1;
                continue;
            }
        }

        match try_move_file(src, new_path) {
            Ok(()) => {
                moved += 1;
                db_updates.push((
                    item.track_id,
                    new_file_path,
                    item.proposed_relative_path.clone(),
                ));
            }
            Err(e) => {
                errors.push(format!("{}: {}", item.current_file_path, e));
                skipped += 1;
            }
        }

        let _ = on_progress.send(ProgressEvent::OrganizeProgress {
            completed: i + 1,
            total,
            current_file: item.proposed_relative_path.clone(),
        });
    }

    // Batch-update DB paths in a single transaction
    if !db_updates.is_empty() {
        let conn = db.lock().map_err(|e| AppError::General(e.to_string()))?;
        organization_repo::bulk_update_track_paths(&conn, &db_updates)?;
    }

    let duration_ms = start.elapsed().as_millis() as u64;
    let _ = on_progress.send(ProgressEvent::OrganizeComplete { moved, duration_ms });

    Ok(OrganizeApplyResult {
        moved,
        skipped,
        errors,
    })
}

/// Tries an atomic rename first (same filesystem); falls back to copy-then-rename for
/// cross-device moves, using the safe-write pattern (copy → fsync → rename → remove source).
fn try_move_file(src: &std::path::Path, dst: &std::path::Path) -> Result<(), std::io::Error> {
    // Attempt atomic rename (fast path, same filesystem)
    if std::fs::rename(src, dst).is_ok() {
        return Ok(());
    }

    // Copy-then-rename fallback (handles cross-device moves)
    let tmp = dst.with_extension("tmp_organize");
    std::fs::copy(src, &tmp)?;
    {
        let f = std::fs::File::open(&tmp)?;
        f.sync_all()?;
    }
    std::fs::rename(&tmp, dst)?;
    std::fs::remove_file(src)?;

    Ok(())
}
