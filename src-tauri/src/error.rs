use serde::Serialize;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Walk error: {0}")]
    Walk(#[from] walkdir::Error),

    #[error("Metadata error: {0}")]
    Metadata(String),

    #[error("Sync cancelled")]
    SyncCancelled,

    #[error("Profile not found: {0}")]
    ProfileNotFound(String),

    #[error("Playlist not found: {0}")]
    PlaylistNotFound(String),

    #[error("Device not found: {0}")]
    DeviceNotFound(String),

    #[error("Device disconnected: {0}")]
    DeviceDisconnected(String),

    #[error("Path not accessible: {0}")]
    PathNotAccessible(String),

    #[error("Smart playlist not found: {0}")]
    SmartPlaylistNotFound(String),

    #[error("{0}")]
    General(String),
}

impl Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}
