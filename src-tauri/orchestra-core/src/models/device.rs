use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Device {
    pub id: String,
    pub name: String,
    pub volume_uuid: String,
    pub volume_name: String,
    pub mount_path: Option<String>,
    pub capacity_bytes: Option<u64>,
    pub music_folder: String,
    pub created_at: i64,
    pub last_synced_at: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlbumSelection {
    pub artist_name: String,
    pub album_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlbumSummary {
    pub artist_name: String,
    pub album_name: String,
    pub track_count: usize,
    pub total_size: u64,
    pub year: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceWithStatus {
    pub device: Device,
    pub connected: bool,
    pub selected_artists: Vec<String>,
    pub selected_albums: Vec<AlbumSelection>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectedVolume {
    pub volume_uuid: String,
    pub volume_name: String,
    pub mount_path: String,
    pub capacity_bytes: u64,
    pub free_bytes: u64,
    pub bus_protocol: String,
    pub already_registered: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterDeviceRequest {
    pub name: String,
    pub volume_uuid: String,
    pub volume_name: String,
    pub mount_path: String,
    pub capacity_bytes: Option<u64>,
    pub music_folder: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtistSummary {
    pub name: String,
    pub album_count: usize,
    pub track_count: usize,
    pub total_size: u64,
}
