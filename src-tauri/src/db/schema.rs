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

        CREATE TABLE IF NOT EXISTS device_album_selections (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            device_id TEXT NOT NULL REFERENCES devices(id) ON DELETE CASCADE,
            artist_name TEXT NOT NULL,
            album_name TEXT NOT NULL,
            UNIQUE(device_id, artist_name, album_name)
        );
        CREATE INDEX IF NOT EXISTS idx_device_album_device ON device_album_selections(device_id);

        CREATE TABLE IF NOT EXISTS playlists (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            created_at INTEGER NOT NULL,
            updated_at INTEGER NOT NULL
        );

        CREATE TABLE IF NOT EXISTS playlist_tracks (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            playlist_id TEXT NOT NULL REFERENCES playlists(id) ON DELETE CASCADE,
            track_id INTEGER NOT NULL REFERENCES tracks(id) ON DELETE CASCADE,
            position INTEGER NOT NULL,
            UNIQUE(playlist_id, track_id)
        );
        CREATE INDEX IF NOT EXISTS idx_playlist_tracks_playlist ON playlist_tracks(playlist_id);

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

    // Migration: add has_album_art column if it doesn't exist
    let has_column: bool = conn
        .prepare("SELECT COUNT(*) FROM pragma_table_info('tracks') WHERE name='has_album_art'")?
        .query_row([], |row| row.get::<_, i64>(0))
        .map(|count| count > 0)?;

    if !has_column {
        conn.execute_batch(
            "ALTER TABLE tracks ADD COLUMN has_album_art INTEGER NOT NULL DEFAULT 0;",
        )?;
    }

    let has_bitrate: bool = conn
        .prepare("SELECT COUNT(*) FROM pragma_table_info('tracks') WHERE name='bitrate'")?
        .query_row([], |row| row.get::<_, i64>(0))
        .map(|count| count > 0)?;

    if !has_bitrate {
        conn.execute_batch(
            "ALTER TABLE tracks ADD COLUMN bitrate INTEGER;",
        )?;
    }

    let has_scanned_at: bool = conn
        .prepare("SELECT COUNT(*) FROM pragma_table_info('tracks') WHERE name='scanned_at'")?
        .query_row([], |row| row.get::<_, i64>(0))
        .map(|count| count > 0)?;

    if !has_scanned_at {
        conn.execute_batch(
            "ALTER TABLE tracks ADD COLUMN scanned_at INTEGER NOT NULL DEFAULT 0;",
        )?;
    }

    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS favorites (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            entity_type TEXT NOT NULL,
            entity_id TEXT NOT NULL,
            created_at INTEGER NOT NULL,
            UNIQUE(entity_type, entity_id)
        );
        CREATE INDEX IF NOT EXISTS idx_favorites_type ON favorites(entity_type);

        CREATE TABLE IF NOT EXISTS play_history (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            track_id INTEGER NOT NULL REFERENCES tracks(id) ON DELETE CASCADE,
            played_at INTEGER NOT NULL
        );
        CREATE INDEX IF NOT EXISTS idx_play_history_track ON play_history(track_id);
        CREATE INDEX IF NOT EXISTS idx_play_history_played_at ON play_history(played_at DESC);
        ",
    )?;

    Ok(())
}
