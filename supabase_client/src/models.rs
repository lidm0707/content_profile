use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Content status constants
pub mod status {
    pub const DRAFT: &str = "draft";
    pub const PUBLISHED: &str = "published";
    pub const ARCHIVED: &str = "archived";
}

/// Content model representing a CMS content item
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Content {
    pub id: Option<i32>,
    pub title: String,
    pub slug: String,
    pub body: String,
    pub status: String,
    pub tags: Option<Vec<i32>>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
    pub synced_at: Option<DateTime<Utc>>,
}

impl Content {
    pub fn new(title: String, slug: String, body: String) -> Self {
        Content {
            id: None,
            title,
            slug,
            body,
            status: status::DRAFT.to_string(),
            tags: None,
            created_at: None,
            updated_at: None,
            synced_at: None,
        }
    }

    pub fn with_status(mut self, status: String) -> Self {
        self.status = status;
        self
    }

    pub fn generate_slug(title: &str) -> String {
        title
            .to_lowercase()
            .chars()
            .map(|c| {
                if c.is_alphanumeric() || c == '-' || c == '_' {
                    c
                } else {
                    '-'
                }
            })
            .collect::<String>()
            .split('-')
            .filter(|s| !s.is_empty())
            .collect::<Vec<&str>>()
            .join("-")
    }
}

/// Request structure for creating/updating content
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ContentRequest {
    pub title: String,
    pub slug: String,
    pub body: String,
    pub status: String,
    pub tags: Option<Vec<i32>>,
}

impl From<Content> for ContentRequest {
    fn from(content: Content) -> Self {
        ContentRequest {
            title: content.title,
            slug: content.slug,
            body: content.body,
            status: content.status,
            tags: content.tags,
        }
    }
}

/// Type alias for status string
pub type Status = String;
