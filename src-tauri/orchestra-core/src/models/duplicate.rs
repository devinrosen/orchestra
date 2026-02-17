use serde::{Deserialize, Serialize};

use crate::models::track::Track;

/// How duplicates were detected
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum DuplicateMatchType {
    ContentHash,
    MetadataSimilarity,
}

/// A group of tracks that are duplicates of each other
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DuplicateGroup {
    pub match_type: DuplicateMatchType,
    /// The shared key (hash for content, "title|artist|duration" for metadata)
    pub match_key: String,
    pub tracks: Vec<Track>,
}

/// Full result returned to the frontend
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DuplicateResult {
    pub groups: Vec<DuplicateGroup>,
    pub total_duplicate_tracks: usize,
    pub total_wasted_bytes: u64,
}
