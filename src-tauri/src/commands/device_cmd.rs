use std::path::Path;
use std::sync::Mutex;
use rusqlite::Connection;
use tauri::ipc::Channel;

use crate::db::{device_repo, library_repo};
use crate::device::{detect, sync as device_sync};
use crate::error::AppError;
use crate::models::device::{AlbumSelection, AlbumSummary, ArtistSummary, DetectedVolume, DeviceWithStatus, RegisterDeviceRequest};
use crate::models::diff::DiffResult;
use crate::models::progress::ProgressEvent;
use crate::sync::progress::CancelToken;

#[tauri::command]
pub async fn detect_volumes(
    db: tauri::State<'_, Mutex<Connection>>,
) -> Result<Vec<DetectedVolume>, AppError> {
    let mut volumes = detect::detect_usb_volumes()?;

    // Cross-reference with saved devices and update mount paths
    let conn = db.lock().map_err(|e| AppError::General(e.to_string()))?;
    for vol in &mut volumes {
        if let Ok(Some(device)) = device_repo::get_device_by_uuid(&conn, &vol.volume_uuid) {
            vol.already_registered = true;
            // Update mount_path if the device reconnected at a different path
            if device.mount_path.as_deref() != Some(&vol.mount_path) {
                let _ = device_repo::update_mount_path(&conn, &device.id, &vol.mount_path);
            }
        }
    }

    Ok(volumes)
}

#[tauri::command]
pub async fn register_device(
    db: tauri::State<'_, Mutex<Connection>>,
    request: RegisterDeviceRequest,
) -> Result<DeviceWithStatus, AppError> {
    let conn = db.lock().map_err(|e| AppError::General(e.to_string()))?;

    // Check if already registered by UUID
    let existing = device_repo::get_device_by_uuid(&conn, &request.volume_uuid)?;
    let device = if let Some(mut existing) = existing {
        // Update existing
        existing.name = request.name;
        existing.mount_path = Some(request.mount_path);
        existing.capacity_bytes = request.capacity_bytes;
        existing.music_folder = request.music_folder;
        device_repo::save_device(&conn, &existing)?;
        existing
    } else {
        let device = crate::models::device::Device {
            id: uuid::Uuid::new_v4().to_string(),
            name: request.name,
            volume_uuid: request.volume_uuid,
            volume_name: request.volume_name,
            mount_path: Some(request.mount_path),
            capacity_bytes: request.capacity_bytes,
            music_folder: request.music_folder,
            created_at: chrono::Utc::now().timestamp(),
            last_synced_at: None,
        };
        device_repo::save_device(&conn, &device)?;
        device
    };

    let connected = device
        .mount_path
        .as_ref()
        .map(|p| Path::new(p).exists())
        .unwrap_or(false);

    Ok(DeviceWithStatus {
        device,
        connected,
        selected_artists: vec![],
        selected_albums: vec![],
    })
}

#[tauri::command]
pub async fn list_devices(
    db: tauri::State<'_, Mutex<Connection>>,
) -> Result<Vec<DeviceWithStatus>, AppError> {
    let conn = db.lock().map_err(|e| AppError::General(e.to_string()))?;
    let devices = device_repo::list_devices(&conn)?;

    let mut result = Vec::new();
    for device in devices {
        let connected = device
            .mount_path
            .as_ref()
            .map(|p| Path::new(p).exists())
            .unwrap_or(false);
        let selected_artists = device_repo::get_selected_artists(&conn, &device.id)?;
        let selected_albums = device_repo::get_selected_albums(&conn, &device.id)?;
        result.push(DeviceWithStatus {
            device,
            connected,
            selected_artists,
            selected_albums,
        });
    }

    Ok(result)
}

#[tauri::command]
pub async fn delete_device(
    db: tauri::State<'_, Mutex<Connection>>,
    device_id: String,
) -> Result<(), AppError> {
    let conn = db.lock().map_err(|e| AppError::General(e.to_string()))?;
    device_repo::delete_device(&conn, &device_id)
}

#[tauri::command]
pub async fn set_device_artists(
    db: tauri::State<'_, Mutex<Connection>>,
    device_id: String,
    artists: Vec<String>,
) -> Result<(), AppError> {
    let conn = db.lock().map_err(|e| AppError::General(e.to_string()))?;
    // Verify device exists
    let _ = device_repo::get_device(&conn, &device_id)?;
    device_repo::set_selected_artists(&conn, &device_id, &artists)
}

#[tauri::command]
pub async fn compute_device_diff(
    db: tauri::State<'_, Mutex<Connection>>,
    device_id: String,
    on_progress: Channel<ProgressEvent>,
) -> Result<DiffResult, AppError> {
    let (device, selected_artists, selected_albums, library_root, hash_cache) = {
        let conn = db.lock().map_err(|e| AppError::General(e.to_string()))?;
        let device = device_repo::get_device(&conn, &device_id)?;
        let artists = device_repo::get_selected_artists(&conn, &device.id)?;
        let albums = device_repo::get_selected_albums(&conn, &device.id)?;
        let cache = device_repo::get_file_cache(&conn, &device_id)?;

        // Get library root from settings
        let mut stmt = conn.prepare("SELECT value FROM settings WHERE key = 'library_root'")?;
        let library_root: String = stmt
            .query_row([], |row| row.get(0))
            .map_err(|_| AppError::General("Library root not configured".to_string()))?;

        (device, artists, albums, library_root, cache)
    };

    let mount_path = device
        .mount_path
        .as_ref()
        .ok_or_else(|| AppError::DeviceDisconnected(device.name.clone()))?;

    if !Path::new(mount_path).exists() {
        return Err(AppError::DeviceDisconnected(device.name.clone()));
    }

    let device_root = if device.music_folder.is_empty() {
        Path::new(mount_path).to_path_buf()
    } else {
        Path::new(mount_path).join(&device.music_folder)
    };

    // Get tracks for selected artists and albums
    let tracks = {
        let conn = db.lock().map_err(|e| AppError::General(e.to_string()))?;
        library_repo::get_tracks_for_device(&conn, &library_root, &selected_artists, &selected_albums)?
    };

    let (diff, new_cache) = device_sync::compute_device_diff(
        &device_id, &tracks, &device_root, &on_progress, &hash_cache,
    )?;

    // Persist updated cache (includes any new hashes computed during diff)
    {
        let conn = db.lock().map_err(|e| AppError::General(e.to_string()))?;
        device_repo::save_file_cache(&conn, &device_id, &new_cache)?;
    }

    Ok(diff)
}

#[tauri::command]
pub async fn execute_device_sync(
    db: tauri::State<'_, Mutex<Connection>>,
    cancel_token: tauri::State<'_, Mutex<CancelToken>>,
    device_id: String,
    diff_result: DiffResult,
    on_progress: Channel<ProgressEvent>,
) -> Result<usize, AppError> {
    let (device, library_root, pre_cache) = {
        let conn = db.lock().map_err(|e| AppError::General(e.to_string()))?;
        let device = device_repo::get_device(&conn, &device_id)?;
        let cache_map = device_repo::get_file_cache(&conn, &device_id)?;
        let cache_vec: Vec<_> = cache_map.into_values().collect();

        let mut stmt = conn.prepare("SELECT value FROM settings WHERE key = 'library_root'")?;
        let library_root: String = stmt
            .query_row([], |row| row.get(0))
            .map_err(|_| AppError::General("Library root not configured".to_string()))?;

        (device, library_root, cache_vec)
    };

    let mount_path = device
        .mount_path
        .as_ref()
        .ok_or_else(|| AppError::DeviceDisconnected(device.name.clone()))?;

    if !Path::new(mount_path).exists() {
        return Err(AppError::DeviceDisconnected(device.name.clone()));
    }

    let device_root = if device.music_folder.is_empty() {
        Path::new(mount_path).to_path_buf()
    } else {
        Path::new(mount_path).join(&device.music_folder)
    };

    // Reset cancel token
    let flag = {
        let token = cancel_token
            .lock()
            .map_err(|e| AppError::General(e.to_string()))?;
        token.flag()
    };

    let (count, post_cache) = device_sync::execute_device_sync(
        &diff_result,
        Path::new(&library_root),
        &device_root,
        flag,
        &on_progress,
        pre_cache,
    )?;

    // Update last_synced_at and save updated cache
    {
        let conn = db.lock().map_err(|e| AppError::General(e.to_string()))?;
        let now = chrono::Utc::now().timestamp();
        device_repo::update_last_synced(&conn, &device_id, now)?;
        device_repo::save_file_cache(&conn, &device_id, &post_cache)?;
    }

    Ok(count)
}

#[tauri::command]
pub async fn set_device_albums(
    db: tauri::State<'_, Mutex<Connection>>,
    device_id: String,
    albums: Vec<AlbumSelection>,
) -> Result<(), AppError> {
    let conn = db.lock().map_err(|e| AppError::General(e.to_string()))?;
    let _ = device_repo::get_device(&conn, &device_id)?;
    device_repo::set_selected_albums(&conn, &device_id, &albums)
}

#[tauri::command]
pub async fn list_artists(
    db: tauri::State<'_, Mutex<Connection>>,
) -> Result<Vec<ArtistSummary>, AppError> {
    let conn = db.lock().map_err(|e| AppError::General(e.to_string()))?;

    let mut stmt = conn.prepare("SELECT value FROM settings WHERE key = 'library_root'")?;
    let library_root: String = stmt
        .query_row([], |row| row.get(0))
        .map_err(|_| AppError::General("Library root not configured".to_string()))?;

    library_repo::list_artists(&conn, &library_root)
}

#[tauri::command]
pub async fn list_albums(
    db: tauri::State<'_, Mutex<Connection>>,
) -> Result<Vec<AlbumSummary>, AppError> {
    let conn = db.lock().map_err(|e| AppError::General(e.to_string()))?;

    let mut stmt = conn.prepare("SELECT value FROM settings WHERE key = 'library_root'")?;
    let library_root: String = stmt
        .query_row([], |row| row.get(0))
        .map_err(|_| AppError::General("Library root not configured".to_string()))?;

    library_repo::list_albums(&conn, &library_root)
}
