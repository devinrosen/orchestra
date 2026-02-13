use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Track {
    pub id: Option<i64>,
    pub file_path: String,
    pub relative_path: String,
    pub library_root: String,
    pub title: Option<String>,
    pub artist: Option<String>,
    pub album_artist: Option<String>,
    pub album: Option<String>,
    pub track_number: Option<u32>,
    pub disc_number: Option<u32>,
    pub year: Option<i32>,
    pub genre: Option<String>,
    pub duration_secs: Option<f64>,
    pub format: String,
    pub file_size: u64,
    pub modified_at: i64,
    pub hash: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtistNode {
    pub name: String,
    pub albums: Vec<AlbumNode>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlbumNode {
    pub name: String,
    pub year: Option<i32>,
    pub tracks: Vec<Track>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LibraryTree {
    pub root: String,
    pub artists: Vec<ArtistNode>,
    pub total_tracks: usize,
}

pub const AUDIO_EXTENSIONS: &[&str] = &[
    "flac", "mp3", "m4a", "aac", "wav", "alac", "ogg", "opus", "wma",
];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackMetadataUpdate {
    pub file_path: String,
    pub title: Option<String>,
    pub artist: Option<String>,
    pub album_artist: Option<String>,
    pub album: Option<String>,
    pub track_number: Option<u32>,
    pub disc_number: Option<u32>,
    pub year: Option<i32>,
    pub genre: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlbumArt {
    pub data: String,
    pub mime_type: String,
}

pub fn is_audio_file(path: &std::path::Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .map(|e| AUDIO_EXTENSIONS.contains(&e.to_lowercase().as_str()))
        .unwrap_or(false)
}
