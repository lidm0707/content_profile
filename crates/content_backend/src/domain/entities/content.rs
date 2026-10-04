use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

pub const STATUS_DRAFT: &str = "draft";
pub const STATUS_PUBLISHED: &str = "published";

use crate::domain::value_objects::generate_slug;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Content {
    pub id: Option<i32>,
    pub title: String,
    pub slug: String,
    pub body: String,
    pub status: String,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

impl Content {
    pub fn new(title: String, slug: String, body: String) -> Self {
        let slug = if slug.is_empty() {
            generate_slug(&title)
        } else {
            slug
        };
        Self {
            id: None,
            title,
            slug,
            body,
            status: STATUS_DRAFT.to_string(),
            created_at: None,
            updated_at: None,
        }
    }

    pub fn is_published(&self) -> bool {
        self.status == STATUS_PUBLISHED
    }

    pub fn touch(&mut self) {
        self.updated_at = Some(Utc::now());
    }
}
