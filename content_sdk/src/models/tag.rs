use chrono::{DateTime, Utc};
use dioxus::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Props)]
pub struct Tag {
    pub id: Option<i32>,
    pub name: String,
    pub slug: String,
    pub parent_id: Option<i32>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
    pub synced_at: Option<DateTime<Utc>>,
}

/// Request structure for creating/updating tags
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Props)]
pub struct TagRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<i32>,
    pub name: String,
    pub slug: String,
    pub parent_id: Option<i32>,
}

impl From<Tag> for TagRequest {
    fn from(tag: Tag) -> Self {
        TagRequest {
            id: tag.id,
            name: tag.name,
            slug: tag.slug,
            parent_id: tag.parent_id,
        }
    }
}
