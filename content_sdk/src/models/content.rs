//! Content model
//!
//! This module defines the Content struct and related types for representing content items.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Deserializer, Serialize};

/// Content status constants
pub const STATUS_DRAFT: &str = "draft";
pub const STATUS_PUBLISHED: &str = "published";
pub const STATUS_ARCHIVED: &str = "archived";

/// Content status enum
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ContentStatus {
    Draft,
    Published,
    Archived,
}

impl ContentStatus {
    /// Convert status to string
    pub fn as_str(&self) -> &'static str {
        match self {
            ContentStatus::Draft => STATUS_DRAFT,
            ContentStatus::Published => STATUS_PUBLISHED,
            ContentStatus::Archived => STATUS_ARCHIVED,
        }
    }

    /// Create status from string
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            STATUS_DRAFT => Some(ContentStatus::Draft),
            STATUS_PUBLISHED => Some(ContentStatus::Published),
            STATUS_ARCHIVED => Some(ContentStatus::Archived),
            _ => None,
        }
    }
}

impl std::fmt::Display for ContentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

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

/// Content model
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Content {
    /// Unique identifier
    pub id: Option<i32>,
    /// Content title
    pub title: String,
    /// URL-friendly slug
    pub slug: String,
    /// Content body
    pub body: String,
    /// Content status
    #[serde(rename = "status")]
    pub status: String,
    /// Associated tag IDs
    pub tags: Option<Vec<i32>>,
    /// Featured image URL
    pub featured_image: Option<String>,
    /// Excerpt/summary
    pub excerpt: Option<String>,
    /// Author name
    pub author: Option<String>,
    /// Content type (blog, page, article, etc.)
    pub content_type: Option<String>,
    /// Creation timestamp
    pub created_at: Option<DateTime<Utc>>,
    /// Last update timestamp
    pub updated_at: Option<DateTime<Utc>>,
    /// Published timestamp
    pub published_at: Option<DateTime<Utc>>,
    /// Last sync timestamp
    #[serde(default, deserialize_with = "deserialize_optional_datetime")]
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
            featured_image: None,
            excerpt: None,
            author: None,
            content_type: None,
            created_at: None,
            updated_at: None,
            published_at: None,
            synced_at: None,
        }
    }

    /// Updates the content status
    pub fn with_status(mut self, status: ContentStatus) -> Self {
        self.status = status.as_str().to_string();
        self
    }

    /// Sets the author
    pub fn with_author(mut self, author: String) -> Self {
        self.author = Some(author);
        self
    }

    /// Sets the featured image
    pub fn with_featured_image(mut self, image_url: String) -> Self {
        self.featured_image = Some(image_url);
        self
    }

    /// Sets the excerpt
    pub fn with_excerpt(mut self, excerpt: String) -> Self {
        self.excerpt = Some(excerpt);
        self
    }

    /// Sets the content type
    pub fn with_content_type(mut self, content_type: String) -> Self {
        self.content_type = Some(content_type);
        self
    }

    /// Gets the content status as an enum
    pub fn get_status(&self) -> Option<ContentStatus> {
        ContentStatus::from_str(&self.status)
    }

    /// Checks if the content is published
    pub fn is_published(&self) -> bool {
        self.get_status() == Some(ContentStatus::Published)
    }

    /// Checks if the content is a draft
    pub fn is_draft(&self) -> bool {
        self.get_status() == Some(ContentStatus::Draft)
    }

    /// Checks if the content is archived
    pub fn is_archived(&self) -> bool {
        self.get_status() == Some(ContentStatus::Archived)
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

    /// Generates an excerpt from the body if none exists
    pub fn get_excerpt(&self, max_length: usize) -> String {
        if let Some(excerpt) = &self.excerpt {
            if !excerpt.is_empty() {
                return excerpt.clone();
            }
        }

        // Strip markdown and generate excerpt
        let plain_text = self
            .body
            .replace('#', "")
            .replace('*', "")
            .replace('`', "")
            .replace('[', "")
            .replace(']', "")
            .replace('(', "")
            .replace(')', "")
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ");

        if plain_text.len() <= max_length {
            plain_text
        } else {
            let mut excerpt = plain_text.chars().take(max_length).collect::<String>();
            while !excerpt.ends_with(' ') && !excerpt.is_empty() {
                excerpt.pop();
            }
            if !excerpt.is_empty() {
                excerpt.push_str("...");
            }
            excerpt
        }
    }
}

/// Request structure for creating/updating content
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ContentRequest {
    /// Content ID (0 for new content)
    pub id: Option<i32>,
    /// Content title
    pub title: String,
    /// URL-friendly slug
    pub slug: String,
    /// Content body
    pub body: String,
    /// Content status
    pub status: String,
    /// Associated tag IDs
    pub tags: Option<Vec<i32>>,
    /// Featured image URL
    pub featured_image: Option<String>,
    /// Excerpt/summary
    pub excerpt: Option<String>,
    /// Author name
    pub author: Option<String>,
    /// Content type
    pub content_type: Option<String>,
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
            featured_image: content.featured_image,
            excerpt: content.excerpt,
            author: content.author,
            content_type: content.content_type,
        }
    }
}
