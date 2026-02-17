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
