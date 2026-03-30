use rusqlite::{params, Connection};

use crate::error::AppError;
use crate::models::sync_profile::{SyncMode, SyncProfile};

pub fn create_profile(conn: &Connection, profile: &SyncProfile) -> Result<(), AppError> {
    let exclude_json = serde_json::to_string(&profile.exclude_patterns)
        .map_err(|e| AppError::General(e.to_string()))?;
    let mode_str = match profile.sync_mode {
        SyncMode::OneWay => "one_way",
        SyncMode::TwoWay => "two_way",
    };

    conn.execute(
        "INSERT INTO sync_profiles (id, name, source_path, target_path, sync_mode, exclude_patterns, created_at, last_synced_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![
            profile.id,
            profile.name,
            profile.source_path,
            profile.target_path,
            mode_str,
            exclude_json,
            profile.created_at,
            profile.last_synced_at,
        ],
    )?;
    Ok(())
}

pub fn get_profile(conn: &Connection, id: &str) -> Result<SyncProfile, AppError> {
    let mut stmt = conn.prepare(
        "SELECT id, name, source_path, target_path, sync_mode, exclude_patterns, created_at, last_synced_at
         FROM sync_profiles WHERE id = ?1",
    )?;

    stmt.query_row(params![id], |row| {
        let mode_str: String = row.get(4)?;
        let exclude_json: String = row.get(5)?;
        Ok(SyncProfile {
            id: row.get(0)?,
            name: row.get(1)?,
            source_path: row.get(2)?,
            target_path: row.get(3)?,
            sync_mode: if mode_str == "two_way" {
                SyncMode::TwoWay
            } else {
                SyncMode::OneWay
            },
            exclude_patterns: serde_json::from_str(&exclude_json).unwrap_or_default(),
            created_at: row.get(6)?,
            last_synced_at: row.get(7)?,
        })
    })
    .map_err(|e| match e {
        rusqlite::Error::QueryReturnedNoRows => AppError::ProfileNotFound(id.to_string()),
        other => AppError::Database(other),
    })
}

pub fn list_profiles(conn: &Connection) -> Result<Vec<SyncProfile>, AppError> {
    let mut stmt = conn.prepare(
        "SELECT id, name, source_path, target_path, sync_mode, exclude_patterns, created_at, last_synced_at
         FROM sync_profiles ORDER BY created_at DESC",
    )?;

    let profiles = stmt
        .query_map([], |row| {
            let mode_str: String = row.get(4)?;
            let exclude_json: String = row.get(5)?;
            Ok(SyncProfile {
                id: row.get(0)?,
                name: row.get(1)?,
                source_path: row.get(2)?,
                target_path: row.get(3)?,
                sync_mode: if mode_str == "two_way" {
                    SyncMode::TwoWay
                } else {
                    SyncMode::OneWay
                },
                exclude_patterns: serde_json::from_str(&exclude_json).unwrap_or_default(),
                created_at: row.get(6)?,
                last_synced_at: row.get(7)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

    Ok(profiles)
}

pub fn update_profile(conn: &Connection, profile: &SyncProfile) -> Result<(), AppError> {
    let exclude_json = serde_json::to_string(&profile.exclude_patterns)
        .map_err(|e| AppError::General(e.to_string()))?;
    let mode_str = match profile.sync_mode {
        SyncMode::OneWay => "one_way",
        SyncMode::TwoWay => "two_way",
    };

    let rows = conn.execute(
        "UPDATE sync_profiles SET name=?2, source_path=?3, target_path=?4, sync_mode=?5,
         exclude_patterns=?6, last_synced_at=?7 WHERE id=?1",
        params![
            profile.id,
            profile.name,
            profile.source_path,
            profile.target_path,
            mode_str,
            exclude_json,
            profile.last_synced_at,
        ],
    )?;

    if rows == 0 {
        return Err(AppError::ProfileNotFound(profile.id.clone()));
    }
    Ok(())
}

pub fn delete_profile(conn: &Connection, id: &str) -> Result<(), AppError> {
    let rows = conn.execute("DELETE FROM sync_profiles WHERE id = ?1", params![id])?;
    if rows == 0 {
        return Err(AppError::ProfileNotFound(id.to_string()));
    }
    Ok(())
}

pub fn update_last_synced(conn: &Connection, id: &str, timestamp: i64) -> Result<(), AppError> {
    conn.execute(
        "UPDATE sync_profiles SET last_synced_at = ?2 WHERE id = ?1",
        params![id, timestamp],
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::schema;
    use crate::models::sync_profile::{SyncMode, SyncProfile};

    fn setup_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        schema::run_migrations(&conn).unwrap();
        conn
    }

    fn make_profile(id: &str) -> SyncProfile {
        SyncProfile {
            id: id.to_string(),
            name: format!("Profile {}", id),
            source_path: "/source".to_string(),
            target_path: "/target".to_string(),
            sync_mode: SyncMode::OneWay,
            exclude_patterns: vec![],
            created_at: 1700000000,
            last_synced_at: None,
        }
    }

    #[test]
    fn test_create_and_get_profile() {
        let conn = setup_db();
        let profile = make_profile("p1");
        create_profile(&conn, &profile).unwrap();

        let fetched = get_profile(&conn, "p1").unwrap();
        assert_eq!(fetched.id, "p1");
        assert_eq!(fetched.name, "Profile p1");
        assert_eq!(fetched.source_path, "/source");
        assert_eq!(fetched.target_path, "/target");
        assert_eq!(fetched.created_at, 1700000000);
        assert_eq!(fetched.last_synced_at, None);
    }

    #[test]
    fn test_list_profiles() {
        let conn = setup_db();
        create_profile(&conn, &make_profile("p1")).unwrap();
        create_profile(&conn, &make_profile("p2")).unwrap();
        create_profile(&conn, &make_profile("p3")).unwrap();

        let profiles = list_profiles(&conn).unwrap();
        assert_eq!(profiles.len(), 3);
    }

    #[test]
    fn test_update_profile() {
        let conn = setup_db();
        let mut profile = make_profile("p1");
        create_profile(&conn, &profile).unwrap();

        profile.name = "Updated Name".to_string();
        profile.source_path = "/new_source".to_string();
        profile.target_path = "/new_target".to_string();
        profile.sync_mode = SyncMode::TwoWay;
        profile.exclude_patterns = vec!["*.tmp".to_string()];
        update_profile(&conn, &profile).unwrap();

        let fetched = get_profile(&conn, "p1").unwrap();
        assert_eq!(fetched.name, "Updated Name");
        assert_eq!(fetched.source_path, "/new_source");
        assert_eq!(fetched.target_path, "/new_target");
        assert!(matches!(fetched.sync_mode, SyncMode::TwoWay));
        assert_eq!(fetched.exclude_patterns, vec!["*.tmp"]);
    }

    #[test]
    fn test_delete_profile() {
        let conn = setup_db();
        create_profile(&conn, &make_profile("p1")).unwrap();
        delete_profile(&conn, "p1").unwrap();

        let result = get_profile(&conn, "p1");
        assert!(matches!(result, Err(AppError::ProfileNotFound(_))));
    }

    #[test]
    fn test_delete_nonexistent_returns_error() {
        let conn = setup_db();
        let result = delete_profile(&conn, "nonexistent");
        assert!(matches!(result, Err(AppError::ProfileNotFound(_))));
    }

    #[test]
    fn test_update_last_synced() {
        let conn = setup_db();
        create_profile(&conn, &make_profile("p1")).unwrap();

        update_last_synced(&conn, "p1", 1750000000).unwrap();

        let fetched = get_profile(&conn, "p1").unwrap();
        assert_eq!(fetched.last_synced_at, Some(1750000000));
    }

    #[test]
    fn test_profile_sync_mode_roundtrip() {
        let conn = setup_db();

        let mut p1 = make_profile("p1");
        p1.sync_mode = SyncMode::OneWay;
        create_profile(&conn, &p1).unwrap();
        let fetched1 = get_profile(&conn, "p1").unwrap();
        assert!(matches!(fetched1.sync_mode, SyncMode::OneWay));

        let mut p2 = make_profile("p2");
        p2.sync_mode = SyncMode::TwoWay;
        create_profile(&conn, &p2).unwrap();
        let fetched2 = get_profile(&conn, "p2").unwrap();
        assert!(matches!(fetched2.sync_mode, SyncMode::TwoWay));
    }

    #[test]
    fn test_profile_exclude_patterns_roundtrip() {
        let conn = setup_db();
        let mut profile = make_profile("p1");
        profile.exclude_patterns = vec![
            "*.tmp".to_string(),
            "backup/**".to_string(),
            ".DS_Store".to_string(),
        ];
        create_profile(&conn, &profile).unwrap();

        let fetched = get_profile(&conn, "p1").unwrap();
        assert_eq!(
            fetched.exclude_patterns,
            vec!["*.tmp", "backup/**", ".DS_Store"]
        );
    }

    #[test]
    fn test_get_nonexistent_profile_returns_error() {
        let conn = setup_db();
        let result = get_profile(&conn, "nonexistent");
        assert!(matches!(result, Err(AppError::ProfileNotFound(_))));
    }
}
