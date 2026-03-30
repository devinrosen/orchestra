use std::sync::Mutex;

use rusqlite::{params, Connection};

use orchestra_core::cover;
use orchestra_core::error::AppError;

use crate::media_session::MediaSessionState;

#[tauri::command]
pub async fn update_now_playing(
    db: tauri::State<'_, Mutex<Connection>>,
    session: tauri::State<'_, Mutex<MediaSessionState>>,
    title: Option<String>,
    artist: Option<String>,
    album: Option<String>,
    duration_secs: Option<f64>,
    file_path: String,
) -> Result<(), AppError> {
    // Validate file_path against the library root to prevent path traversal.
    let library_root: Option<String> = {
        let conn = db
            .lock()
            .map_err(|e| AppError::General(format!("update_now_playing: db lock poisoned: {e}")))?;
        let mut stmt = conn.prepare("SELECT value FROM settings WHERE key = 'library_root'")?;
        stmt.query_row(params![], |row| row.get(0)).ok()
    };

    if let Some(root) = library_root {
        let file_path_for_check = file_path.clone();
        tauri::async_runtime::spawn_blocking(move || {
            let root_canonical =
                std::fs::canonicalize(&root).unwrap_or_else(|_| std::path::PathBuf::from(&root));
            let file_canonical = std::fs::canonicalize(&file_path_for_check).map_err(|_| {
                AppError::General(
                    "update_now_playing: invalid or inaccessible file path".to_string(),
                )
            })?;
            if !file_canonical.starts_with(&root_canonical) {
                return Err(AppError::General(
                    "update_now_playing: file path is outside library root".to_string(),
                ));
            }
            Ok::<(), AppError>(())
        })
        .await
        .map_err(|e| AppError::General(format!("update_now_playing: path check task failed: {e}")))?
        .map_err(|e| e)?;
    }

    let file_path_clone = file_path.clone();
    let cover_url = tauri::async_runtime::spawn_blocking(move || {
        cover::extract_cover(&file_path_clone, "orchestra-art.jpg")
    })
    .await
    .unwrap_or(None);

    let s = session.lock().map_err(|e| {
        AppError::General(format!(
            "update_now_playing: media session lock poisoned: {e}"
        ))
    })?;
    s.update_metadata(title, artist, album, duration_secs, cover_url);
    Ok(())
}

#[tauri::command]
pub async fn update_playback_state(
    session: tauri::State<'_, Mutex<MediaSessionState>>,
    playing: bool,
    position_secs: f64,
) -> Result<(), AppError> {
    let s = session.lock().map_err(|e| {
        AppError::General(format!(
            "update_playback_state: media session lock poisoned: {e}"
        ))
    })?;
    s.update_playback(playing, position_secs);
    Ok(())
}
