use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrganizePreviewItem {
    pub track_id: i64,
    pub current_relative_path: String,
    pub proposed_relative_path: String,
    pub status: OrganizeItemStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum OrganizeItemStatus {
    #[serde(rename = "Ok")]
    Ok,
    #[serde(rename = "AlreadyCorrect")]
    AlreadyCorrect,
    #[serde(rename = "Collision")]
    Collision { conflicting_track_id: Option<i64> },
    #[serde(rename = "Error")]
    Error { reason: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrganizePreviewResult {
    pub items: Vec<OrganizePreviewItem>,
    pub total: usize,
    pub already_correct: usize,
    pub collisions: usize,
    pub errors: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrganizeApplyItem {
    pub track_id: i64,
    pub current_file_path: String,
    pub proposed_relative_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrganizeApplyResult {
    pub moved: usize,
    pub skipped: usize,
    pub errors: Vec<String>,
}
