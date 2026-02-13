use std::collections::HashMap;
use rusqlite::{params, Connection};

use crate::error::AppError;
use crate::models::device::ArtistSummary;
use crate::models::track::{AlbumNode, ArtistNode, LibraryTree, Track};

pub fn upsert_track(conn: &Connection, track: &Track) -> Result<(), AppError> {
    conn.execute(
        "INSERT INTO tracks (file_path, relative_path, library_root, title, artist, album_artist, album,
         track_number, disc_number, year, genre, duration_secs, format, file_size, modified_at, hash)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16)
         ON CONFLICT(file_path) DO UPDATE SET
           relative_path=excluded.relative_path, library_root=excluded.library_root,
           title=excluded.title, artist=excluded.artist, album_artist=excluded.album_artist,
           album=excluded.album, track_number=excluded.track_number, disc_number=excluded.disc_number,
           year=excluded.year, genre=excluded.genre, duration_secs=excluded.duration_secs,
           format=excluded.format, file_size=excluded.file_size, modified_at=excluded.modified_at,
           hash=excluded.hash",
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
         track_number, disc_number, year, genre, duration_secs, format, file_size, modified_at, hash
         FROM tracks WHERE library_root = ?1
         ORDER BY COALESCE(album_artist, artist) COLLATE NOCASE,
                  album COLLATE NOCASE,
                  disc_number,
                  track_number",
    )?;

    let tracks = stmt
        .query_map(params![library_root], |row| {
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
            })
        })?
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

pub fn get_tracks_by_artists(
    conn: &Connection,
    library_root: &str,
    artist_names: &[String],
) -> Result<Vec<Track>, AppError> {
    if artist_names.is_empty() {
        return Ok(vec![]);
    }

    let placeholders: Vec<String> = (0..artist_names.len())
        .map(|i| format!("?{}", i + 2))
        .collect();
    let sql = format!(
        "SELECT id, file_path, relative_path, library_root, title, artist, album_artist, album,
         track_number, disc_number, year, genre, duration_secs, format, file_size, modified_at, hash
         FROM tracks
         WHERE library_root = ?1
           AND COALESCE(album_artist, artist, 'Unknown Artist') IN ({})
         ORDER BY COALESCE(album_artist, artist) COLLATE NOCASE, album COLLATE NOCASE, disc_number, track_number",
        placeholders.join(",")
    );

    let mut param_values: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();
    param_values.push(Box::new(library_root.to_string()));
    for name in artist_names {
        param_values.push(Box::new(name.clone()));
    }
    let params_ref: Vec<&dyn rusqlite::types::ToSql> =
        param_values.iter().map(|p| p.as_ref()).collect();

    let mut stmt = conn.prepare(&sql)?;
    let tracks = stmt
        .query_map(params_ref.as_slice(), |row| {
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
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(tracks)
}

pub fn search_tracks(conn: &Connection, query: &str) -> Result<Vec<Track>, AppError> {
    let pattern = format!("%{}%", query);
    let mut stmt = conn.prepare(
        "SELECT id, file_path, relative_path, library_root, title, artist, album_artist, album,
         track_number, disc_number, year, genre, duration_secs, format, file_size, modified_at, hash
         FROM tracks
         WHERE title LIKE ?1 OR artist LIKE ?1 OR album LIKE ?1 OR album_artist LIKE ?1
         ORDER BY artist COLLATE NOCASE, album COLLATE NOCASE, track_number
         LIMIT 200",
    )?;

    let tracks = stmt
        .query_map(params![pattern], |row| {
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
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

    Ok(tracks)
}
