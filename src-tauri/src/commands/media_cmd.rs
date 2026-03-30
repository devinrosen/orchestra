use std::sync::Mutex;

use orchestra_core::error::AppError;

use crate::media_session::{self, MediaSessionState};

#[tauri::command]
pub async fn update_now_playing(
    session: tauri::State<'_, Mutex<MediaSessionState>>,
    title: Option<String>,
    artist: Option<String>,
    album: Option<String>,
    duration_secs: Option<f64>,
    file_path: String,
) -> Result<(), AppError> {
    let cover_url = media_session::extract_cover(&file_path);
    let s = session
        .lock()
        .map_err(|e| AppError::General(e.to_string()))?;
    s.update_metadata(title, artist, album, duration_secs, cover_url);
    Ok(())
}

#[tauri::command]
pub async fn update_playback_state(
    session: tauri::State<'_, Mutex<MediaSessionState>>,
    playing: bool,
    position_secs: f64,
) -> Result<(), AppError> {
    let s = session
        .lock()
        .map_err(|e| AppError::General(e.to_string()))?;
    s.update_playback(playing, position_secs);
    Ok(())
}
