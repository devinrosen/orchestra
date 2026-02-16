use rusqlite::{params, Connection};

use crate::db::library_repo::track_from_row;
use crate::error::AppError;
use crate::models::favorite::Favorite;
use crate::models::track::Track;

pub fn add_favorite(
    conn: &Connection,
    entity_type: &str,
    entity_id: &str,
) -> Result<(), AppError> {
    let now = chrono::Utc::now().timestamp();
    conn.execute(
        "INSERT OR IGNORE INTO favorites (entity_type, entity_id, created_at) VALUES (?1, ?2, ?3)",
        params![entity_type, entity_id, now],
    )?;
    Ok(())
}

pub fn remove_favorite(
    conn: &Connection,
    entity_type: &str,
    entity_id: &str,
) -> Result<(), AppError> {
    conn.execute(
        "DELETE FROM favorites WHERE entity_type = ?1 AND entity_id = ?2",
        params![entity_type, entity_id],
    )?;
    Ok(())
}

pub fn is_favorite(
    conn: &Connection,
    entity_type: &str,
    entity_id: &str,
) -> Result<bool, AppError> {
    let count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM favorites WHERE entity_type = ?1 AND entity_id = ?2",
        params![entity_type, entity_id],
        |row| row.get(0),
    )?;
    Ok(count > 0)
}

pub fn list_favorites(conn: &Connection, entity_type: &str) -> Result<Vec<Favorite>, AppError> {
    let mut stmt = conn.prepare(
        "SELECT entity_type, entity_id, created_at FROM favorites WHERE entity_type = ?1 ORDER BY created_at DESC",
    )?;
    let favorites = stmt
        .query_map(params![entity_type], |row| {
            Ok(Favorite {
                entity_type: row.get(0)?,
                entity_id: row.get(1)?,
                created_at: row.get(2)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(favorites)
}

pub fn list_all_favorites(conn: &Connection) -> Result<Vec<Favorite>, AppError> {
    let mut stmt = conn.prepare(
        "SELECT entity_type, entity_id, created_at FROM favorites ORDER BY entity_type, created_at DESC",
    )?;
    let favorites = stmt
        .query_map([], |row| {
            Ok(Favorite {
                entity_type: row.get(0)?,
                entity_id: row.get(1)?,
                created_at: row.get(2)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(favorites)
}

pub fn get_favorite_tracks(conn: &Connection) -> Result<Vec<Track>, AppError> {
    let mut stmt = conn.prepare(
        "SELECT t.id, t.file_path, t.relative_path, t.library_root, t.title, t.artist,
                t.album_artist, t.album, t.track_number, t.disc_number, t.year, t.genre,
                t.duration_secs, t.format, t.file_size, t.modified_at, t.hash, t.has_album_art, t.bitrate, t.scanned_at
         FROM favorites f
         JOIN tracks t ON t.id = CAST(f.entity_id AS INTEGER)
         WHERE f.entity_type = 'track'
         ORDER BY f.created_at DESC",
    )?;
    let tracks = stmt
        .query_map([], track_from_row)?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(tracks)
}

pub fn toggle_favorite(
    conn: &Connection,
    entity_type: &str,
    entity_id: &str,
) -> Result<bool, AppError> {
    if is_favorite(conn, entity_type, entity_id)? {
        remove_favorite(conn, entity_type, entity_id)?;
        Ok(false)
    } else {
        add_favorite(conn, entity_type, entity_id)?;
        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::library_repo;
    use crate::db::schema;
    use crate::models::track::Track;

    fn setup_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        schema::run_migrations(&conn).unwrap();
        conn
    }

    fn insert_test_track(conn: &Connection, file_suffix: &str) -> i64 {
        let track = Track {
            id: None,
            file_path: format!("/music/artist/album/{}.flac", file_suffix),
            relative_path: format!("artist/album/{}.flac", file_suffix),
            library_root: "/music".to_string(),
            title: Some(format!("Track {}", file_suffix)),
            artist: Some("Artist".to_string()),
            album_artist: None,
            album: Some("Album".to_string()),
            track_number: Some(1),
            disc_number: Some(1),
            year: Some(2024),
            genre: Some("Rock".to_string()),
            duration_secs: Some(200.0),
            format: "flac".to_string(),
            file_size: 30_000_000,
            modified_at: 1700000000,
            hash: None,
            has_album_art: false,
            bitrate: None,
            scanned_at: 0,
        };
        library_repo::upsert_track(conn, &track).unwrap();
        conn.query_row(
            "SELECT id FROM tracks WHERE file_path = ?1",
            params![track.file_path],
            |row| row.get(0),
        )
        .unwrap()
    }

    #[test]
    fn test_favorites_table_created() {
        let conn = setup_db();
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM favorites", [], |row| row.get(0))
            .unwrap();
        assert_eq!(count, 0);
    }

    #[test]
    fn test_add_and_check_favorite() {
        let conn = setup_db();
        let track_id = insert_test_track(&conn, "t1");
        add_favorite(&conn, "track", &track_id.to_string()).unwrap();
        assert!(is_favorite(&conn, "track", &track_id.to_string()).unwrap());
    }

    #[test]
    fn test_remove_favorite() {
        let conn = setup_db();
        let track_id = insert_test_track(&conn, "t1");
        add_favorite(&conn, "track", &track_id.to_string()).unwrap();
        remove_favorite(&conn, "track", &track_id.to_string()).unwrap();
        assert!(!is_favorite(&conn, "track", &track_id.to_string()).unwrap());
    }

    #[test]
    fn test_add_duplicate_favorite_is_idempotent() {
        let conn = setup_db();
        add_favorite(&conn, "artist", "Pink Floyd").unwrap();
        add_favorite(&conn, "artist", "Pink Floyd").unwrap();
        let favs = list_favorites(&conn, "artist").unwrap();
        assert_eq!(favs.len(), 1);
    }

    #[test]
    fn test_list_favorites_by_type() {
        let conn = setup_db();
        let track_id = insert_test_track(&conn, "t1");
        add_favorite(&conn, "artist", "Pink Floyd").unwrap();
        add_favorite(&conn, "track", &track_id.to_string()).unwrap();

        let artists = list_favorites(&conn, "artist").unwrap();
        assert_eq!(artists.len(), 1);
        assert_eq!(artists[0].entity_id, "Pink Floyd");

        let tracks = list_favorites(&conn, "track").unwrap();
        assert_eq!(tracks.len(), 1);
    }

    #[test]
    fn test_list_all_favorites() {
        let conn = setup_db();
        add_favorite(&conn, "artist", "Pink Floyd").unwrap();
        add_favorite(&conn, "album", "Pink Floyd\0The Wall").unwrap();
        add_favorite(&conn, "track", "42").unwrap();

        let all = list_all_favorites(&conn).unwrap();
        assert_eq!(all.len(), 3);
    }

    #[test]
    fn test_get_favorite_tracks_returns_full_track_data() {
        let conn = setup_db();
        let track_id = insert_test_track(&conn, "t1");
        add_favorite(&conn, "track", &track_id.to_string()).unwrap();

        let tracks = get_favorite_tracks(&conn).unwrap();
        assert_eq!(tracks.len(), 1);
        assert_eq!(tracks[0].id, Some(track_id));
        assert_eq!(tracks[0].title, Some("Track t1".to_string()));
    }

    #[test]
    fn test_get_favorite_tracks_skips_deleted_tracks() {
        let conn = setup_db();
        let track_id = insert_test_track(&conn, "t1");
        add_favorite(&conn, "track", &track_id.to_string()).unwrap();

        conn.execute("DELETE FROM tracks WHERE id = ?1", params![track_id])
            .unwrap();

        let tracks = get_favorite_tracks(&conn).unwrap();
        assert!(tracks.is_empty());
    }

    #[test]
    fn test_toggle_favorite_add_then_remove() {
        let conn = setup_db();
        let result1 = toggle_favorite(&conn, "artist", "Radiohead").unwrap();
        assert!(result1); // added

        let result2 = toggle_favorite(&conn, "artist", "Radiohead").unwrap();
        assert!(!result2); // removed

        assert!(!is_favorite(&conn, "artist", "Radiohead").unwrap());
    }
}
