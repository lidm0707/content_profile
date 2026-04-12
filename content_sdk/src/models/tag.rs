//! Tag model
//!
//! This module defines the Tag struct and related types for representing tags.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Tag model
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Tag {
    /// Unique identifier
    pub id: Option<i32>,
    /// Tag name
    pub name: String,
    /// URL-friendly slug
    pub slug: String,
    /// Tag description
    pub description: Option<String>,
    /// Tag color (hex code)
    pub color: Option<String>,
    /// Number of content items with this tag
    pub count: Option<i32>,
    /// Creation timestamp
    pub created_at: Option<DateTime<Utc>>,
    /// Last update timestamp
    pub updated_at: Option<DateTime<Utc>>,
}

impl Tag {
    /// Creates a new tag with default values
    pub fn new(name: String, slug: String) -> Self {
        Tag {
            id: None,
            name,
            slug,
            description: None,
            color: None,
            count: None,
            created_at: None,
            updated_at: None,
        }
    }

    /// Sets the description
    pub fn with_description(mut self, description: String) -> Self {
        self.description = Some(description);
        self
    }

    /// Sets the color
    pub fn with_color(mut self, color: String) -> Self {
        self.color = Some(color);
        self
    }

    /// Generates a slug from a tag name
    pub fn generate_slug(name: &str) -> String {
        name.to_lowercase()
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

    /// Validates the tag name
    pub fn is_valid_name(name: &str) -> bool {
        !name.trim().is_empty() && name.len() <= 100
    }

    /// Validates the slug
    pub fn is_valid_slug(slug: &str) -> bool {
        !slug.trim().is_empty()
            && slug.len() <= 100
            && slug
                .chars()
                .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
    }

    /// Validates the color (hex format)
    pub fn is_valid_color(color: &str) -> bool {
        if color.is_empty() {
            return true;
        }

        let color = color.trim_start_matches('#');
        matches!(color.len(), 3 | 6) && color.chars().all(|c| c.is_ascii_hexdigit())
    }
}

/// Request structure for creating/updating tags
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TagRequest {
    /// Tag ID
    pub id: Option<i32>,
    /// Tag name
    pub name: String,
    /// URL-friendly slug
    pub slug: String,
    /// Tag description
    pub description: Option<String>,
    /// Tag color (hex code)
    pub color: Option<String>,
}

impl From<Tag> for TagRequest {
    fn from(tag: Tag) -> Self {
        TagRequest {
            id: tag.id,
            name: tag.name,
            slug: tag.slug,
            description: tag.description,
            color: tag.color,
        }
    }
}
