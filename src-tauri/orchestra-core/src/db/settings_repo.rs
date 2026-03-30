use rusqlite::{params, Connection};

use crate::error::AppError;

pub fn get_setting(conn: &Connection, key: &str) -> Result<Option<String>, AppError> {
    let mut stmt = conn.prepare("SELECT value FROM settings WHERE key = ?1")?;
    match stmt.query_row(params![key], |row| row.get(0)) {
        Ok(value) => Ok(Some(value)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(AppError::from(e)),
    }
}

pub fn set_setting(conn: &Connection, key: &str, value: &str) -> Result<(), AppError> {
    conn.execute(
        "INSERT INTO settings (key, value) VALUES (?1, ?2)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value",
        params![key, value],
    )?;
    Ok(())
}

pub fn get_all_settings(conn: &Connection) -> Result<Vec<(String, String)>, AppError> {
    let mut stmt = conn.prepare("SELECT key, value FROM settings ORDER BY key")?;
    let settings = stmt
        .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(settings)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::schema;

    fn setup_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        schema::run_migrations(&conn).unwrap();
        conn
    }

    #[test]
    fn test_get_set_round_trip() {
        let conn = setup_db();
        set_setting(&conn, "library_root", "/music").unwrap();
        let value = get_setting(&conn, "library_root").unwrap();
        assert_eq!(value, Some("/music".to_string()));
    }

    #[test]
    fn test_get_missing_key_returns_none() {
        let conn = setup_db();
        let value = get_setting(&conn, "nonexistent_key").unwrap();
        assert_eq!(value, None);
    }

    #[test]
    fn test_set_upsert_overwrites_existing() {
        let conn = setup_db();
        set_setting(&conn, "library_root", "/old_path").unwrap();
        set_setting(&conn, "library_root", "/new_path").unwrap();
        let value = get_setting(&conn, "library_root").unwrap();
        assert_eq!(value, Some("/new_path".to_string()));
    }

    #[test]
    fn test_get_all_settings() {
        let conn = setup_db();
        set_setting(&conn, "library_root", "/music").unwrap();
        set_setting(&conn, "theme", "dark").unwrap();
        set_setting(&conn, "volume", "80").unwrap();
        let all = get_all_settings(&conn).unwrap();
        assert_eq!(all.len(), 3);
        assert_eq!(all[0], ("library_root".to_string(), "/music".to_string()));
        assert_eq!(all[1], ("theme".to_string(), "dark".to_string()));
        assert_eq!(all[2], ("volume".to_string(), "80".to_string()));
    }

    #[test]
    fn test_get_all_settings_empty() {
        let conn = setup_db();
        let all = get_all_settings(&conn).unwrap();
        assert!(all.is_empty());
    }

    #[test]
    fn test_db_error_propagates_as_err() {
        // Open a connection without running migrations so the settings table
        // does not exist — any query against it should produce a real DB error,
        // which get_setting must propagate as Err, not absorb as Ok(None).
        let conn = Connection::open_in_memory().unwrap();
        let result = get_setting(&conn, "library_root");
        assert!(
            result.is_err(),
            "expected Err when settings table is absent, got Ok({:?})",
            result
        );
    }
}
