use rusqlite::{params, Connection};

use crate::error::AppError;
use crate::models::device::Device;

pub fn save_device(conn: &Connection, device: &Device) -> Result<(), AppError> {
    conn.execute(
        "INSERT INTO devices (id, name, volume_uuid, volume_name, mount_path, capacity_bytes, music_folder, created_at, last_synced_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
         ON CONFLICT(volume_uuid) DO UPDATE SET
           name=excluded.name, mount_path=excluded.mount_path,
           capacity_bytes=excluded.capacity_bytes, music_folder=excluded.music_folder",
        params![
            device.id,
            device.name,
            device.volume_uuid,
            device.volume_name,
            device.mount_path,
            device.capacity_bytes.map(|b| b as i64),
            device.music_folder,
            device.created_at,
            device.last_synced_at,
        ],
    )?;
    Ok(())
}

pub fn get_device(conn: &Connection, id: &str) -> Result<Device, AppError> {
    conn.query_row(
        "SELECT id, name, volume_uuid, volume_name, mount_path, capacity_bytes, music_folder, created_at, last_synced_at
         FROM devices WHERE id = ?1",
        params![id],
        |row| {
            Ok(Device {
                id: row.get(0)?,
                name: row.get(1)?,
                volume_uuid: row.get(2)?,
                volume_name: row.get(3)?,
                mount_path: row.get(4)?,
                capacity_bytes: row.get::<_, Option<i64>>(5)?.map(|v| v as u64),
                music_folder: row.get(6)?,
                created_at: row.get(7)?,
                last_synced_at: row.get(8)?,
            })
        },
    )
    .map_err(|e| match e {
        rusqlite::Error::QueryReturnedNoRows => AppError::DeviceNotFound(id.to_string()),
        _ => AppError::Database(e),
    })
}

pub fn get_device_by_uuid(conn: &Connection, volume_uuid: &str) -> Result<Option<Device>, AppError> {
    let mut stmt = conn.prepare(
        "SELECT id, name, volume_uuid, volume_name, mount_path, capacity_bytes, music_folder, created_at, last_synced_at
         FROM devices WHERE volume_uuid = ?1",
    )?;
    let mut rows = stmt.query_map(params![volume_uuid], |row| {
        Ok(Device {
            id: row.get(0)?,
            name: row.get(1)?,
            volume_uuid: row.get(2)?,
            volume_name: row.get(3)?,
            mount_path: row.get(4)?,
            capacity_bytes: row.get::<_, Option<i64>>(5)?.map(|v| v as u64),
            music_folder: row.get(6)?,
            created_at: row.get(7)?,
            last_synced_at: row.get(8)?,
        })
    })?;
    match rows.next() {
        Some(Ok(device)) => Ok(Some(device)),
        Some(Err(e)) => Err(AppError::Database(e)),
        None => Ok(None),
    }
}

pub fn list_devices(conn: &Connection) -> Result<Vec<Device>, AppError> {
    let mut stmt = conn.prepare(
        "SELECT id, name, volume_uuid, volume_name, mount_path, capacity_bytes, music_folder, created_at, last_synced_at
         FROM devices ORDER BY name COLLATE NOCASE",
    )?;
    let devices = stmt
        .query_map([], |row| {
            Ok(Device {
                id: row.get(0)?,
                name: row.get(1)?,
                volume_uuid: row.get(2)?,
                volume_name: row.get(3)?,
                mount_path: row.get(4)?,
                capacity_bytes: row.get::<_, Option<i64>>(5)?.map(|v| v as u64),
                music_folder: row.get(6)?,
                created_at: row.get(7)?,
                last_synced_at: row.get(8)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(devices)
}

pub fn delete_device(conn: &Connection, id: &str) -> Result<(), AppError> {
    let affected = conn.execute("DELETE FROM devices WHERE id = ?1", params![id])?;
    if affected == 0 {
        return Err(AppError::DeviceNotFound(id.to_string()));
    }
    Ok(())
}

pub fn update_last_synced(conn: &Connection, id: &str, timestamp: i64) -> Result<(), AppError> {
    conn.execute(
        "UPDATE devices SET last_synced_at = ?2 WHERE id = ?1",
        params![id, timestamp],
    )?;
    Ok(())
}

pub fn update_mount_path(conn: &Connection, id: &str, mount_path: &str) -> Result<(), AppError> {
    conn.execute(
        "UPDATE devices SET mount_path = ?2 WHERE id = ?1",
        params![id, mount_path],
    )?;
    Ok(())
}

// --- File hash cache ---

#[derive(Debug, Clone)]
pub struct CachedFileHash {
    pub relative_path: String,
    pub hash: String,
    pub file_size: u64,
    pub modified_at: i64,
}

pub fn get_file_cache(
    conn: &Connection,
    device_id: &str,
) -> Result<std::collections::HashMap<String, CachedFileHash>, AppError> {
    let mut stmt = conn.prepare(
        "SELECT relative_path, hash, file_size, modified_at
         FROM device_file_cache WHERE device_id = ?1",
    )?;
    let entries = stmt
        .query_map(params![device_id], |row| {
            Ok(CachedFileHash {
                relative_path: row.get(0)?,
                hash: row.get(1)?,
                file_size: row.get::<_, i64>(2)? as u64,
                modified_at: row.get(3)?,
            })
        })?
        .filter_map(|r| r.ok())
        .map(|c| (c.relative_path.clone(), c))
        .collect();
    Ok(entries)
}

pub fn save_file_cache(
    conn: &Connection,
    device_id: &str,
    entries: &[CachedFileHash],
) -> Result<(), AppError> {
    conn.execute(
        "DELETE FROM device_file_cache WHERE device_id = ?1",
        params![device_id],
    )?;
    let mut stmt = conn.prepare(
        "INSERT INTO device_file_cache (device_id, relative_path, hash, file_size, modified_at)
         VALUES (?1, ?2, ?3, ?4, ?5)",
    )?;
    for entry in entries {
        stmt.execute(params![
            device_id,
            entry.relative_path,
            entry.hash,
            entry.file_size as i64,
            entry.modified_at,
        ])?;
    }
    Ok(())
}

// --- Artist selections ---

pub fn get_selected_artists(conn: &Connection, device_id: &str) -> Result<Vec<String>, AppError> {
    let mut stmt = conn.prepare(
        "SELECT artist_name FROM device_artist_selections WHERE device_id = ?1 ORDER BY artist_name COLLATE NOCASE",
    )?;
    let artists = stmt
        .query_map(params![device_id], |row| row.get::<_, String>(0))?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(artists)
}

pub fn set_selected_artists(
    conn: &Connection,
    device_id: &str,
    artists: &[String],
) -> Result<(), AppError> {
    conn.execute(
        "DELETE FROM device_artist_selections WHERE device_id = ?1",
        params![device_id],
    )?;
    let mut stmt = conn.prepare(
        "INSERT INTO device_artist_selections (device_id, artist_name) VALUES (?1, ?2)",
    )?;
    for artist in artists {
        stmt.execute(params![device_id, artist])?;
    }
    Ok(())
}
