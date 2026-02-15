use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LibraryRoot {
    pub path: String,
    pub label: Option<String>,
    pub added_at: i64,
}
