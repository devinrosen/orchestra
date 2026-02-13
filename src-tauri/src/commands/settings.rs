use std::sync::Mutex;
use rusqlite::{params, Connection};

use crate::error::AppError;

#[tauri::command]
pub async fn get_setting(
    db: tauri::State<'_, Mutex<Connection>>,
    key: String,
) -> Result<Option<String>, AppError> {
    let conn = db.lock().map_err(|e| AppError::General(e.to_string()))?;
    let mut stmt = conn.prepare("SELECT value FROM settings WHERE key = ?1")?;
    let result = stmt
        .query_row(params![key], |row| row.get(0))
        .ok();
    Ok(result)
}

#[tauri::command]
pub async fn set_setting(
    db: tauri::State<'_, Mutex<Connection>>,
    key: String,
    value: String,
) -> Result<(), AppError> {
    let conn = db.lock().map_err(|e| AppError::General(e.to_string()))?;
    conn.execute(
        "INSERT INTO settings (key, value) VALUES (?1, ?2)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value",
        params![key, value],
    )?;
    Ok(())
}

#[tauri::command]
pub async fn get_all_settings(
    db: tauri::State<'_, Mutex<Connection>>,
) -> Result<Vec<(String, String)>, AppError> {
    let conn = db.lock().map_err(|e| AppError::General(e.to_string()))?;
    let mut stmt = conn.prepare("SELECT key, value FROM settings ORDER BY key")?;
    let settings = stmt
        .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(settings)
}
