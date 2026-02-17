use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum SyncMode {
    OneWay,
    TwoWay,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncProfile {
    pub id: String,
    pub name: String,
    pub source_path: String,
    pub target_path: String,
    pub sync_mode: SyncMode,
    pub exclude_patterns: Vec<String>,
    pub created_at: i64,
    pub last_synced_at: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateProfileRequest {
    pub name: String,
    pub source_path: String,
    pub target_path: String,
    pub sync_mode: SyncMode,
    pub exclude_patterns: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateProfileRequest {
    pub id: String,
    pub name: Option<String>,
    pub source_path: Option<String>,
    pub target_path: Option<String>,
    pub sync_mode: Option<SyncMode>,
    pub exclude_patterns: Option<Vec<String>>,
}
