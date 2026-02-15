use std::sync::Mutex;
use rusqlite::Connection;

use crate::db::favorite_repo;
use crate::error::AppError;
use crate::models::favorite::Favorite;
use crate::models::track::Track;

#[tauri::command]
pub async fn toggle_favorite(
    db: tauri::State<'_, Mutex<Connection>>,
    entity_type: String,
    entity_id: String,
) -> Result<bool, AppError> {
    let conn = db.lock().map_err(|e| AppError::General(e.to_string()))?;
    favorite_repo::toggle_favorite(&conn, &entity_type, &entity_id)
}

#[tauri::command]
pub async fn is_favorite(
    db: tauri::State<'_, Mutex<Connection>>,
    entity_type: String,
    entity_id: String,
) -> Result<bool, AppError> {
    let conn = db.lock().map_err(|e| AppError::General(e.to_string()))?;
    favorite_repo::is_favorite(&conn, &entity_type, &entity_id)
}

#[tauri::command]
pub async fn list_favorites(
    db: tauri::State<'_, Mutex<Connection>>,
    entity_type: String,
) -> Result<Vec<Favorite>, AppError> {
    let conn = db.lock().map_err(|e| AppError::General(e.to_string()))?;
    favorite_repo::list_favorites(&conn, &entity_type)
}

#[tauri::command]
pub async fn list_all_favorites(
    db: tauri::State<'_, Mutex<Connection>>,
) -> Result<Vec<Favorite>, AppError> {
    let conn = db.lock().map_err(|e| AppError::General(e.to_string()))?;
    favorite_repo::list_all_favorites(&conn)
}

#[tauri::command]
pub async fn get_favorite_tracks(
    db: tauri::State<'_, Mutex<Connection>>,
) -> Result<Vec<Track>, AppError> {
    let conn = db.lock().map_err(|e| AppError::General(e.to_string()))?;
    favorite_repo::get_favorite_tracks(&conn)
}
