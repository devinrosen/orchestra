use std::sync::Mutex;

use rusqlite::Connection;

use crate::db::recent_repo;
use crate::error::AppError;
use crate::models::track::Track;

#[tauri::command]
pub async fn record_play(
    db: tauri::State<'_, Mutex<Connection>>,
    track_id: i64,
) -> Result<(), AppError> {
    let conn = db.lock().map_err(|e| AppError::General(e.to_string()))?;
    recent_repo::record_play(&conn, track_id)
}

#[tauri::command]
pub async fn get_recently_added(
    db: tauri::State<'_, Mutex<Connection>>,
    days: u32,
    limit: usize,
) -> Result<Vec<Track>, AppError> {
    let conn = db.lock().map_err(|e| AppError::General(e.to_string()))?;
    recent_repo::get_recently_added(&conn, days, limit)
}

#[tauri::command]
pub async fn get_recently_played(
    db: tauri::State<'_, Mutex<Connection>>,
    limit: usize,
) -> Result<Vec<Track>, AppError> {
    let conn = db.lock().map_err(|e| AppError::General(e.to_string()))?;
    recent_repo::get_recently_played(&conn, limit)
}
