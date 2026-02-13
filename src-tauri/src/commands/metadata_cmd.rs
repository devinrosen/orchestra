use std::path::Path;
use std::sync::Mutex;
use rusqlite::Connection;

use crate::db::library_repo;
use crate::error::AppError;
use crate::models::track::{AlbumArt, Track, TrackMetadataUpdate};
use crate::scanner::{metadata, writer};

#[tauri::command]
pub async fn get_track_artwork(file_path: String) -> Result<Option<AlbumArt>, AppError> {
    let path = Path::new(&file_path);
    if !path.exists() {
        return Err(AppError::PathNotAccessible(file_path));
    }
    writer::extract_artwork(path)
}

#[tauri::command]
pub async fn update_track_metadata(
    db: tauri::State<'_, Mutex<Connection>>,
    updates: Vec<TrackMetadataUpdate>,
) -> Result<Vec<Track>, AppError> {
    let mut updated_tracks = Vec::with_capacity(updates.len());

    for update in &updates {
        let path = Path::new(&update.file_path);
        if !path.exists() {
            return Err(AppError::PathNotAccessible(update.file_path.clone()));
        }

        // Write metadata to the audio file
        writer::write_metadata(path, update)?;
    }

    // Re-read metadata and update DB for all tracks
    let conn = db.lock().map_err(|e| AppError::General(e.to_string()))?;
    for update in &updates {
        let path = Path::new(&update.file_path);

        // Re-read to get updated mtime/size and confirm written values
        let existing = conn.query_row(
            "SELECT library_root FROM tracks WHERE file_path = ?1",
            rusqlite::params![update.file_path],
            |row| row.get::<_, String>(0),
        ).map_err(|_| AppError::General(format!("Track not found in DB: {}", update.file_path)))?;

        let library_root = Path::new(&existing);
        let mut track = metadata::extract_metadata(path, library_root)?;
        track.hash = None; // Clear stale hash — will be recomputed on next diff

        library_repo::upsert_track(&conn, &track)?;
        track.id = Some(conn.last_insert_rowid());
        updated_tracks.push(track);
    }

    Ok(updated_tracks)
}
