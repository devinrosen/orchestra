use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tauri::ipc::Channel;

use orchestra_core::error::AppError;
use orchestra_core::models::diff::{DiffAction, DiffResult};
use orchestra_core::models::progress::ProgressEvent;

pub fn execute_one_way_sync(
    diff: &DiffResult,
    source: &Path,
    target: &Path,
    cancel_flag: Arc<AtomicBool>,
    channel: &Channel<ProgressEvent>,
) -> Result<usize, AppError> {
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

        let _ = channel.send(ProgressEvent::SyncProgress {
            files_completed,
            total_files,
            bytes_completed,
            total_bytes,
            current_file: entry.relative_path.clone(),
        });

        let result = match entry.action {
            DiffAction::Add | DiffAction::Update => {
                let src_path = source.join(&entry.relative_path);
                let tgt_path = target.join(&entry.relative_path);
                copy_file_safe(&src_path, &tgt_path)
            }
            DiffAction::Remove => {
                let tgt_path = target.join(&entry.relative_path);
                remove_file_safe(&tgt_path)
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

    Ok(files_completed)
}

pub fn copy_file_safe(src: &Path, dst: &Path) -> Result<(), AppError> {
    if let Some(parent) = dst.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let tmp_path = dst.with_extension("tmp_sync");
    std::fs::copy(src, &tmp_path)?;

    // fsync the temp file
    let file = std::fs::File::open(&tmp_path)?;
    file.sync_all()?;
    drop(file);

    // atomic rename
    std::fs::rename(&tmp_path, dst)?;

    // preserve modification time from source
    let src_meta = std::fs::metadata(src)?;
    if let Ok(mtime) = src_meta.modified() {
        let _ = filetime_set(dst, mtime);
    }

    Ok(())
}

fn filetime_set(path: &Path, mtime: std::time::SystemTime) -> Result<(), AppError> {
    // Use file's set_modified via open
    let file = std::fs::OpenOptions::new().write(true).open(path)?;
    file.set_modified(mtime)?;
    Ok(())
}

fn remove_file_safe(path: &Path) -> Result<(), AppError> {
    if path.exists() {
        std::fs::remove_file(path)?;
        // Clean up empty parent directories
        if let Some(parent) = path.parent() {
            let _ = remove_empty_parents(parent);
        }
    }
    Ok(())
}

pub fn remove_empty_parents(dir: &Path) -> Result<(), std::io::Error> {
    if dir.is_dir() {
        if std::fs::read_dir(dir)?.next().is_none() {
            std::fs::remove_dir(dir)?;
            if let Some(parent) = dir.parent() {
                let _ = remove_empty_parents(parent);
            }
        }
    }
    Ok(())
}
