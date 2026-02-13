use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ConflictType {
    BothModified,
    DeletedAndModified,
    FirstSyncDiffers,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Conflict {
    pub relative_path: String,
    pub conflict_type: ConflictType,
    pub source_hash: Option<String>,
    pub target_hash: Option<String>,
    pub source_modified: Option<i64>,
    pub target_modified: Option<i64>,
    pub source_size: Option<u64>,
    pub target_size: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum Resolution {
    KeepSource,
    KeepTarget,
    KeepBoth,
    Skip,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConflictResolution {
    pub relative_path: String,
    pub resolution: Resolution,
}
