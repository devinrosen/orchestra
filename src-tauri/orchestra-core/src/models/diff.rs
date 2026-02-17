use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum DiffAction {
    Add,
    Remove,
    Update,
    Unchanged,
    Conflict,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum DiffDirection {
    SourceToTarget,
    TargetToSource,
    Both,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffEntry {
    pub relative_path: String,
    pub action: DiffAction,
    pub direction: DiffDirection,
    pub source_size: Option<u64>,
    pub target_size: Option<u64>,
    pub source_hash: Option<String>,
    pub target_hash: Option<String>,
    pub source_modified: Option<i64>,
    pub target_modified: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffResult {
    pub profile_id: String,
    pub entries: Vec<DiffEntry>,
    pub total_add: usize,
    pub total_remove: usize,
    pub total_update: usize,
    pub total_conflict: usize,
    pub total_unchanged: usize,
    pub bytes_to_transfer: u64,
}
