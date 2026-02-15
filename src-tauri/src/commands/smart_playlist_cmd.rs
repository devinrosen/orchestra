use std::sync::Mutex;
use rusqlite::Connection;

use crate::db::smart_playlist_repo;
use crate::error::AppError;
use crate::models::smart_playlist::{
    CreateSmartPlaylistRequest, SmartPlaylist, SmartPlaylistWithTracks, UpdateSmartPlaylistRequest,
};

#[tauri::command]
pub async fn create_smart_playlist(
    db: tauri::State<'_, Mutex<Connection>>,
    request: CreateSmartPlaylistRequest,
) -> Result<SmartPlaylistWithTracks, AppError> {
    let now = chrono::Utc::now().timestamp();
    let sp = SmartPlaylist {
        id: uuid::Uuid::new_v4().to_string(),
        name: request.name,
        rule: request.rule,
        created_at: now,
        updated_at: now,
    };

    let conn = db.lock().map_err(|e| AppError::General(e.to_string()))?;
    smart_playlist_repo::create_smart_playlist(&conn, &sp)?;
    // Return the playlist with an empty track list — caller can evaluate separately
    Ok(SmartPlaylistWithTracks {
        playlist: sp,
        tracks: vec![],
    })
}

#[tauri::command]
pub async fn list_smart_playlists(
    db: tauri::State<'_, Mutex<Connection>>,
) -> Result<Vec<SmartPlaylist>, AppError> {
    let conn = db.lock().map_err(|e| AppError::General(e.to_string()))?;
    smart_playlist_repo::list_smart_playlists(&conn)
}

#[tauri::command]
pub async fn get_smart_playlist(
    db: tauri::State<'_, Mutex<Connection>>,
    id: String,
) -> Result<SmartPlaylistWithTracks, AppError> {
    let conn = db.lock().map_err(|e| AppError::General(e.to_string()))?;
    smart_playlist_repo::evaluate_smart_playlist(&conn, &id)
}

#[tauri::command]
pub async fn update_smart_playlist(
    db: tauri::State<'_, Mutex<Connection>>,
    request: UpdateSmartPlaylistRequest,
) -> Result<SmartPlaylist, AppError> {
    let conn = db.lock().map_err(|e| AppError::General(e.to_string()))?;
    let mut sp = smart_playlist_repo::get_smart_playlist(&conn, &request.id)?;

    if let Some(name) = request.name {
        sp.name = name;
    }
    if let Some(rule) = request.rule {
        sp.rule = rule;
    }
    sp.updated_at = chrono::Utc::now().timestamp();

    smart_playlist_repo::update_smart_playlist(&conn, &sp)?;
    Ok(sp)
}

#[tauri::command]
pub async fn delete_smart_playlist(
    db: tauri::State<'_, Mutex<Connection>>,
    id: String,
) -> Result<(), AppError> {
    let conn = db.lock().map_err(|e| AppError::General(e.to_string()))?;
    smart_playlist_repo::delete_smart_playlist(&conn, &id)
}

#[tauri::command]
pub async fn evaluate_smart_playlist(
    db: tauri::State<'_, Mutex<Connection>>,
    id: String,
) -> Result<SmartPlaylistWithTracks, AppError> {
    let conn = db.lock().map_err(|e| AppError::General(e.to_string()))?;
    smart_playlist_repo::evaluate_smart_playlist(&conn, &id)
}
