use std::path::Path;
use std::sync::Mutex;
use rusqlite::Connection;

use crate::db::library_root_repo;
use crate::error::AppError;
use crate::models::library_root::LibraryRoot;

#[tauri::command]
pub async fn add_library_root(
    db: tauri::State<'_, Mutex<Connection>>,
    path: String,
    label: Option<String>,
) -> Result<(), AppError> {
    if !Path::new(&path).exists() {
        return Err(AppError::PathNotAccessible(path));
    }

    let conn = db.lock().map_err(|e| AppError::General(e.to_string()))?;
    let added_at = chrono::Utc::now().timestamp();
    library_root_repo::add_library_root(&conn, &path, label.as_deref(), added_at)
}

#[tauri::command]
pub async fn remove_library_root(
    db: tauri::State<'_, Mutex<Connection>>,
    path: String,
) -> Result<(), AppError> {
    let conn = db.lock().map_err(|e| AppError::General(e.to_string()))?;
    library_root_repo::remove_library_root(&conn, &path)
}

#[tauri::command]
pub async fn list_library_roots(
    db: tauri::State<'_, Mutex<Connection>>,
) -> Result<Vec<LibraryRoot>, AppError> {
    let conn = db.lock().map_err(|e| AppError::General(e.to_string()))?;
    library_root_repo::list_library_roots(&conn)
}
