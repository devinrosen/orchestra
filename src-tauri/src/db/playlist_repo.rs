use rusqlite::{params, Connection};

use crate::db::library_repo::track_from_row;
use crate::error::AppError;
use crate::models::playlist::{Playlist, PlaylistWithTracks};
use crate::models::track::Track;

pub fn create_playlist(conn: &Connection, playlist: &Playlist) -> Result<(), AppError> {
    conn.execute(
        "INSERT INTO playlists (id, name, created_at, updated_at) VALUES (?1, ?2, ?3, ?4)",
        params![playlist.id, playlist.name, playlist.created_at, playlist.updated_at],
    )?;
    Ok(())
}

pub fn get_playlist(conn: &Connection, id: &str) -> Result<Playlist, AppError> {
    conn.query_row(
        "SELECT id, name, created_at, updated_at FROM playlists WHERE id = ?1",
        params![id],
        |row| {
            Ok(Playlist {
                id: row.get(0)?,
                name: row.get(1)?,
                created_at: row.get(2)?,
                updated_at: row.get(3)?,
            })
        },
    )
    .map_err(|e| match e {
        rusqlite::Error::QueryReturnedNoRows => AppError::PlaylistNotFound(id.to_string()),
        other => AppError::Database(other),
    })
}

pub fn list_playlists(conn: &Connection) -> Result<Vec<Playlist>, AppError> {
    let mut stmt = conn.prepare(
        "SELECT id, name, created_at, updated_at FROM playlists ORDER BY updated_at DESC",
    )?;
    let playlists = stmt
        .query_map([], |row| {
            Ok(Playlist {
                id: row.get(0)?,
                name: row.get(1)?,
                created_at: row.get(2)?,
                updated_at: row.get(3)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(playlists)
}

pub fn update_playlist(conn: &Connection, playlist: &Playlist) -> Result<(), AppError> {
    let rows = conn.execute(
        "UPDATE playlists SET name = ?2, updated_at = ?3 WHERE id = ?1",
        params![playlist.id, playlist.name, playlist.updated_at],
    )?;
    if rows == 0 {
        return Err(AppError::PlaylistNotFound(playlist.id.clone()));
    }
    Ok(())
}

pub fn delete_playlist(conn: &Connection, id: &str) -> Result<(), AppError> {
    let rows = conn.execute("DELETE FROM playlists WHERE id = ?1", params![id])?;
    if rows == 0 {
        return Err(AppError::PlaylistNotFound(id.to_string()));
    }
    Ok(())
}

pub fn add_tracks(
    conn: &Connection,
    playlist_id: &str,
    track_ids: &[i64],
) -> Result<(), AppError> {
    let max_pos: i32 = conn
        .query_row(
            "SELECT COALESCE(MAX(position), 0) FROM playlist_tracks WHERE playlist_id = ?1",
            params![playlist_id],
            |row| row.get(0),
        )
        .unwrap_or(0);

    let mut stmt = conn.prepare(
        "INSERT OR IGNORE INTO playlist_tracks (playlist_id, track_id, position) VALUES (?1, ?2, ?3)",
    )?;
    for (i, track_id) in track_ids.iter().enumerate() {
        stmt.execute(params![playlist_id, track_id, max_pos + 1 + i as i32])?;
    }
    Ok(())
}

pub fn remove_tracks(
    conn: &Connection,
    playlist_id: &str,
    track_ids: &[i64],
) -> Result<(), AppError> {
    if track_ids.is_empty() {
        return Ok(());
    }

    let placeholders: Vec<String> = (0..track_ids.len())
        .map(|i| format!("?{}", i + 2))
        .collect();
    let sql = format!(
        "DELETE FROM playlist_tracks WHERE playlist_id = ?1 AND track_id IN ({})",
        placeholders.join(",")
    );

    let mut param_values: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();
    param_values.push(Box::new(playlist_id.to_string()));
    for id in track_ids {
        param_values.push(Box::new(*id));
    }
    let params_ref: Vec<&dyn rusqlite::types::ToSql> =
        param_values.iter().map(|p| p.as_ref()).collect();
    conn.execute(&sql, params_ref.as_slice())?;

    // Reorder remaining positions
    reorder_remaining(conn, playlist_id)?;
    Ok(())
}

pub fn reorder_tracks(
    conn: &Connection,
    playlist_id: &str,
    track_ids: &[i64],
) -> Result<(), AppError> {
    conn.execute(
        "DELETE FROM playlist_tracks WHERE playlist_id = ?1",
        params![playlist_id],
    )?;

    let mut stmt = conn.prepare(
        "INSERT INTO playlist_tracks (playlist_id, track_id, position) VALUES (?1, ?2, ?3)",
    )?;
    for (i, track_id) in track_ids.iter().enumerate() {
        stmt.execute(params![playlist_id, track_id, i as i32 + 1])?;
    }
    Ok(())
}

pub fn get_playlist_tracks(conn: &Connection, playlist_id: &str) -> Result<Vec<Track>, AppError> {
    let mut stmt = conn.prepare(
        "SELECT t.id, t.file_path, t.relative_path, t.library_root, t.title, t.artist,
                t.album_artist, t.album, t.track_number, t.disc_number, t.year, t.genre,
                t.duration_secs, t.format, t.file_size, t.modified_at, t.hash, t.has_album_art, t.bitrate, t.scanned_at
         FROM playlist_tracks pt
         JOIN tracks t ON t.id = pt.track_id
         WHERE pt.playlist_id = ?1
         ORDER BY pt.position",
    )?;
    let tracks = stmt
        .query_map(params![playlist_id], track_from_row)?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(tracks)
}

pub fn get_playlist_with_tracks(
    conn: &Connection,
    playlist_id: &str,
) -> Result<PlaylistWithTracks, AppError> {
    let playlist = get_playlist(conn, playlist_id)?;
    let tracks = get_playlist_tracks(conn, playlist_id)?;
    Ok(PlaylistWithTracks { playlist, tracks })
}

fn reorder_remaining(conn: &Connection, playlist_id: &str) -> Result<(), AppError> {
    let mut stmt = conn.prepare(
        "SELECT id FROM playlist_tracks WHERE playlist_id = ?1 ORDER BY position",
    )?;
    let ids: Vec<i64> = stmt
        .query_map(params![playlist_id], |row| row.get(0))?
        .collect::<Result<Vec<_>, _>>()?;

    let mut update = conn.prepare(
        "UPDATE playlist_tracks SET position = ?1 WHERE id = ?2",
    )?;
    for (i, id) in ids.iter().enumerate() {
        update.execute(params![i as i32 + 1, id])?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::library_repo;
    use crate::db::schema;
    use crate::models::playlist::Playlist;
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
    fn test_create_and_get_playlist() {
        let conn = setup_db();
        let pl = Playlist {
            id: "p1".into(),
            name: "My Playlist".into(),
            created_at: 1000,
            updated_at: 1000,
        };
        create_playlist(&conn, &pl).unwrap();
        let fetched = get_playlist(&conn, "p1").unwrap();
        assert_eq!(fetched.name, "My Playlist");
    }

    #[test]
    fn test_list_playlists_ordered_by_updated_at() {
        let conn = setup_db();
        create_playlist(
            &conn,
            &Playlist {
                id: "p1".into(),
                name: "Old".into(),
                created_at: 1000,
                updated_at: 1000,
            },
        )
        .unwrap();
        create_playlist(
            &conn,
            &Playlist {
                id: "p2".into(),
                name: "New".into(),
                created_at: 2000,
                updated_at: 2000,
            },
        )
        .unwrap();
        let list = list_playlists(&conn).unwrap();
        assert_eq!(list.len(), 2);
        assert_eq!(list[0].name, "New"); // most recent first
    }

    #[test]
    fn test_delete_playlist_cascades() {
        let conn = setup_db();
        let track_id = insert_test_track(&conn, "t1");
        create_playlist(
            &conn,
            &Playlist {
                id: "p1".into(),
                name: "PL".into(),
                created_at: 1000,
                updated_at: 1000,
            },
        )
        .unwrap();
        add_tracks(&conn, "p1", &[track_id]).unwrap();
        delete_playlist(&conn, "p1").unwrap();
        // playlist_tracks should be empty due to CASCADE
        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM playlist_tracks WHERE playlist_id = 'p1'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(count, 0);
    }

    #[test]
    fn test_add_and_get_tracks() {
        let conn = setup_db();
        let t1 = insert_test_track(&conn, "t1");
        let t2 = insert_test_track(&conn, "t2");
        create_playlist(
            &conn,
            &Playlist {
                id: "p1".into(),
                name: "PL".into(),
                created_at: 1000,
                updated_at: 1000,
            },
        )
        .unwrap();
        add_tracks(&conn, "p1", &[t1, t2]).unwrap();
        let tracks = get_playlist_tracks(&conn, "p1").unwrap();
        assert_eq!(tracks.len(), 2);
        assert_eq!(tracks[0].id, Some(t1));
        assert_eq!(tracks[1].id, Some(t2));
    }

    #[test]
    fn test_remove_tracks_reorders() {
        let conn = setup_db();
        let t1 = insert_test_track(&conn, "t1");
        let t2 = insert_test_track(&conn, "t2");
        let t3 = insert_test_track(&conn, "t3");
        create_playlist(
            &conn,
            &Playlist {
                id: "p1".into(),
                name: "PL".into(),
                created_at: 1000,
                updated_at: 1000,
            },
        )
        .unwrap();
        add_tracks(&conn, "p1", &[t1, t2, t3]).unwrap();
        remove_tracks(&conn, "p1", &[t2]).unwrap();
        let tracks = get_playlist_tracks(&conn, "p1").unwrap();
        assert_eq!(tracks.len(), 2);
        assert_eq!(tracks[0].id, Some(t1));
        assert_eq!(tracks[1].id, Some(t3));
    }

    #[test]
    fn test_reorder_tracks() {
        let conn = setup_db();
        let t1 = insert_test_track(&conn, "t1");
        let t2 = insert_test_track(&conn, "t2");
        let t3 = insert_test_track(&conn, "t3");
        create_playlist(
            &conn,
            &Playlist {
                id: "p1".into(),
                name: "PL".into(),
                created_at: 1000,
                updated_at: 1000,
            },
        )
        .unwrap();
        add_tracks(&conn, "p1", &[t1, t2, t3]).unwrap();
        reorder_tracks(&conn, "p1", &[t3, t1, t2]).unwrap();
        let tracks = get_playlist_tracks(&conn, "p1").unwrap();
        assert_eq!(tracks[0].id, Some(t3));
        assert_eq!(tracks[1].id, Some(t1));
        assert_eq!(tracks[2].id, Some(t2));
    }

    #[test]
    fn test_playlist_not_found() {
        let conn = setup_db();
        let err = get_playlist(&conn, "nonexistent").unwrap_err();
        assert!(err.to_string().contains("Playlist not found"));
    }

    #[test]
    fn test_update_playlist() {
        let conn = setup_db();
        let mut pl = Playlist {
            id: "p1".into(),
            name: "Original".into(),
            created_at: 1000,
            updated_at: 1000,
        };
        create_playlist(&conn, &pl).unwrap();
        pl.name = "Renamed".to_string();
        pl.updated_at = 2000;
        update_playlist(&conn, &pl).unwrap();
        let fetched = get_playlist(&conn, "p1").unwrap();
        assert_eq!(fetched.name, "Renamed");
        assert_eq!(fetched.updated_at, 2000);
    }
}
