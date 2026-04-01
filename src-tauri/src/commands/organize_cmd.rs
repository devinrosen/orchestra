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
        let track_id = match track.id {
            Some(id) => id,
            None => {
                return Err(AppError::General(format!(
                    "track has no database id: {}",
                    track.file_path
                )))
            }
        };
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

    let start = std::time::Instant::now();
    let root = library_root.trim_end_matches('/');

    let root_path = std::path::Path::new(root);

    // Security: fetch authoritative file_path for every track_id from the DB so we never
    // operate on a client-supplied path that doesn't match the actual track record.
    let track_ids: Vec<i64> = items.iter().map(|it| it.track_id).collect();
    let db_paths = {
        let conn = db.lock().map_err(|e| AppError::General(e.to_string()))?;
        organization_repo::get_file_paths_by_ids(&conn, &track_ids)?
    };

    // Phase 1: move files on disk, collecting successful moves for a single batch DB update.
    // Using a named struct here avoids lifetime issues with borrowing `items` entries.
    struct SuccessfulMove {
        track_id: i64,
        current_file_path: String,
        new_file_path: String,
        proposed_relative_path: String,
    }
    let mut successful_moves: Vec<SuccessfulMove> = Vec::new();

    for (i, item) in items.iter().enumerate() {
        if flag.load(Ordering::Relaxed) {
            break;
        }

        // Security: use the DB-authoritative path instead of the client-supplied one.
        // Reject any item whose track_id is unknown or whose client path doesn't match.
        let db_file_path = match db_paths.get(&item.track_id) {
            Some(p) => p,
            None => {
                errors.push(format!(
                    "track_id {}: not found in database",
                    item.track_id
                ));
                skipped += 1;
                continue;
            }
        };
        if db_file_path != &item.current_file_path {
            errors.push(format!(
                "track_id {}: client path '{}' does not match database path '{}'",
                item.track_id, item.current_file_path, db_file_path
            ));
            skipped += 1;
            continue;
        }

        // Security: verify current_file_path is within library_root
        let src = std::path::Path::new(db_file_path);
        if !src.starts_with(root_path) {
            errors.push(format!(
                "{}: path is outside library root",
                db_file_path
            ));
            skipped += 1;
            continue;
        }

        // Security: verify proposed_relative_path contains no parent-dir components
        let proposed_path = std::path::Path::new(&item.proposed_relative_path);
        if proposed_path
            .components()
            .any(|c| c == std::path::Component::ParentDir)
        {
            errors.push(format!(
                "{}: proposed path contains path traversal",
                item.proposed_relative_path
            ));
            skipped += 1;
            continue;
        }

        let new_file_path = format!("{}/{}", root, item.proposed_relative_path);
        let new_path = std::path::Path::new(&new_file_path);

        // Skip if source equals destination (already correct)
        if db_file_path.as_str() == new_file_path {
            skipped += 1;
            continue;
        }

        match organizer::apply_file_move(src, new_path) {
            Ok(()) => {
                successful_moves.push(SuccessfulMove {
                    track_id: item.track_id,
                    current_file_path: db_file_path.clone(),
                    new_file_path,
                    proposed_relative_path: item.proposed_relative_path.clone(),
                });
            }
            Err(e) => {
                errors.push(format!("{}: {}", db_file_path, e));
                skipped += 1;
            }
        }

        let _ = on_progress.send(ProgressEvent::OrganizeProgress {
            completed: i + 1,
            total,
            current_file: item.proposed_relative_path.clone(),
        });
    }

    // Phase 2: commit all successful moves to the DB in a single transaction (one mutex
    // acquisition, one round-trip to SQLite instead of N).
    if !successful_moves.is_empty() {
        let updates: Vec<(i64, &str, &str)> = successful_moves
            .iter()
            .map(|m| {
                (
                    m.track_id,
                    m.new_file_path.as_str(),
                    m.proposed_relative_path.as_str(),
                )
            })
            .collect();

        match db.lock() {
            Err(e) => {
                // Mutex poisoned — roll back all moves so disk and DB remain consistent.
                let lock_err = e.to_string();
                skipped += successful_moves.len();
                for m in &successful_moves {
                    let src = std::path::Path::new(&m.current_file_path);
                    let dst = std::path::Path::new(&m.new_file_path);
                    match organizer::apply_file_move(dst, src) {
                        Ok(()) => {
                            errors.push(format!(
                                "{}: db lock failed, file rolled back: {}",
                                m.current_file_path, lock_err
                            ));
                        }
                        Err(rollback_err) => {
                            errors.push(format!(
                                "{}: db lock failed AND rollback failed — \
                                 file is at {} but DB still points to original path. \
                                 Lock error: {}; rollback error: {}",
                                m.current_file_path, m.new_file_path, lock_err, rollback_err
                            ));
                        }
                    }
                }
            }
            Ok(conn) => {
                match organization_repo::bulk_update_track_paths(&conn, &updates) {
                    Ok(()) => {
                        moved = successful_moves.len();
                    }
                    Err(db_err) => {
                        // DB update failed — attempt to reverse every file move so disk and DB
                        // remain consistent (DB still has the old paths). Log both the original
                        // error and any rollback failure so the user has full diagnostic info.
                        drop(conn);
                        skipped += successful_moves.len();
                        for m in &successful_moves {
                            let src = std::path::Path::new(&m.current_file_path);
                            let dst = std::path::Path::new(&m.new_file_path);
                            match organizer::apply_file_move(dst, src) {
                                Ok(()) => {
                                    errors.push(format!(
                                        "{}: db update failed, file rolled back: {}",
                                        m.current_file_path, db_err
                                    ));
                                }
                                Err(rollback_err) => {
                                    errors.push(format!(
                                        "{}: db update failed AND rollback failed — \
                                         file is at {} but DB still points to original path. \
                                         DB error: {}; rollback error: {}",
                                        m.current_file_path, m.new_file_path, db_err, rollback_err
                                    ));
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    let duration_ms = start.elapsed().as_millis() as u64;
    let _ = on_progress.send(ProgressEvent::OrganizeComplete { moved, duration_ms });

    Ok(OrganizeApplyResult {
        moved,
        skipped,
        errors,
    })
}
