use crate::models::{ContentTag, ContentTagRequest, Tag};
use crate::services::TagService;
use std::sync::Arc;

/// Tag context for managing tag state across the app
#[derive(Clone)]
pub struct TagContext {
    tag_service: Arc<TagService>,
}

impl TagContext {
    /// Creates a new TagContext
    pub fn new() -> Self {
        TagContext {
            tag_service: Arc::new(TagService::new()),
        }
    }

    /// Gets the tag service
    pub fn tag_service(&self) -> &TagService {
        &self.tag_service
    }

    /// Fetches all tags
    pub async fn get_all_tags(&self) -> Result<Vec<Tag>, String> {
        self.tag_service.get_all_tags().await
    }

    /// Fetches tags for a specific content item
    pub async fn get_tags_for_content(&self, content_id: i32) -> Result<Vec<Tag>, String> {
        self.tag_service.get_tags_for_content(content_id).await
    }

    /// Adds a tag to a content item
    pub async fn add_tag_to_content(
        &self,
        request: ContentTagRequest,
    ) -> Result<ContentTag, String> {
        self.tag_service.add_tag_to_content(request).await
    }

    /// Removes a tag from a content item
    pub async fn remove_tag_from_content(
        &self,
        content_id: i32,
        tag_id: i32,
    ) -> Result<(), String> {
        self.tag_service
            .remove_tag_from_content(content_id, tag_id)
            .await
    }

    /// Updates the tags for a content item (replaces all tags)
    pub async fn update_content_tags(
        &self,
        content_id: i32,
        tag_ids: Vec<i32>,
    ) -> Result<(), String> {
        self.tag_service
            .update_content_tags(content_id, tag_ids)
            .await
    }
}

impl Default for TagContext {
    fn default() -> Self {
        Self::new()
    }
}
