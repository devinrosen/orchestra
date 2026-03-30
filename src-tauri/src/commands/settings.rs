use rusqlite::{params, Connection};
use std::sync::Mutex;

use orchestra_core::error::AppError;

#[tauri::command]
pub async fn get_setting(
    db: tauri::State<'_, Mutex<Connection>>,
    key: String,
) -> Result<Option<String>, AppError> {
    let conn = db.lock().map_err(|e| AppError::General(e.to_string()))?;
    let mut stmt = conn.prepare("SELECT value FROM settings WHERE key = ?1")?;
    let result = stmt.query_row(params![key], |row| row.get(0)).ok();
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

#[cfg(test)]
mod tests {
    use orchestra_core::db::schema;
    use rusqlite::{params, Connection};

    fn setup_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        schema::run_migrations(&conn).unwrap();
        conn
    }

    fn db_get(conn: &Connection, key: &str) -> Option<String> {
        conn.prepare("SELECT value FROM settings WHERE key = ?1")
            .ok()?
            .query_row(params![key], |row| row.get(0))
            .ok()
    }

    fn db_set(conn: &Connection, key: &str, value: &str) {
        conn.execute(
            "INSERT INTO settings (key, value) VALUES (?1, ?2)
             ON CONFLICT(key) DO UPDATE SET value = excluded.value",
            params![key, value],
        )
        .unwrap();
    }

    fn db_get_all(conn: &Connection) -> Vec<(String, String)> {
        let mut stmt = conn
            .prepare("SELECT key, value FROM settings ORDER BY key")
            .unwrap();
        stmt.query_map([], |row| Ok((row.get(0)?, row.get(1)?)))
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap()
    }

    #[test]
    fn test_get_set_round_trip() {
        let conn = setup_db();
        db_set(&conn, "library_root", "/music");

        let value = db_get(&conn, "library_root");
        assert_eq!(value, Some("/music".to_string()));
    }

    #[test]
    fn test_get_missing_key_returns_none() {
        let conn = setup_db();
        let value = db_get(&conn, "nonexistent_key");
        assert_eq!(value, None);
    }

    #[test]
    fn test_set_upsert_overwrites_existing() {
        let conn = setup_db();
        db_set(&conn, "library_root", "/old_path");
        db_set(&conn, "library_root", "/new_path");

        let value = db_get(&conn, "library_root");
        assert_eq!(value, Some("/new_path".to_string()));
    }

    #[test]
    fn test_get_all_settings() {
        let conn = setup_db();
        db_set(&conn, "library_root", "/music");
        db_set(&conn, "theme", "dark");
        db_set(&conn, "volume", "80");

        let all = db_get_all(&conn);
        assert_eq!(all.len(), 3);
        // Ordered by key: library_root, theme, volume
        assert_eq!(all[0], ("library_root".to_string(), "/music".to_string()));
        assert_eq!(all[1], ("theme".to_string(), "dark".to_string()));
        assert_eq!(all[2], ("volume".to_string(), "80".to_string()));
    }

    #[test]
    fn test_get_all_settings_empty() {
        let conn = setup_db();
        let all = db_get_all(&conn);
        assert!(all.is_empty());
    }
}
