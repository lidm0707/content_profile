use crate::models::{ContentTag, ContentTagRequest, Tag};
use crate::services::TagService;
use crate::utils::config::Config;
use dioxus::prelude::*;

/// Content tags context for managing content-tag relationships
///
/// This context provides a reactive interface for managing the junction table
/// relationships between content items and tags. It handles CRUD operations
/// on the content_tags table.
#[derive(Clone, PartialEq, Props)]
pub struct ContentTagsContext {
    tag_service: Signal<TagService>,
    config: Signal<Option<Config>>,
}

impl ContentTagsContext {
    /// Creates a new ContentTagsContext
    pub fn new(config: Option<Config>) -> Self {
        let config_signal = Signal::new(config);
        ContentTagsContext {
            tag_service: Signal::new(TagService::new(config_signal.read().clone())),
            config: config_signal,
        }
    }

    /// Updates the JWT token and recreates the service with new config
    pub fn update_jwt_token(&mut self, jwt_token: Option<String>) {
        let mut config = self.config.write();
        if let Some(ref mut cfg) = *config {
            cfg.jwt_token = jwt_token;
        } else {
            *config = Some(Config::new("office", "", "", jwt_token));
        }

        // Recreate the service with updated config
        let updated_service = TagService::new((*config).clone());
        *self.tag_service.write() = updated_service;
    }

    /// Gets the tag service
    pub fn tag_service(&self) -> TagService {
        self.tag_service.cloned()
    }

    /// Fetches tags for a specific content item
    ///
    /// This method queries the content_tags junction table and returns
    /// the actual Tag objects assigned to the given content_id.
    pub async fn get_tags_for_content(&self, content_id: i32) -> Result<Vec<Tag>, String> {
        let service = self.tag_service.cloned();
        service.get_tags_for_content(content_id).await
    }

    /// Fetches content-tag junction records for a specific content item
    ///
    /// This method queries the content_tags junction table and returns
    /// the ContentTag records (junction table) for the given content_id.
    pub async fn get_content_tags_for_content(
        &self,
        content_id: i32,
    ) -> Result<Vec<ContentTag>, String> {
        let service = self.tag_service.cloned();
        service.get_content_tags_for_content(content_id).await
    }

    /// Adds a tag to a content item
    ///
    /// Creates a new record in the content_tags junction table.
    pub async fn add_tag_to_content(
        &mut self,
        request: ContentTagRequest,
    ) -> Result<ContentTag, String> {
        let mut service = self.tag_service.cloned();
        service.add_tag_to_content(request).await
    }

    /// Removes a tag from a content item
    ///
    /// Deletes the corresponding record in the content_tags junction table.
    pub async fn remove_tag_from_content(
        &mut self,
        content_id: i32,
        tag_id: i32,
    ) -> Result<(), String> {
        let mut service = self.tag_service.cloned();
        service.remove_tag_from_content(content_id, tag_id).await
    }

    /// Updates the tags for a content item (replaces all tags)
    ///
    /// This is a full replacement operation that ensures the content has
    /// exactly the tags specified in the tag_ids vector.
    pub async fn update_content_tags(
        &mut self,
        content_id: i32,
        tag_ids: Vec<i32>,
    ) -> Result<(), String> {
        let mut service = self.tag_service.cloned();
        service.update_content_tags(content_id, tag_ids).await
    }

    /// Fetches content-tag junction records for a specific tag
    ///
    /// This method queries the content_tags junction table and returns
    /// the ContentTag records (junction table) for the given tag_id.
    pub async fn get_content_tags_for_tag(&self, tag_id: i32) -> Result<Vec<ContentTag>, String> {
        let service = self.tag_service.cloned();
        service.get_content_tags_for_tag(tag_id).await
    }

    /// Fetches content IDs for a specific tag
    ///
    /// This method queries the content_tags junction table and returns
    /// the content_id values for the given tag_id.
    pub async fn get_content_ids_for_tag(&self, tag_id: i32) -> Result<Vec<i32>, String> {
        let service = self.tag_service.cloned();
        service.get_content_ids_for_tag(tag_id).await
    }
}

impl Default for ContentTagsContext {
    fn default() -> Self {
        Self::new(None)
    }
}
