use rusqlite::Connection;

use crate::error::AppError;

pub fn run_migrations(conn: &Connection) -> Result<(), AppError> {
    conn.execute_batch("PRAGMA journal_mode=WAL;")?;
    conn.execute_batch("PRAGMA foreign_keys=ON;")?;

    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS tracks (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            file_path TEXT NOT NULL UNIQUE,
            relative_path TEXT NOT NULL,
            library_root TEXT NOT NULL,
            title TEXT,
            artist TEXT,
            album_artist TEXT,
            album TEXT,
            track_number INTEGER,
            disc_number INTEGER,
            year INTEGER,
            genre TEXT,
            duration_secs REAL,
            format TEXT NOT NULL,
            file_size INTEGER NOT NULL,
            modified_at INTEGER NOT NULL,
            hash TEXT
        );

        CREATE INDEX IF NOT EXISTS idx_tracks_library_root ON tracks(library_root);
        CREATE INDEX IF NOT EXISTS idx_tracks_artist ON tracks(artist);
        CREATE INDEX IF NOT EXISTS idx_tracks_album ON tracks(album);
        CREATE INDEX IF NOT EXISTS idx_tracks_relative_path ON tracks(relative_path);

        CREATE TABLE IF NOT EXISTS sync_profiles (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            source_path TEXT NOT NULL,
            target_path TEXT NOT NULL,
            sync_mode TEXT NOT NULL DEFAULT 'one_way',
            exclude_patterns TEXT NOT NULL DEFAULT '[]',
            created_at INTEGER NOT NULL,
            last_synced_at INTEGER
        );

        CREATE TABLE IF NOT EXISTS sync_state (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            profile_id TEXT NOT NULL REFERENCES sync_profiles(id) ON DELETE CASCADE,
            relative_path TEXT NOT NULL,
            source_hash TEXT,
            target_hash TEXT,
            source_modified INTEGER,
            target_modified INTEGER,
            source_size INTEGER,
            target_size INTEGER,
            snapshot_at INTEGER NOT NULL,
            UNIQUE(profile_id, relative_path)
        );

        CREATE INDEX IF NOT EXISTS idx_sync_state_profile ON sync_state(profile_id);

        CREATE TABLE IF NOT EXISTS settings (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS devices (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            volume_uuid TEXT NOT NULL UNIQUE,
            volume_name TEXT NOT NULL,
            mount_path TEXT,
            capacity_bytes INTEGER,
            music_folder TEXT NOT NULL DEFAULT '',
            created_at INTEGER NOT NULL,
            last_synced_at INTEGER
        );

        CREATE TABLE IF NOT EXISTS device_artist_selections (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            device_id TEXT NOT NULL REFERENCES devices(id) ON DELETE CASCADE,
            artist_name TEXT NOT NULL,
            UNIQUE(device_id, artist_name)
        );
        CREATE INDEX IF NOT EXISTS idx_device_artist_device ON device_artist_selections(device_id);

        CREATE TABLE IF NOT EXISTS device_file_cache (
            device_id TEXT NOT NULL REFERENCES devices(id) ON DELETE CASCADE,
            relative_path TEXT NOT NULL,
            hash TEXT NOT NULL,
            file_size INTEGER NOT NULL,
            modified_at INTEGER NOT NULL,
            PRIMARY KEY (device_id, relative_path)
        );
        ",
    )?;

    Ok(())
}
