use std::sync::Mutex;
use rusqlite::Connection;

use orchestra_core::db::profile_repo;
use orchestra_core::error::AppError;
use orchestra_core::models::sync_profile::{CreateProfileRequest, SyncProfile, UpdateProfileRequest};

#[tauri::command]
pub async fn create_profile(
    db: tauri::State<'_, Mutex<Connection>>,
    request: CreateProfileRequest,
) -> Result<SyncProfile, AppError> {
    let profile = SyncProfile {
        id: uuid::Uuid::new_v4().to_string(),
        name: request.name,
        source_path: request.source_path,
        target_path: request.target_path,
        sync_mode: request.sync_mode,
        exclude_patterns: request.exclude_patterns,
        created_at: chrono::Utc::now().timestamp(),
        last_synced_at: None,
    };

    let conn = db.lock().map_err(|e| AppError::General(e.to_string()))?;
    profile_repo::create_profile(&conn, &profile)?;
    Ok(profile)
}

#[tauri::command]
pub async fn get_profile(
    db: tauri::State<'_, Mutex<Connection>>,
    id: String,
) -> Result<SyncProfile, AppError> {
    let conn = db.lock().map_err(|e| AppError::General(e.to_string()))?;
    profile_repo::get_profile(&conn, &id)
}

#[tauri::command]
pub async fn list_profiles(
    db: tauri::State<'_, Mutex<Connection>>,
) -> Result<Vec<SyncProfile>, AppError> {
    let conn = db.lock().map_err(|e| AppError::General(e.to_string()))?;
    profile_repo::list_profiles(&conn)
}

#[tauri::command]
pub async fn update_profile(
    db: tauri::State<'_, Mutex<Connection>>,
    request: UpdateProfileRequest,
) -> Result<SyncProfile, AppError> {
    let conn = db.lock().map_err(|e| AppError::General(e.to_string()))?;
    let mut profile = profile_repo::get_profile(&conn, &request.id)?;

    if let Some(name) = request.name {
        profile.name = name;
    }
    if let Some(source_path) = request.source_path {
        profile.source_path = source_path;
    }
    if let Some(target_path) = request.target_path {
        profile.target_path = target_path;
    }
    if let Some(sync_mode) = request.sync_mode {
        profile.sync_mode = sync_mode;
    }
    if let Some(exclude_patterns) = request.exclude_patterns {
        profile.exclude_patterns = exclude_patterns;
    }

    profile_repo::update_profile(&conn, &profile)?;
    Ok(profile)
}

#[tauri::command]
pub async fn delete_profile(
    db: tauri::State<'_, Mutex<Connection>>,
    id: String,
) -> Result<(), AppError> {
    let conn = db.lock().map_err(|e| AppError::General(e.to_string()))?;
    profile_repo::delete_profile(&conn, &id)
}
