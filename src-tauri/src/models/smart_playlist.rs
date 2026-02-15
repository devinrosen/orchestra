use serde::{Deserialize, Serialize};

/// Leaf rule: a single field condition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleCondition {
    pub field: String, // e.g. "genre", "year", "artist"
    pub op: String,    // e.g. "contains", "greater_than", "equals"
    pub value: String, // always a string; numeric parsing happens at match time
}

/// A rule is either a single condition or a logical group of sub-rules.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Rule {
    Condition(RuleCondition),
    Group {
        operator: String, // "and" | "or"
        rules: Vec<Rule>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmartPlaylist {
    pub id: String,
    pub name: String,
    pub rule: Rule,      // deserialized from rule_json on read
    pub created_at: i64,
    pub updated_at: i64,
}

// ---- Request types (consumed by Tauri commands) ----
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateSmartPlaylistRequest {
    pub name: String,
    pub rule: Rule,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateSmartPlaylistRequest {
    pub id: String,
    pub name: Option<String>,
    pub rule: Option<Rule>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmartPlaylistWithTracks {
    pub playlist: SmartPlaylist,
    pub tracks: Vec<crate::models::track::Track>,
}
