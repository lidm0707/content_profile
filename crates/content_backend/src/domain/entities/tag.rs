use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Tag {
    pub id: Option<i32>,
    pub name: String,
    pub slug: String,
    pub parent_id: Option<i32>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

impl Tag {
    pub fn new(name: String, slug: String, parent_id: Option<i32>) -> Self {
        Self {
            id: None,
            name,
            slug,
            parent_id,
            created_at: None,
            updated_at: None,
        }
    }
}
