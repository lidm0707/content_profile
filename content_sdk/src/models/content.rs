use chrono::{DateTime, Utc};
use dioxus::prelude::*;
use serde::{Deserialize, Deserializer, Serialize};

/// Content status constants
pub const STATUS_DRAFT: &str = "draft";
pub const STATUS_PUBLISHED: &str = "published";

/// Custom deserializer for optional datetime fields that handles empty strings
fn deserialize_optional_datetime<'de, D>(deserializer: D) -> Result<Option<DateTime<Utc>>, D::Error>
where
    D: Deserializer<'de>,
{
    let opt = Option::<String>::deserialize(deserializer)?;
    match opt.as_deref() {
        None | Some("") => Ok(None),
        Some(s) => DateTime::parse_from_rfc3339(s)
            .map(|dt| Some(dt.with_timezone(&Utc)))
            .map_err(serde::de::Error::custom),
    }
}

/// Content model representing a CMS content item
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Props)]
pub struct Content {
    pub id: Option<i32>,
    pub title: String,
    pub slug: String,
    pub body: String,
    pub status: String,
    pub tags: Option<Vec<i32>>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
    #[serde(deserialize_with = "deserialize_optional_datetime")]
    pub synced_at: Option<DateTime<Utc>>,
}

impl Content {
    /// Creates a new content item with default values
    pub fn new(title: String, slug: String, body: String) -> Self {
        Content {
            id: None,
            title,
            slug,
            body,
            status: STATUS_DRAFT.to_string(),
            tags: None,
            created_at: None,
            updated_at: None,
            synced_at: None,
        }
    }

    /// Updates the content status
    pub fn with_status(mut self, status: String) -> Self {
        self.status = status;
        self
    }

    /// Generates a slug from a title
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

/// Custom serializer for id field - converts None to 0 for create operations
// fn serialize_id<S>(id: &Option<i32>, serializer: S) -> Result<S::Ok, S::Error>
// where
//     S: serde::Serializer,
// {
//     match id {
//         Some(value) => serializer.serialize_i32(*value),
//         None => serializer.serialize_i32(0),
//     }
// }

/// Request structure for creating/updating content
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Props)]
pub struct ContentRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<i32>,
    pub title: String,
    pub slug: String,
    pub body: String,
    pub status: String,
    pub tags: Option<Vec<i32>>,
}

impl From<Content> for ContentRequest {
    fn from(content: Content) -> Self {
        ContentRequest {
            id: content.id,
            title: content.title,
            slug: content.slug,
            body: content.body,
            status: content.status,
            tags: content.tags,
        }
    }
}
