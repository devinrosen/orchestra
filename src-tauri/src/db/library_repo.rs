use std::collections::HashMap;
use rusqlite::{params, Connection};

use crate::error::AppError;
use crate::models::device::{AlbumSelection, AlbumSummary, ArtistSummary};
use crate::models::duplicate::{DuplicateGroup, DuplicateMatchType};
use crate::models::track::{AlbumNode, ArtistNode, FormatStat, GenreStat, LibraryStats, LibraryTree, Track};

/// Maps a row from a SELECT that returns all 19 Track columns (id first) to a Track struct.
fn track_from_row(row: &rusqlite::Row) -> rusqlite::Result<Track> {
    Ok(Track {
        id: Some(row.get(0)?),
        file_path: row.get(1)?,
        relative_path: row.get(2)?,
        library_root: row.get(3)?,
        title: row.get(4)?,
        artist: row.get(5)?,
        album_artist: row.get(6)?,
        album: row.get(7)?,
        track_number: row.get(8)?,
        disc_number: row.get(9)?,
        year: row.get(10)?,
        genre: row.get(11)?,
        duration_secs: row.get(12)?,
        format: row.get(13)?,
        file_size: row.get(14)?,
        modified_at: row.get(15)?,
        hash: row.get(16)?,
        has_album_art: row.get(17)?,
        bitrate: row.get(18)?,
    })
}

pub fn upsert_track(conn: &Connection, track: &Track) -> Result<(), AppError> {
    conn.execute(
        "INSERT INTO tracks (file_path, relative_path, library_root, title, artist, album_artist, album,
         track_number, disc_number, year, genre, duration_secs, format, file_size, modified_at, hash, has_album_art, bitrate)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18)
         ON CONFLICT(file_path) DO UPDATE SET
           relative_path=excluded.relative_path, library_root=excluded.library_root,
           title=excluded.title, artist=excluded.artist, album_artist=excluded.album_artist,
           album=excluded.album, track_number=excluded.track_number, disc_number=excluded.disc_number,
           year=excluded.year, genre=excluded.genre, duration_secs=excluded.duration_secs,
           format=excluded.format, file_size=excluded.file_size, modified_at=excluded.modified_at,
           hash=excluded.hash, has_album_art=excluded.has_album_art, bitrate=excluded.bitrate",
        params![
            track.file_path,
            track.relative_path,
            track.library_root,
            track.title,
            track.artist,
            track.album_artist,
            track.album,
            track.track_number,
            track.disc_number,
            track.year,
            track.genre,
            track.duration_secs,
            track.format,
            track.file_size,
            track.modified_at,
            track.hash,
            track.has_album_art,
            track.bitrate,
        ],
    )?;
    Ok(())
}

pub fn remove_tracks_not_in(
    conn: &Connection,
    library_root: &str,
    existing_paths: &[String],
) -> Result<usize, AppError> {
    if existing_paths.is_empty() {
        let deleted = conn.execute(
            "DELETE FROM tracks WHERE library_root = ?1",
            params![library_root],
        )?;
        return Ok(deleted);
    }

    let placeholders: Vec<String> = (0..existing_paths.len()).map(|i| format!("?{}", i + 2)).collect();
    let sql = format!(
        "DELETE FROM tracks WHERE library_root = ?1 AND file_path NOT IN ({})",
        placeholders.join(",")
    );

    let mut param_values: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();
    param_values.push(Box::new(library_root.to_string()));
    for p in existing_paths {
        param_values.push(Box::new(p.clone()));
    }
    let params_ref: Vec<&dyn rusqlite::types::ToSql> = param_values.iter().map(|p| p.as_ref()).collect();

    let deleted = conn.execute(&sql, params_ref.as_slice())?;
    Ok(deleted)
}

/// Returns the set of distinct parent directories (from relative_path) for a library root.
pub fn get_known_directories(
    conn: &Connection,
    library_root: &str,
) -> Result<std::collections::HashSet<String>, AppError> {
    let mut stmt = conn.prepare(
        "SELECT relative_path FROM tracks WHERE library_root = ?1",
    )?;
    let paths: Vec<String> = stmt
        .query_map(params![library_root], |row| row.get(0))?
        .collect::<Result<Vec<_>, _>>()?;

    let mut dirs = std::collections::HashSet::new();
    for path in paths {
        if let Some(parent) = std::path::Path::new(&path).parent() {
            let dir = parent.to_string_lossy().to_string();
            if !dir.is_empty() {
                dirs.insert(dir);
            }
        }
    }
    Ok(dirs)
}

/// Removes all tracks whose relative_path starts with the given directory prefix.
pub fn remove_tracks_by_directory(
    conn: &Connection,
    library_root: &str,
    dir_prefix: &str,
) -> Result<usize, AppError> {
    let pattern = format!("{}/%", dir_prefix);
    let deleted = conn.execute(
        "DELETE FROM tracks WHERE library_root = ?1 AND relative_path LIKE ?2",
        params![library_root, pattern],
    )?;
    Ok(deleted)
}

/// Returns a map of file_path -> (file_size, modified_at) for all tracks in a library root.
/// Used by incremental scan to skip unchanged files.
pub fn get_track_fingerprints(
    conn: &Connection,
    library_root: &str,
) -> Result<HashMap<String, (u64, i64)>, AppError> {
    let mut stmt = conn.prepare(
        "SELECT file_path, file_size, modified_at FROM tracks WHERE library_root = ?1",
    )?;
    let rows = stmt.query_map(params![library_root], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, u64>(1)?,
            row.get::<_, i64>(2)?,
        ))
    })?;
    let mut map = HashMap::new();
    for row in rows {
        let (path, size, mtime) = row?;
        map.insert(path, (size, mtime));
    }
    Ok(map)
}

pub fn get_library_tree(conn: &Connection, library_root: &str) -> Result<LibraryTree, AppError> {
    let mut stmt = conn.prepare(
        "SELECT id, file_path, relative_path, library_root, title, artist, album_artist, album,
         track_number, disc_number, year, genre, duration_secs, format, file_size, modified_at, hash, has_album_art, bitrate
         FROM tracks WHERE library_root = ?1
         ORDER BY COALESCE(album_artist, artist) COLLATE NOCASE,
                  album COLLATE NOCASE,
                  disc_number,
                  track_number",
    )?;

    let tracks = stmt
        .query_map(params![library_root], track_from_row)?
        .collect::<Result<Vec<_>, _>>()?;

    let total_tracks = tracks.len();
    let mut artists: Vec<ArtistNode> = Vec::new();

    for track in tracks {
        let artist_name = track
            .album_artist
            .clone()
            .or_else(|| track.artist.clone())
            .unwrap_or_else(|| "Unknown Artist".to_string());
        let album_name = track
            .album
            .clone()
            .unwrap_or_else(|| "Unknown Album".to_string());

        let artist_node = if let Some(a) = artists.iter_mut().find(|a| a.name == artist_name) {
            a
        } else {
            artists.push(ArtistNode {
                name: artist_name.clone(),
                albums: Vec::new(),
            });
            artists.last_mut().unwrap()
        };

        let album_node = if let Some(a) = artist_node.albums.iter_mut().find(|a| a.name == album_name) {
            a
        } else {
            artist_node.albums.push(AlbumNode {
                name: album_name.clone(),
                year: track.year,
                tracks: Vec::new(),
            });
            artist_node.albums.last_mut().unwrap()
        };

        album_node.tracks.push(track);
    }

    Ok(LibraryTree {
        root: library_root.to_string(),
        artists,
        total_tracks,
    })
}

pub fn list_artists(conn: &Connection, library_root: &str) -> Result<Vec<ArtistSummary>, AppError> {
    let mut stmt = conn.prepare(
        "SELECT COALESCE(album_artist, artist, 'Unknown Artist') as display_artist,
                COUNT(DISTINCT album) as album_count,
                COUNT(*) as track_count,
                SUM(file_size) as total_size
         FROM tracks
         WHERE library_root = ?1
         GROUP BY display_artist
         ORDER BY display_artist COLLATE NOCASE",
    )?;
    let artists = stmt
        .query_map(params![library_root], |row| {
            Ok(ArtistSummary {
                name: row.get(0)?,
                album_count: row.get::<_, i64>(1)? as usize,
                track_count: row.get::<_, i64>(2)? as usize,
                total_size: row.get::<_, i64>(3)? as u64,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(artists)
}

pub fn search_tracks(conn: &Connection, query: &str) -> Result<Vec<Track>, AppError> {
    let pattern = format!("%{}%", query);
    let mut stmt = conn.prepare(
        "SELECT id, file_path, relative_path, library_root, title, artist, album_artist, album,
         track_number, disc_number, year, genre, duration_secs, format, file_size, modified_at, hash, has_album_art, bitrate
         FROM tracks
         WHERE title LIKE ?1 OR artist LIKE ?1 OR album LIKE ?1 OR album_artist LIKE ?1
         ORDER BY artist COLLATE NOCASE, album COLLATE NOCASE, track_number
         LIMIT 200",
    )?;

    let tracks = stmt
        .query_map(params![pattern], track_from_row)?
        .collect::<Result<Vec<_>, _>>()?;

    Ok(tracks)
}

pub fn get_tracks_for_device(
    conn: &Connection,
    library_root: &str,
    artist_names: &[String],
    albums: &[AlbumSelection],
) -> Result<Vec<Track>, AppError> {
    if artist_names.is_empty() && albums.is_empty() {
        return Ok(vec![]);
    }

    // Build a UNION query for artist tracks and album tracks
    let mut param_values: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();
    let mut query_parts = Vec::new();
    let mut idx = 1;

    let select_cols = "id, file_path, relative_path, library_root, title, artist, album_artist, album,
         track_number, disc_number, year, genre, duration_secs, format, file_size, modified_at, hash, has_album_art, bitrate";

    if !artist_names.is_empty() {
        let lib_param = format!("?{}", idx);
        param_values.push(Box::new(library_root.to_string()));
        idx += 1;

        let placeholders: Vec<String> = artist_names.iter().map(|_| {
            let p = format!("?{}", idx);
            idx += 1;
            p
        }).collect();
        for name in artist_names {
            param_values.push(Box::new(name.clone()));
        }

        query_parts.push(format!(
            "SELECT {} FROM tracks WHERE library_root = {} AND COALESCE(album_artist, artist, 'Unknown Artist') IN ({})",
            select_cols, lib_param, placeholders.join(",")
        ));
    }

    if !albums.is_empty() {
        let lib_param = format!("?{}", idx);
        param_values.push(Box::new(library_root.to_string()));
        idx += 1;

        let mut conditions = Vec::new();
        for album in albums {
            conditions.push(format!(
                "(COALESCE(album_artist, artist, 'Unknown Artist') = ?{} AND COALESCE(album, 'Unknown Album') = ?{})",
                idx, idx + 1
            ));
            param_values.push(Box::new(album.artist_name.clone()));
            param_values.push(Box::new(album.album_name.clone()));
            idx += 2;
        }

        query_parts.push(format!(
            "SELECT {} FROM tracks WHERE library_root = {} AND ({})",
            select_cols, lib_param, conditions.join(" OR ")
        ));
    }

    let sql = format!(
        "{} ORDER BY COALESCE(album_artist, artist) COLLATE NOCASE, album COLLATE NOCASE, disc_number, track_number",
        query_parts.join(" UNION ")
    );

    let params_ref: Vec<&dyn rusqlite::types::ToSql> =
        param_values.iter().map(|p| p.as_ref()).collect();

    let mut stmt = conn.prepare(&sql)?;
    let tracks = stmt
        .query_map(params_ref.as_slice(), track_from_row)?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(tracks)
}

pub fn list_albums(conn: &Connection, library_root: &str) -> Result<Vec<AlbumSummary>, AppError> {
    let mut stmt = conn.prepare(
        "SELECT COALESCE(album_artist, artist, 'Unknown Artist') as display_artist,
                COALESCE(album, 'Unknown Album') as display_album,
                COUNT(*) as track_count,
                SUM(file_size) as total_size,
                MAX(year) as year
         FROM tracks
         WHERE library_root = ?1
         GROUP BY display_artist, display_album
         ORDER BY display_artist COLLATE NOCASE, display_album COLLATE NOCASE",
    )?;
    let albums = stmt
        .query_map(params![library_root], |row| {
            Ok(AlbumSummary {
                artist_name: row.get(0)?,
                album_name: row.get(1)?,
                track_count: row.get::<_, i64>(2)? as usize,
                total_size: row.get::<_, i64>(3)? as u64,
                year: row.get(4)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(albums)
}

pub fn get_incomplete_tracks(
    conn: &Connection,
    library_root: &str,
) -> Result<Vec<Track>, AppError> {
    let mut stmt = conn.prepare(
        "SELECT id, file_path, relative_path, library_root, title, artist, album_artist, album,
         track_number, disc_number, year, genre, duration_secs, format, file_size, modified_at, hash, has_album_art, bitrate
         FROM tracks
         WHERE library_root = ?1
           AND (title IS NULL OR artist IS NULL OR album IS NULL OR has_album_art = 0)
         ORDER BY COALESCE(album_artist, artist) COLLATE NOCASE, album COLLATE NOCASE, track_number",
    )?;

    let tracks = stmt
        .query_map(params![library_root], track_from_row)?
        .collect::<Result<Vec<_>, _>>()?;

    Ok(tracks)
}

pub fn get_library_stats(conn: &Connection, library_root: &str) -> Result<LibraryStats, AppError> {
    // Summary row
    let (total_tracks, total_size, total_duration, avg_bitrate): (usize, u64, f64, Option<f64>) =
        conn.query_row(
            "SELECT COUNT(*), COALESCE(SUM(file_size), 0), COALESCE(SUM(duration_secs), 0.0),
                    AVG(bitrate)
             FROM tracks WHERE library_root = ?1",
            params![library_root],
            |row| Ok((
                row.get::<_, i64>(0)? as usize,
                row.get::<_, i64>(1)? as u64,
                row.get::<_, f64>(2)?,
                row.get::<_, Option<f64>>(3)?,
            )),
        )?;

    let total_artists: usize = conn.query_row(
        "SELECT COUNT(DISTINCT COALESCE(album_artist, artist, 'Unknown Artist'))
         FROM tracks WHERE library_root = ?1",
        params![library_root],
        |row| row.get::<_, i64>(0).map(|v| v as usize),
    )?;

    let total_albums: usize = conn.query_row(
        "SELECT COUNT(DISTINCT COALESCE(album, 'Unknown Album'))
         FROM tracks WHERE library_root = ?1",
        params![library_root],
        |row| row.get::<_, i64>(0).map(|v| v as usize),
    )?;

    // Format breakdown
    let mut fmt_stmt = conn.prepare(
        "SELECT format, COUNT(*), COALESCE(SUM(file_size), 0)
         FROM tracks WHERE library_root = ?1
         GROUP BY format ORDER BY COUNT(*) DESC",
    )?;
    let formats = fmt_stmt
        .query_map(params![library_root], |row| {
            Ok(FormatStat {
                format: row.get(0)?,
                count: row.get::<_, i64>(1)? as usize,
                total_size: row.get::<_, i64>(2)? as u64,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

    // Genre breakdown
    let mut genre_stmt = conn.prepare(
        "SELECT COALESCE(genre, 'Unknown'), COUNT(*)
         FROM tracks WHERE library_root = ?1
         GROUP BY COALESCE(genre, 'Unknown') ORDER BY COUNT(*) DESC",
    )?;
    let genres = genre_stmt
        .query_map(params![library_root], |row| {
            Ok(GenreStat {
                genre: row.get(0)?,
                count: row.get::<_, i64>(1)? as usize,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

    Ok(LibraryStats {
        total_tracks,
        total_artists,
        total_albums,
        total_size,
        total_duration_secs: total_duration,
        avg_bitrate,
        formats,
        genres,
    })
}

pub fn find_hash_duplicates(
    conn: &Connection,
    library_root: &str,
) -> Result<Vec<DuplicateGroup>, AppError> {
    // Step 1: Find hashes that appear more than once
    let mut hash_stmt = conn.prepare(
        "SELECT hash, COUNT(*) as cnt
         FROM tracks
         WHERE library_root = ?1 AND hash IS NOT NULL
         GROUP BY hash
         HAVING cnt > 1
         ORDER BY cnt DESC",
    )?;
    let dup_hashes: Vec<String> = hash_stmt
        .query_map(params![library_root], |row| row.get::<_, String>(0))?
        .collect::<Result<Vec<_>, _>>()?;

    // Step 2: For each hash, fetch the full tracks
    let mut groups = Vec::new();
    for hash in dup_hashes {
        let mut track_stmt = conn.prepare(
            "SELECT id, file_path, relative_path, library_root, title, artist, album_artist, album,
             track_number, disc_number, year, genre, duration_secs, format, file_size, modified_at, hash, has_album_art, bitrate
             FROM tracks
             WHERE library_root = ?1 AND hash = ?2
             ORDER BY file_path",
        )?;
        let tracks = track_stmt
            .query_map(params![library_root, hash], track_from_row)?
            .collect::<Result<Vec<_>, _>>()?;

        groups.push(DuplicateGroup {
            match_type: DuplicateMatchType::ContentHash,
            match_key: hash,
            tracks,
        });
    }

    Ok(groups)
}

pub fn find_metadata_duplicates(
    conn: &Connection,
    library_root: &str,
) -> Result<Vec<DuplicateGroup>, AppError> {
    // Group by lowercase title + artist + duration rounded to nearest second
    let mut stmt = conn.prepare(
        "SELECT LOWER(title) as lt, LOWER(COALESCE(artist, '')), CAST(ROUND(duration_secs) AS INTEGER) as dur,
                COUNT(*) as cnt
         FROM tracks
         WHERE library_root = ?1 AND title IS NOT NULL
         GROUP BY lt, LOWER(COALESCE(artist, '')), dur
         HAVING cnt > 1
         ORDER BY cnt DESC",
    )?;
    let keys: Vec<(String, String, i64)> = stmt
        .query_map(params![library_root], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, i64>(2)?,
            ))
        })?
        .collect::<Result<Vec<_>, _>>()?;

    let mut groups = Vec::new();
    for (title, artist, dur) in keys {
        let mut track_stmt = conn.prepare(
            "SELECT id, file_path, relative_path, library_root, title, artist, album_artist, album,
             track_number, disc_number, year, genre, duration_secs, format, file_size, modified_at, hash, has_album_art, bitrate
             FROM tracks
             WHERE library_root = ?1
               AND LOWER(title) = ?2
               AND LOWER(COALESCE(artist, '')) = ?3
               AND CAST(ROUND(duration_secs) AS INTEGER) = ?4
             ORDER BY file_path",
        )?;
        let tracks = track_stmt
            .query_map(params![library_root, title, artist, dur], track_from_row)?
            .collect::<Result<Vec<_>, _>>()?;

        // Skip groups that are entirely the same hash (already covered by content match)
        let all_same_hash = tracks.len() > 1
            && tracks[0].hash.is_some()
            && tracks.iter().all(|t| t.hash == tracks[0].hash);
        if all_same_hash {
            continue;
        }

        let match_key = format!("{}|{}|{}", title, artist, dur);
        groups.push(DuplicateGroup {
            match_type: DuplicateMatchType::MetadataSimilarity,
            match_key,
            tracks,
        });
    }

    Ok(groups)
}

pub fn get_tracks_without_hash(
    conn: &Connection,
    library_root: &str,
) -> Result<Vec<(i64, String)>, AppError> {
    let mut stmt = conn.prepare(
        "SELECT id, file_path FROM tracks WHERE library_root = ?1 AND hash IS NULL",
    )?;
    let rows = stmt
        .query_map(params![library_root], |row| {
            Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?))
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}

pub fn update_track_hash(conn: &Connection, track_id: i64, hash: &str) -> Result<(), AppError> {
    conn.execute(
        "UPDATE tracks SET hash = ?1 WHERE id = ?2",
        params![hash, track_id],
    )?;
    Ok(())
}

pub fn delete_tracks_by_ids(conn: &Connection, ids: &[i64]) -> Result<usize, AppError> {
    if ids.is_empty() {
        return Ok(0);
    }
    let placeholders: Vec<String> = (0..ids.len()).map(|i| format!("?{}", i + 1)).collect();
    let sql = format!(
        "DELETE FROM tracks WHERE id IN ({})",
        placeholders.join(",")
    );
    let param_values: Vec<Box<dyn rusqlite::types::ToSql>> = ids
        .iter()
        .map(|id| Box::new(*id) as Box<dyn rusqlite::types::ToSql>)
        .collect();
    let params_ref: Vec<&dyn rusqlite::types::ToSql> =
        param_values.iter().map(|p| p.as_ref()).collect();
    let deleted = conn.execute(&sql, params_ref.as_slice())?;
    Ok(deleted)
}

#[cfg(test)]
fn setup_db() -> Connection {
    use crate::db::schema;
    let conn = Connection::open_in_memory().unwrap();
    schema::run_migrations(&conn).unwrap();
    conn
}

#[cfg(test)]
fn make_track(
    artist: &str, album: &str, format: &str, genre: &str,
    size: u64, duration: f64, bitrate: Option<u32>,
    file_suffix: &str,
) -> Track {
    Track {
        id: None,
        file_path: format!("/music/{}/{}/{}.{}", artist, album, file_suffix, format),
        relative_path: format!("{}/{}/{}.{}", artist, album, file_suffix, format),
        library_root: "/music".to_string(),
        title: Some("Track".to_string()),
        artist: Some(artist.to_string()),
        album_artist: None,
        album: Some(album.to_string()),
        track_number: Some(1),
        disc_number: Some(1),
        year: Some(2024),
        genre: Some(genre.to_string()),
        duration_secs: Some(duration),
        format: format.to_string(),
        file_size: size,
        modified_at: 1700000000,
        hash: None,
        has_album_art: false,
        bitrate,
    }
}

#[cfg(test)]
mod stats_tests {
    use super::*;

    #[test]
    fn test_empty_library_stats() {
        let conn = setup_db();
        let stats = get_library_stats(&conn, "/music").unwrap();
        assert_eq!(stats.total_tracks, 0);
        assert_eq!(stats.total_artists, 0);
        assert_eq!(stats.total_albums, 0);
        assert_eq!(stats.total_size, 0);
        assert_eq!(stats.total_duration_secs, 0.0);
        assert!(stats.avg_bitrate.is_none());
        assert!(stats.formats.is_empty());
        assert!(stats.genres.is_empty());
    }

    #[test]
    fn test_stats_counts_and_totals() {
        let conn = setup_db();
        upsert_track(&conn, &make_track("ArtistA", "Album1", "flac", "Rock", 50_000_000, 300.0, Some(1411), "t1")).unwrap();
        upsert_track(&conn, &make_track("ArtistA", "Album1", "flac", "Rock", 48_000_000, 280.0, Some(1411), "t2")).unwrap();
        upsert_track(&conn, &make_track("ArtistB", "Album2", "mp3", "Jazz", 8_000_000, 240.0, Some(320), "t3")).unwrap();

        let stats = get_library_stats(&conn, "/music").unwrap();
        assert_eq!(stats.total_tracks, 3);
        assert_eq!(stats.total_artists, 2);
        assert_eq!(stats.total_albums, 2);
        assert_eq!(stats.total_size, 106_000_000);
        assert!((stats.total_duration_secs - 820.0).abs() < 0.01);
    }

    #[test]
    fn test_stats_format_breakdown() {
        let conn = setup_db();
        upsert_track(&conn, &make_track("A", "A1", "flac", "Rock", 50_000_000, 300.0, None, "t1")).unwrap();
        upsert_track(&conn, &make_track("A", "A1", "flac", "Rock", 48_000_000, 280.0, None, "t2")).unwrap();
        upsert_track(&conn, &make_track("B", "B1", "mp3", "Jazz", 8_000_000, 240.0, None, "t3")).unwrap();

        let stats = get_library_stats(&conn, "/music").unwrap();
        assert_eq!(stats.formats.len(), 2);
        assert_eq!(stats.formats[0].format, "flac");
        assert_eq!(stats.formats[0].count, 2);
        assert_eq!(stats.formats[1].format, "mp3");
        assert_eq!(stats.formats[1].count, 1);
    }

    #[test]
    fn test_stats_genre_breakdown() {
        let conn = setup_db();
        upsert_track(&conn, &make_track("A", "A1", "flac", "Rock", 50_000_000, 300.0, None, "t1")).unwrap();
        upsert_track(&conn, &make_track("B", "B1", "mp3", "Rock", 8_000_000, 240.0, None, "t2")).unwrap();
        upsert_track(&conn, &make_track("C", "C1", "flac", "Jazz", 45_000_000, 300.0, None, "t3")).unwrap();

        let stats = get_library_stats(&conn, "/music").unwrap();
        assert_eq!(stats.genres.len(), 2);
        assert_eq!(stats.genres[0].genre, "Rock");
        assert_eq!(stats.genres[0].count, 2);
        assert_eq!(stats.genres[1].genre, "Jazz");
        assert_eq!(stats.genres[1].count, 1);
    }

    #[test]
    fn test_stats_avg_bitrate() {
        let conn = setup_db();
        upsert_track(&conn, &make_track("A", "A1", "flac", "Rock", 50_000_000, 300.0, Some(1411), "t1")).unwrap();
        upsert_track(&conn, &make_track("B", "B1", "mp3", "Jazz", 8_000_000, 240.0, Some(320), "t2")).unwrap();

        let stats = get_library_stats(&conn, "/music").unwrap();
        let avg = stats.avg_bitrate.unwrap();
        assert!((avg - 865.5).abs() < 1.0); // (1411 + 320) / 2
    }

    #[test]
    fn test_stats_avg_bitrate_with_nulls() {
        let conn = setup_db();
        upsert_track(&conn, &make_track("A", "A1", "flac", "Rock", 50_000_000, 300.0, Some(1000), "t1")).unwrap();
        upsert_track(&conn, &make_track("B", "B1", "mp3", "Jazz", 8_000_000, 240.0, None, "t2")).unwrap();

        let stats = get_library_stats(&conn, "/music").unwrap();
        // AVG ignores NULLs in SQLite, so only the 1000 is counted
        let avg = stats.avg_bitrate.unwrap();
        assert!((avg - 1000.0).abs() < 1.0);
    }

    #[test]
    fn test_stats_scoped_to_library_root() {
        let conn = setup_db();
        upsert_track(&conn, &make_track("A", "A1", "flac", "Rock", 50_000_000, 300.0, None, "t1")).unwrap();
        upsert_track(&conn, &{
            let mut t = make_track("B", "B1", "mp3", "Jazz", 8_000_000, 240.0, None, "t2");
            t.file_path = "/other/B/B1/t2.mp3".to_string();
            t.relative_path = "B/B1/t2.mp3".to_string();
            t.library_root = "/other".to_string();
            t
        }).unwrap();

        let stats = get_library_stats(&conn, "/music").unwrap();
        assert_eq!(stats.total_tracks, 1);
        assert_eq!(stats.total_artists, 1);
        assert_eq!(stats.formats.len(), 1);
    }
}

#[cfg(test)]
mod duplicate_tests {
    use super::*;

    #[test]
    fn test_find_hash_duplicates_empty_library() {
        let conn = setup_db();
        let groups = find_hash_duplicates(&conn, "/music").unwrap();
        assert!(groups.is_empty());
    }

    #[test]
    fn test_find_hash_duplicates_no_duplicates() {
        let conn = setup_db();
        let mut t1 = make_track("A", "A1", "flac", "Rock", 50_000_000, 300.0, None, "t1");
        t1.hash = Some("hash_a".to_string());
        let mut t2 = make_track("A", "A1", "flac", "Rock", 48_000_000, 280.0, None, "t2");
        t2.hash = Some("hash_b".to_string());
        let mut t3 = make_track("B", "B1", "mp3", "Jazz", 8_000_000, 240.0, None, "t3");
        t3.hash = Some("hash_c".to_string());
        upsert_track(&conn, &t1).unwrap();
        upsert_track(&conn, &t2).unwrap();
        upsert_track(&conn, &t3).unwrap();

        let groups = find_hash_duplicates(&conn, "/music").unwrap();
        assert!(groups.is_empty());
    }

    #[test]
    fn test_find_hash_duplicates_with_duplicates() {
        let conn = setup_db();
        let mut t1 = make_track("A", "A1", "flac", "Rock", 50_000_000, 300.0, None, "t1");
        t1.hash = Some("hash_same".to_string());
        let mut t2 = make_track("A", "A1", "flac", "Rock", 48_000_000, 280.0, None, "t2");
        t2.hash = Some("hash_same".to_string());
        let mut t3 = make_track("B", "B1", "mp3", "Jazz", 8_000_000, 240.0, None, "t3");
        t3.hash = Some("hash_unique".to_string());
        upsert_track(&conn, &t1).unwrap();
        upsert_track(&conn, &t2).unwrap();
        upsert_track(&conn, &t3).unwrap();

        let groups = find_hash_duplicates(&conn, "/music").unwrap();
        assert_eq!(groups.len(), 1);
        assert_eq!(groups[0].tracks.len(), 2);
        assert_eq!(groups[0].match_type, DuplicateMatchType::ContentHash);
    }

    #[test]
    fn test_find_hash_duplicates_ignores_null_hashes() {
        let conn = setup_db();
        let t1 = make_track("A", "A1", "flac", "Rock", 50_000_000, 300.0, None, "t1");
        let t2 = make_track("A", "A1", "flac", "Rock", 48_000_000, 280.0, None, "t2");
        upsert_track(&conn, &t1).unwrap();
        upsert_track(&conn, &t2).unwrap();

        let groups = find_hash_duplicates(&conn, "/music").unwrap();
        assert!(groups.is_empty());
    }

    #[test]
    fn test_find_metadata_duplicates_empty_library() {
        let conn = setup_db();
        let groups = find_metadata_duplicates(&conn, "/music").unwrap();
        assert!(groups.is_empty());
    }

    #[test]
    fn test_find_metadata_duplicates_same_title_artist_duration() {
        let conn = setup_db();
        let mut t1 = make_track("Artist", "Album1", "flac", "Rock", 50_000_000, 180.0, None, "t1");
        t1.title = Some("Song".to_string());
        t1.hash = Some("hash_a".to_string());
        let mut t2 = make_track("Artist", "Album2", "mp3", "Rock", 8_000_000, 180.0, None, "t2");
        t2.title = Some("Song".to_string());
        t2.hash = Some("hash_b".to_string());
        upsert_track(&conn, &t1).unwrap();
        upsert_track(&conn, &t2).unwrap();

        let groups = find_metadata_duplicates(&conn, "/music").unwrap();
        assert_eq!(groups.len(), 1);
        assert_eq!(groups[0].tracks.len(), 2);
        assert_eq!(groups[0].match_type, DuplicateMatchType::MetadataSimilarity);
    }

    #[test]
    fn test_find_metadata_duplicates_skips_already_hash_matched() {
        let conn = setup_db();
        let mut t1 = make_track("Artist", "Album1", "flac", "Rock", 50_000_000, 180.0, None, "t1");
        t1.title = Some("Song".to_string());
        t1.hash = Some("hash_same".to_string());
        let mut t2 = make_track("Artist", "Album2", "mp3", "Rock", 8_000_000, 180.0, None, "t2");
        t2.title = Some("Song".to_string());
        t2.hash = Some("hash_same".to_string());
        upsert_track(&conn, &t1).unwrap();
        upsert_track(&conn, &t2).unwrap();

        let groups = find_metadata_duplicates(&conn, "/music").unwrap();
        assert!(groups.is_empty());
    }

    #[test]
    fn test_find_metadata_duplicates_duration_bucket_tolerance() {
        let conn = setup_db();
        let mut t1 = make_track("Artist", "Album1", "flac", "Rock", 50_000_000, 179.6, None, "t1");
        t1.title = Some("Song".to_string());
        t1.hash = Some("hash_a".to_string());
        let mut t2 = make_track("Artist", "Album2", "mp3", "Rock", 8_000_000, 180.4, None, "t2");
        t2.title = Some("Song".to_string());
        t2.hash = Some("hash_b".to_string());
        upsert_track(&conn, &t1).unwrap();
        upsert_track(&conn, &t2).unwrap();

        let groups = find_metadata_duplicates(&conn, "/music").unwrap();
        assert_eq!(groups.len(), 1);
    }

    #[test]
    fn test_find_metadata_duplicates_case_insensitive() {
        let conn = setup_db();
        let mut t1 = make_track("Beatles", "Album1", "flac", "Rock", 50_000_000, 180.0, None, "t1");
        t1.title = Some("My Song".to_string());
        t1.hash = Some("hash_a".to_string());
        let mut t2 = make_track("beatles", "Album2", "mp3", "Rock", 8_000_000, 180.0, None, "t2");
        t2.title = Some("my song".to_string());
        t2.hash = Some("hash_b".to_string());
        upsert_track(&conn, &t1).unwrap();
        upsert_track(&conn, &t2).unwrap();

        let groups = find_metadata_duplicates(&conn, "/music").unwrap();
        assert_eq!(groups.len(), 1);
    }

    #[test]
    fn test_get_tracks_without_hash() {
        let conn = setup_db();
        let t1 = make_track("A", "A1", "flac", "Rock", 50_000_000, 300.0, None, "t1");
        let t2 = make_track("A", "A1", "flac", "Rock", 48_000_000, 280.0, None, "t2");
        let mut t3 = make_track("B", "B1", "mp3", "Jazz", 8_000_000, 240.0, None, "t3");
        t3.hash = Some("abc".to_string());
        upsert_track(&conn, &t1).unwrap();
        upsert_track(&conn, &t2).unwrap();
        upsert_track(&conn, &t3).unwrap();

        let unhashed = get_tracks_without_hash(&conn, "/music").unwrap();
        assert_eq!(unhashed.len(), 2);
    }

    #[test]
    fn test_update_track_hash() {
        let conn = setup_db();
        let t1 = make_track("A", "A1", "flac", "Rock", 50_000_000, 300.0, None, "t1");
        upsert_track(&conn, &t1).unwrap();

        // Get the track id
        let unhashed = get_tracks_without_hash(&conn, "/music").unwrap();
        assert_eq!(unhashed.len(), 1);
        let (id, _) = &unhashed[0];

        update_track_hash(&conn, *id, "new_hash").unwrap();

        // Verify hash is updated
        let unhashed_after = get_tracks_without_hash(&conn, "/music").unwrap();
        assert!(unhashed_after.is_empty());

        // Verify via direct query
        let hash: Option<String> = conn
            .query_row("SELECT hash FROM tracks WHERE id = ?1", params![id], |row| {
                row.get(0)
            })
            .unwrap();
        assert_eq!(hash, Some("new_hash".to_string()));
    }

    #[test]
    fn test_delete_tracks_by_ids() {
        let conn = setup_db();
        upsert_track(&conn, &make_track("A", "A1", "flac", "Rock", 50_000_000, 300.0, None, "t1")).unwrap();
        upsert_track(&conn, &make_track("A", "A1", "flac", "Rock", 48_000_000, 280.0, None, "t2")).unwrap();
        upsert_track(&conn, &make_track("B", "B1", "mp3", "Jazz", 8_000_000, 240.0, None, "t3")).unwrap();

        // Get all track ids
        let mut stmt = conn.prepare("SELECT id FROM tracks ORDER BY id").unwrap();
        let ids: Vec<i64> = stmt
            .query_map([], |row| row.get(0))
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap();
        assert_eq!(ids.len(), 3);

        let deleted = delete_tracks_by_ids(&conn, &[ids[0], ids[1]]).unwrap();
        assert_eq!(deleted, 2);

        let remaining: i64 = conn
            .query_row("SELECT COUNT(*) FROM tracks", [], |row| row.get(0))
            .unwrap();
        assert_eq!(remaining, 1);
    }

    #[test]
    fn test_delete_tracks_by_ids_empty() {
        let conn = setup_db();
        let deleted = delete_tracks_by_ids(&conn, &[]).unwrap();
        assert_eq!(deleted, 0);
    }

    #[test]
    fn test_find_hash_duplicates_scoped_to_library_root() {
        let conn = setup_db();
        let mut t1 = make_track("A", "A1", "flac", "Rock", 50_000_000, 300.0, None, "t1");
        t1.hash = Some("hash_same".to_string());
        let mut t2 = make_track("B", "B1", "mp3", "Jazz", 8_000_000, 240.0, None, "t2");
        t2.hash = Some("hash_same".to_string());
        t2.file_path = "/other/B/B1/t2.mp3".to_string();
        t2.relative_path = "B/B1/t2.mp3".to_string();
        t2.library_root = "/other".to_string();
        upsert_track(&conn, &t1).unwrap();
        upsert_track(&conn, &t2).unwrap();

        // Each library root has only 1 track with that hash, so no duplicates
        let groups = find_hash_duplicates(&conn, "/music").unwrap();
        assert!(groups.is_empty());

        let groups = find_hash_duplicates(&conn, "/other").unwrap();
        assert!(groups.is_empty());
    }
}
