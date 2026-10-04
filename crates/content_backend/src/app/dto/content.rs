use crate::domain::{Content, STATUS_DRAFT};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentDto {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<i32>,
    pub title: String,
    pub slug: String,
    pub body: String,
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<DateTime<Utc>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub synced_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpsertContent {
    pub title: String,
    #[serde(default)]
    pub slug: String,
    pub body: String,
    #[serde(default = "default_status")]
    pub status: String,
}

fn default_status() -> String {
    STATUS_DRAFT.to_string()
}

impl From<&Content> for ContentDto {
    fn from(c: &Content) -> Self {
        Self {
            id: c.id,
            title: c.title.clone(),
            slug: c.slug.clone(),
            body: c.body.clone(),
            status: c.status.clone(),
            created_at: c.created_at,
            updated_at: c.updated_at,
            synced_at: None,
        }
    }
}

impl From<ContentDto> for Content {
    fn from(d: ContentDto) -> Self {
        Self {
            id: d.id,
            title: d.title,
            slug: d.slug,
            body: d.body,
            status: d.status,
            created_at: d.created_at,
            updated_at: d.updated_at,
        }
    }
}
