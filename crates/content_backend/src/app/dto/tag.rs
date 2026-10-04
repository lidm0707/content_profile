use crate::domain::Tag;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TagDto {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<i32>,
    pub name: String,
    pub slug: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_id: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<DateTime<Utc>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub synced_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpsertTag {
    pub name: String,
    #[serde(default)]
    pub slug: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_id: Option<i32>,
}

impl From<&Tag> for TagDto {
    fn from(t: &Tag) -> Self {
        Self {
            id: t.id,
            name: t.name.clone(),
            slug: t.slug.clone(),
            parent_id: t.parent_id,
            created_at: t.created_at,
            updated_at: t.updated_at,
            synced_at: None,
        }
    }
}

impl From<TagDto> for Tag {
    fn from(d: TagDto) -> Self {
        Self {
            id: d.id,
            name: d.name,
            slug: d.slug,
            parent_id: d.parent_id,
            created_at: d.created_at,
            updated_at: d.updated_at,
        }
    }
}
