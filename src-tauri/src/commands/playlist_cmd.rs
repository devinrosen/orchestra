use std::sync::Mutex;
use rusqlite::Connection;

use crate::db::playlist_repo;
use crate::error::AppError;
use crate::models::playlist::{
    AddTracksRequest, CreatePlaylistRequest, Playlist, PlaylistWithTracks, RemoveTracksRequest,
    ReorderTracksRequest, UpdatePlaylistRequest,
};

#[tauri::command]
pub async fn create_playlist(
    db: tauri::State<'_, Mutex<Connection>>,
    request: CreatePlaylistRequest,
) -> Result<PlaylistWithTracks, AppError> {
    let now = chrono::Utc::now().timestamp();
    let playlist = Playlist {
        id: uuid::Uuid::new_v4().to_string(),
        name: request.name,
        created_at: now,
        updated_at: now,
    };

    let conn = db.lock().map_err(|e| AppError::General(e.to_string()))?;
    playlist_repo::create_playlist(&conn, &playlist)?;
    Ok(PlaylistWithTracks {
        playlist,
        tracks: vec![],
    })
}

#[tauri::command]
pub async fn list_playlists(
    db: tauri::State<'_, Mutex<Connection>>,
) -> Result<Vec<Playlist>, AppError> {
    let conn = db.lock().map_err(|e| AppError::General(e.to_string()))?;
    playlist_repo::list_playlists(&conn)
}

#[tauri::command]
pub async fn get_playlist(
    db: tauri::State<'_, Mutex<Connection>>,
    id: String,
) -> Result<PlaylistWithTracks, AppError> {
    let conn = db.lock().map_err(|e| AppError::General(e.to_string()))?;
    playlist_repo::get_playlist_with_tracks(&conn, &id)
}

#[tauri::command]
pub async fn update_playlist(
    db: tauri::State<'_, Mutex<Connection>>,
    request: UpdatePlaylistRequest,
) -> Result<Playlist, AppError> {
    let conn = db.lock().map_err(|e| AppError::General(e.to_string()))?;
    let mut playlist = playlist_repo::get_playlist(&conn, &request.id)?;

    if let Some(name) = request.name {
        playlist.name = name;
    }
    playlist.updated_at = chrono::Utc::now().timestamp();

    playlist_repo::update_playlist(&conn, &playlist)?;
    Ok(playlist)
}

#[tauri::command]
pub async fn delete_playlist(
    db: tauri::State<'_, Mutex<Connection>>,
    id: String,
) -> Result<(), AppError> {
    let conn = db.lock().map_err(|e| AppError::General(e.to_string()))?;
    playlist_repo::delete_playlist(&conn, &id)
}

#[tauri::command]
pub async fn add_tracks_to_playlist(
    db: tauri::State<'_, Mutex<Connection>>,
    request: AddTracksRequest,
) -> Result<PlaylistWithTracks, AppError> {
    let conn = db.lock().map_err(|e| AppError::General(e.to_string()))?;
    playlist_repo::add_tracks(&conn, &request.playlist_id, &request.track_ids)?;

    // Touch updated_at
    let mut playlist = playlist_repo::get_playlist(&conn, &request.playlist_id)?;
    playlist.updated_at = chrono::Utc::now().timestamp();
    playlist_repo::update_playlist(&conn, &playlist)?;

    playlist_repo::get_playlist_with_tracks(&conn, &request.playlist_id)
}

#[tauri::command]
pub async fn remove_tracks_from_playlist(
    db: tauri::State<'_, Mutex<Connection>>,
    request: RemoveTracksRequest,
) -> Result<PlaylistWithTracks, AppError> {
    let conn = db.lock().map_err(|e| AppError::General(e.to_string()))?;
    playlist_repo::remove_tracks(&conn, &request.playlist_id, &request.track_ids)?;

    // Touch updated_at
    let mut playlist = playlist_repo::get_playlist(&conn, &request.playlist_id)?;
    playlist.updated_at = chrono::Utc::now().timestamp();
    playlist_repo::update_playlist(&conn, &playlist)?;

    playlist_repo::get_playlist_with_tracks(&conn, &request.playlist_id)
}

#[tauri::command]
pub async fn reorder_playlist(
    db: tauri::State<'_, Mutex<Connection>>,
    request: ReorderTracksRequest,
) -> Result<PlaylistWithTracks, AppError> {
    let conn = db.lock().map_err(|e| AppError::General(e.to_string()))?;
    playlist_repo::reorder_tracks(&conn, &request.playlist_id, &request.track_ids)?;

    // Touch updated_at
    let mut playlist = playlist_repo::get_playlist(&conn, &request.playlist_id)?;
    playlist.updated_at = chrono::Utc::now().timestamp();
    playlist_repo::update_playlist(&conn, &playlist)?;

    playlist_repo::get_playlist_with_tracks(&conn, &request.playlist_id)
}

#[tauri::command]
pub async fn export_playlist(
    db: tauri::State<'_, Mutex<Connection>>,
    id: String,
    format: String,
    path: String,
) -> Result<(), AppError> {
    let conn = db.lock().map_err(|e| AppError::General(e.to_string()))?;
    let pwt = playlist_repo::get_playlist_with_tracks(&conn, &id)?;

    let content = match format.to_lowercase().as_str() {
        "m3u" => format_m3u(&pwt),
        "pls" => format_pls(&pwt),
        _ => return Err(AppError::General(format!("Unsupported format: {}", format))),
    };

    std::fs::write(&path, content)?;
    Ok(())
}

fn format_m3u(pwt: &PlaylistWithTracks) -> String {
    let mut lines = vec!["#EXTM3U".to_string()];
    for track in &pwt.tracks {
        let duration = track.duration_secs.map(|d| d as i64).unwrap_or(-1);
        let artist = track.artist.as_deref().unwrap_or("Unknown");
        let title = track.title.as_deref().unwrap_or("Unknown");
        lines.push(format!("#EXTINF:{},{} - {}", duration, artist, title));
        lines.push(track.file_path.clone());
    }
    lines.join("\n")
}

fn format_pls(pwt: &PlaylistWithTracks) -> String {
    let mut lines = vec!["[playlist]".to_string()];
    for (i, track) in pwt.tracks.iter().enumerate() {
        let num = i + 1;
        let artist = track.artist.as_deref().unwrap_or("Unknown");
        let title = track.title.as_deref().unwrap_or("Unknown");
        let duration = track.duration_secs.map(|d| d as i64).unwrap_or(-1);
        lines.push(format!("File{}={}", num, track.file_path));
        lines.push(format!("Title{}={} - {}", num, artist, title));
        lines.push(format!("Length{}={}", num, duration));
    }
    lines.push(format!("NumberOfEntries={}", pwt.tracks.len()));
    lines.push("Version=2".to_string());
    lines.join("\n")
}
