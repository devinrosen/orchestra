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
