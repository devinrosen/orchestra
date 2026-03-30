use rusqlite::{params, Connection};

use crate::db::library_repo::track_from_row;
use crate::error::AppError;
use crate::models::track::Track;

/// Returns all tracks for the given library root, ordered by id.
pub fn get_all_tracks_for_organize(
    conn: &Connection,
    library_root: &str,
) -> Result<Vec<Track>, AppError> {
    let mut stmt = conn.prepare(
        "SELECT id, file_path, relative_path, library_root, title, artist, album_artist, album,
         track_number, disc_number, year, genre, duration_secs, format, file_size, modified_at,
         hash, has_album_art, bitrate, scanned_at
         FROM tracks WHERE library_root = ?1 ORDER BY id",
    )?;
    let tracks = stmt
        .query_map(params![library_root], track_from_row)?
        .collect::<rusqlite::Result<Vec<Track>>>()?;
    Ok(tracks)
}

/// Updates file_path and relative_path for a single track after a successful file move.
pub fn update_track_paths(
    conn: &Connection,
    track_id: i64,
    new_file_path: &str,
    new_relative_path: &str,
) -> Result<(), AppError> {
    bulk_update_track_paths(
        conn,
        &[(track_id, new_file_path.to_string(), new_relative_path.to_string())],
    )
}

/// Batch-updates file_path and relative_path for multiple tracks in a single transaction.
/// Each element of `updates` is `(track_id, new_file_path, new_relative_path)`.
pub fn bulk_update_track_paths(
    conn: &Connection,
    updates: &[(i64, String, String)],
) -> Result<(), AppError> {
    let tx = conn.unchecked_transaction()?;
    for (track_id, new_file_path, new_relative_path) in updates {
        tx.execute(
            "UPDATE tracks SET file_path = ?1, relative_path = ?2 WHERE id = ?3",
            params![new_file_path, new_relative_path, track_id],
        )?;
    }
    tx.commit()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    fn setup_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "CREATE TABLE tracks (
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
                format TEXT NOT NULL DEFAULT '',
                file_size INTEGER NOT NULL DEFAULT 0,
                modified_at INTEGER NOT NULL DEFAULT 0,
                hash TEXT,
                has_album_art INTEGER NOT NULL DEFAULT 0,
                bitrate INTEGER,
                scanned_at INTEGER NOT NULL DEFAULT 0
            );
            INSERT INTO tracks (file_path, relative_path, library_root, title, artist, album_artist, album,
                track_number, disc_number, year, genre, duration_secs, format, file_size, modified_at,
                hash, has_album_art, bitrate, scanned_at)
            VALUES
                ('/music/Pink Floyd/DSOTM/01 Speak to Me.flac',
                 'Pink Floyd/DSOTM/01 Speak to Me.flac', '/music',
                 'Speak to Me', 'Pink Floyd', 'Pink Floyd', 'The Dark Side of the Moon',
                 1, 1, 1973, 'Progressive Rock', 68.0, 'FLAC', 35000000, 1700000000,
                 NULL, 1, 900, 1700000000),
                ('/music/Radiohead/OK Computer/01 Airbag.flac',
                 'Radiohead/OK Computer/01 Airbag.flac', '/music',
                 'Airbag', 'Radiohead', 'Radiohead', 'OK Computer',
                 1, 1, 1997, 'Alternative Rock', 284.0, 'FLAC', 35000000, 1700000000,
                 NULL, 1, 900, 1700000000);",
        )
        .unwrap();
        conn
    }

    #[test]
    fn test_get_all_tracks_for_organize() {
        let conn = setup_db();
        let tracks = get_all_tracks_for_organize(&conn, "/music").unwrap();
        assert_eq!(tracks.len(), 2);
        assert_eq!(tracks[0].title.as_deref(), Some("Speak to Me"));
        assert_eq!(tracks[1].title.as_deref(), Some("Airbag"));
    }

    #[test]
    fn test_get_all_tracks_filters_by_library_root() {
        let conn = setup_db();
        let tracks = get_all_tracks_for_organize(&conn, "/other").unwrap();
        assert!(tracks.is_empty());
    }

    #[test]
    fn test_update_track_paths() {
        let conn = setup_db();
        let tracks = get_all_tracks_for_organize(&conn, "/music").unwrap();
        let id = tracks[0].id.unwrap();

        update_track_paths(&conn, id, "/music/new/path.flac", "new/path.flac").unwrap();

        let updated = get_all_tracks_for_organize(&conn, "/music").unwrap();
        let t = updated.iter().find(|t| t.id == Some(id)).unwrap();
        assert_eq!(t.file_path, "/music/new/path.flac");
        assert_eq!(t.relative_path, "new/path.flac");
    }

    #[test]
    fn test_bulk_update_track_paths() {
        let conn = setup_db();
        let tracks = get_all_tracks_for_organize(&conn, "/music").unwrap();
        let id0 = tracks[0].id.unwrap();
        let id1 = tracks[1].id.unwrap();

        let updates = vec![
            (id0, "/music/a/b.flac".to_string(), "a/b.flac".to_string()),
            (id1, "/music/c/d.flac".to_string(), "c/d.flac".to_string()),
        ];
        bulk_update_track_paths(&conn, &updates).unwrap();

        let updated = get_all_tracks_for_organize(&conn, "/music").unwrap();
        let t0 = updated.iter().find(|t| t.id == Some(id0)).unwrap();
        let t1 = updated.iter().find(|t| t.id == Some(id1)).unwrap();
        assert_eq!(t0.relative_path, "a/b.flac");
        assert_eq!(t1.relative_path, "c/d.flac");
    }
}
