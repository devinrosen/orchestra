use rusqlite::{params, Connection};
use std::collections::HashMap;

use crate::error::AppError;

#[derive(Debug, Clone)]
pub struct FileBaseline {
    pub relative_path: String,
    pub source_hash: Option<String>,
    pub target_hash: Option<String>,
    pub source_modified: Option<i64>,
    pub target_modified: Option<i64>,
    pub source_size: Option<u64>,
    pub target_size: Option<u64>,
}

pub fn get_baselines(
    conn: &Connection,
    profile_id: &str,
) -> Result<HashMap<String, FileBaseline>, AppError> {
    let mut stmt = conn.prepare(
        "SELECT relative_path, source_hash, target_hash, source_modified, target_modified, source_size, target_size
         FROM sync_state WHERE profile_id = ?1",
    )?;

    let mut map = HashMap::new();
    let rows = stmt.query_map(params![profile_id], |row| {
        Ok(FileBaseline {
            relative_path: row.get(0)?,
            source_hash: row.get(1)?,
            target_hash: row.get(2)?,
            source_modified: row.get(3)?,
            target_modified: row.get(4)?,
            source_size: row.get::<_, Option<i64>>(5)?.map(|v| v as u64),
            target_size: row.get::<_, Option<i64>>(6)?.map(|v| v as u64),
        })
    })?;

    for row in rows {
        let baseline = row?;
        map.insert(baseline.relative_path.clone(), baseline);
    }

    Ok(map)
}

pub fn save_baselines(
    conn: &Connection,
    profile_id: &str,
    baselines: &[FileBaseline],
) -> Result<(), AppError> {
    conn.execute(
        "DELETE FROM sync_state WHERE profile_id = ?1",
        params![profile_id],
    )?;

    let now = chrono::Utc::now().timestamp();
    let mut stmt = conn.prepare(
        "INSERT INTO sync_state (profile_id, relative_path, source_hash, target_hash,
         source_modified, target_modified, source_size, target_size, snapshot_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
    )?;

    for b in baselines {
        stmt.execute(params![
            profile_id,
            b.relative_path,
            b.source_hash,
            b.target_hash,
            b.source_modified,
            b.target_modified,
            b.source_size.map(|v| v as i64),
            b.target_size.map(|v| v as i64),
            now,
        ])?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::{profile_repo, schema};
    use crate::models::sync_profile::{SyncMode, SyncProfile};

    fn setup_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        schema::run_migrations(&conn).unwrap();
        conn
    }

    fn insert_test_profile(conn: &Connection, id: &str) {
        let profile = SyncProfile {
            id: id.to_string(),
            name: format!("Profile {}", id),
            source_path: "/source".to_string(),
            target_path: "/target".to_string(),
            sync_mode: SyncMode::OneWay,
            exclude_patterns: vec![],
            created_at: 1700000000,
            last_synced_at: None,
        };
        profile_repo::create_profile(conn, &profile).unwrap();
    }

    #[test]
    fn test_save_and_get_baselines() {
        let conn = setup_db();
        let profile_id = "test-profile-1";
        insert_test_profile(&conn, profile_id);

        let baselines = vec![
            FileBaseline {
                relative_path: "artist/album/track01.flac".to_string(),
                source_hash: Some("abc123".to_string()),
                target_hash: Some("abc123".to_string()),
                source_modified: Some(1700000001),
                target_modified: Some(1700000001),
                source_size: Some(5_000_000),
                target_size: Some(5_000_000),
            },
            FileBaseline {
                relative_path: "artist/album/track02.flac".to_string(),
                source_hash: Some("def456".to_string()),
                target_hash: None,
                source_modified: Some(1700000002),
                target_modified: None,
                source_size: Some(6_000_000),
                target_size: None,
            },
        ];

        save_baselines(&conn, profile_id, &baselines).unwrap();
        let retrieved = get_baselines(&conn, profile_id).unwrap();

        assert_eq!(retrieved.len(), 2);

        let b1 = retrieved.get("artist/album/track01.flac").unwrap();
        assert_eq!(b1.source_hash.as_deref(), Some("abc123"));
        assert_eq!(b1.target_hash.as_deref(), Some("abc123"));
        assert_eq!(b1.source_modified, Some(1700000001));
        assert_eq!(b1.source_size, Some(5_000_000));

        let b2 = retrieved.get("artist/album/track02.flac").unwrap();
        assert_eq!(b2.source_hash.as_deref(), Some("def456"));
        assert_eq!(b2.target_hash, None);
    }

    #[test]
    fn test_save_baselines_replaces_existing() {
        let conn = setup_db();
        let profile_id = "test-profile-2";
        insert_test_profile(&conn, profile_id);

        let first = vec![FileBaseline {
            relative_path: "track.flac".to_string(),
            source_hash: Some("hash1".to_string()),
            target_hash: None,
            source_modified: Some(100),
            target_modified: None,
            source_size: Some(1000),
            target_size: None,
        }];
        save_baselines(&conn, profile_id, &first).unwrap();

        let second = vec![FileBaseline {
            relative_path: "new_track.flac".to_string(),
            source_hash: Some("hash2".to_string()),
            target_hash: Some("hash2".to_string()),
            source_modified: Some(200),
            target_modified: Some(200),
            source_size: Some(2000),
            target_size: Some(2000),
        }];
        save_baselines(&conn, profile_id, &second).unwrap();

        let retrieved = get_baselines(&conn, profile_id).unwrap();
        assert_eq!(retrieved.len(), 1);
        assert!(retrieved.contains_key("new_track.flac"));
        assert!(!retrieved.contains_key("track.flac"));
    }

    #[test]
    fn test_get_baselines_empty() {
        let conn = setup_db();
        let retrieved = get_baselines(&conn, "unknown-profile-id").unwrap();
        assert!(retrieved.is_empty());
    }

    #[test]
    fn test_baseline_with_none_optional_fields() {
        let conn = setup_db();
        let profile_id = "test-profile-3";
        insert_test_profile(&conn, profile_id);

        let baselines = vec![FileBaseline {
            relative_path: "sparse.flac".to_string(),
            source_hash: None,
            target_hash: None,
            source_modified: None,
            target_modified: None,
            source_size: None,
            target_size: None,
        }];
        save_baselines(&conn, profile_id, &baselines).unwrap();
        let retrieved = get_baselines(&conn, profile_id).unwrap();

        assert_eq!(retrieved.len(), 1);
        let b = retrieved.get("sparse.flac").unwrap();
        assert_eq!(b.source_hash, None);
        assert_eq!(b.target_hash, None);
        assert_eq!(b.source_modified, None);
        assert_eq!(b.target_modified, None);
        assert_eq!(b.source_size, None);
        assert_eq!(b.target_size, None);
    }
}
