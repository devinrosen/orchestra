use rusqlite::{params, Connection};

use crate::db::library_repo::track_from_row;
use crate::error::AppError;
use crate::models::track::Track;

/// Insert a play event. Called when a track starts playing.
pub fn record_play(conn: &Connection, track_id: i64) -> Result<(), AppError> {
    let now = chrono::Utc::now().timestamp();
    conn.execute(
        "INSERT INTO play_history (track_id, played_at) VALUES (?1, ?2)",
        params![track_id, now],
    )?;
    Ok(())
}

/// Tracks added/scanned within the last `days` days, most recent first. Capped at `limit`.
pub fn get_recently_added(
    conn: &Connection,
    days: u32,
    limit: usize,
) -> Result<Vec<Track>, AppError> {
    let cutoff = chrono::Utc::now().timestamp() - (days as i64 * 86_400);
    let mut stmt = conn.prepare(
        "SELECT id, file_path, relative_path, library_root, title, artist, album_artist, album,
                track_number, disc_number, year, genre, duration_secs, format, file_size,
                modified_at, hash, has_album_art, bitrate, scanned_at
         FROM tracks
         WHERE scanned_at >= ?1
         ORDER BY scanned_at DESC
         LIMIT ?2",
    )?;
    let tracks = stmt
        .query_map(params![cutoff, limit as i64], track_from_row)?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(tracks)
}

/// Most recently played tracks (distinct), most recent first. Capped at `limit`.
pub fn get_recently_played(conn: &Connection, limit: usize) -> Result<Vec<Track>, AppError> {
    let mut stmt = conn.prepare(
        "SELECT t.id, t.file_path, t.relative_path, t.library_root, t.title, t.artist,
                t.album_artist, t.album, t.track_number, t.disc_number, t.year, t.genre,
                t.duration_secs, t.format, t.file_size, t.modified_at, t.hash,
                t.has_album_art, t.bitrate, t.scanned_at
         FROM play_history ph
         JOIN tracks t ON t.id = ph.track_id
         GROUP BY ph.track_id
         ORDER BY MAX(ph.played_at) DESC
         LIMIT ?1",
    )?;
    let tracks = stmt
        .query_map(params![limit as i64], track_from_row)?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(tracks)
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

    /// Force-set scanned_at on a track for testing windowed queries.
    fn set_scanned_at(conn: &Connection, track_id: i64, scanned_at: i64) {
        conn.execute(
            "UPDATE tracks SET scanned_at = ?1 WHERE id = ?2",
            params![scanned_at, track_id],
        )
        .unwrap();
    }

    #[test]
    fn test_record_play_inserts_row() {
        let conn = setup_db();
        let track_id = insert_test_track(&conn, "t1");
        record_play(&conn, track_id).unwrap();

        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM play_history", [], |r| r.get(0))
            .unwrap();
        assert_eq!(count, 1);

        let stored_track_id: i64 = conn
            .query_row(
                "SELECT track_id FROM play_history LIMIT 1",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(stored_track_id, track_id);
    }

    #[test]
    fn test_record_play_allows_duplicates() {
        let conn = setup_db();
        let track_id = insert_test_track(&conn, "t1");
        record_play(&conn, track_id).unwrap();
        record_play(&conn, track_id).unwrap();

        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM play_history", [], |r| r.get(0))
            .unwrap();
        assert_eq!(count, 2);
    }

    #[test]
    fn test_get_recently_added_within_window() {
        let conn = setup_db();
        let now = chrono::Utc::now().timestamp();

        let t1 = insert_test_track(&conn, "t1");
        set_scanned_at(&conn, t1, now - 86_400); // 1 day ago — within 30-day window

        let t2 = insert_test_track(&conn, "t2");
        set_scanned_at(&conn, t2, now - 40 * 86_400); // 40 days ago — outside window

        let results = get_recently_added(&conn, 30, 50).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, Some(t1));
    }

    #[test]
    fn test_get_recently_added_ordering() {
        let conn = setup_db();
        let now = chrono::Utc::now().timestamp();

        let t1 = insert_test_track(&conn, "t1");
        set_scanned_at(&conn, t1, now);

        let t2 = insert_test_track(&conn, "t2");
        set_scanned_at(&conn, t2, now - 86_400); // 1 day ago

        let t3 = insert_test_track(&conn, "t3");
        set_scanned_at(&conn, t3, now - 2 * 86_400); // 2 days ago

        let results = get_recently_added(&conn, 30, 50).unwrap();
        assert_eq!(results.len(), 3);
        assert_eq!(results[0].id, Some(t1));
        assert_eq!(results[1].id, Some(t2));
        assert_eq!(results[2].id, Some(t3));
    }

    #[test]
    fn test_get_recently_added_respects_limit() {
        let conn = setup_db();
        let now = chrono::Utc::now().timestamp();

        for i in 0..10 {
            let id = insert_test_track(&conn, &format!("t{}", i));
            set_scanned_at(&conn, id, now - i * 3600);
        }

        let results = get_recently_added(&conn, 30, 5).unwrap();
        assert_eq!(results.len(), 5);
    }

    #[test]
    fn test_get_recently_added_empty_when_all_old() {
        let conn = setup_db();
        let now = chrono::Utc::now().timestamp();

        let t1 = insert_test_track(&conn, "t1");
        set_scanned_at(&conn, t1, now - 60 * 86_400); // 60 days ago

        let t2 = insert_test_track(&conn, "t2");
        set_scanned_at(&conn, t2, now - 90 * 86_400); // 90 days ago

        let results = get_recently_added(&conn, 30, 50).unwrap();
        assert!(results.is_empty());
    }

    #[test]
    fn test_get_recently_played_most_recent_first() {
        let conn = setup_db();
        let now = chrono::Utc::now().timestamp();

        let t_a = insert_test_track(&conn, "ta");
        let t_b = insert_test_track(&conn, "tb");

        // Play A first, then B
        conn.execute(
            "INSERT INTO play_history (track_id, played_at) VALUES (?1, ?2)",
            params![t_a, now - 100],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO play_history (track_id, played_at) VALUES (?1, ?2)",
            params![t_b, now],
        )
        .unwrap();

        let results = get_recently_played(&conn, 50).unwrap();
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].id, Some(t_b)); // B played most recently
        assert_eq!(results[1].id, Some(t_a));
    }

    #[test]
    fn test_get_recently_played_deduplicates() {
        let conn = setup_db();
        let now = chrono::Utc::now().timestamp();

        let t_a = insert_test_track(&conn, "ta");
        let t_b = insert_test_track(&conn, "tb");

        // Play A three times
        for i in 0..3 {
            conn.execute(
                "INSERT INTO play_history (track_id, played_at) VALUES (?1, ?2)",
                params![t_a, now - (i * 100)],
            )
            .unwrap();
        }
        // Play B once
        conn.execute(
            "INSERT INTO play_history (track_id, played_at) VALUES (?1, ?2)",
            params![t_b, now - 50],
        )
        .unwrap();

        let results = get_recently_played(&conn, 50).unwrap();
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_get_recently_played_respects_limit() {
        let conn = setup_db();
        let now = chrono::Utc::now().timestamp();

        for i in 0..10 {
            let id = insert_test_track(&conn, &format!("t{}", i));
            conn.execute(
                "INSERT INTO play_history (track_id, played_at) VALUES (?1, ?2)",
                params![id, now - i * 60],
            )
            .unwrap();
        }

        let results = get_recently_played(&conn, 3).unwrap();
        assert_eq!(results.len(), 3);
    }

    #[test]
    fn test_get_recently_played_empty() {
        let conn = setup_db();
        let results = get_recently_played(&conn, 50).unwrap();
        assert!(results.is_empty());
    }

    #[test]
    fn test_play_history_cascade_delete() {
        let conn = setup_db();
        let track_id = insert_test_track(&conn, "t1");
        record_play(&conn, track_id).unwrap();

        // Verify play exists
        let count_before: i64 = conn
            .query_row("SELECT COUNT(*) FROM play_history", [], |r| r.get(0))
            .unwrap();
        assert_eq!(count_before, 1);

        // Delete the track
        conn.execute("DELETE FROM tracks WHERE id = ?1", params![track_id])
            .unwrap();

        // Play history should be empty due to ON DELETE CASCADE
        let count_after: i64 = conn
            .query_row("SELECT COUNT(*) FROM play_history", [], |r| r.get(0))
            .unwrap();
        assert_eq!(count_after, 0);
    }
}
