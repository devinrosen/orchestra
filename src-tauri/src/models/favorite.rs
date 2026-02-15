use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Favorite {
    pub entity_type: String,
    pub entity_id: String,
    pub created_at: i64,
}
