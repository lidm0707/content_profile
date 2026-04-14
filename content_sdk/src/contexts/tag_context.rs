use crate::models::{ContentTag, ContentTagRequest, Tag};
use crate::services::TagService;
use crate::utils::config::Config;
use dioxus::prelude::*;
/// Tag context for managing tag state across the app
#[derive(Clone, PartialEq, Props)]
pub struct TagContext {
    tag_service: Signal<TagService>,
    config: Signal<Option<Config>>,
}

impl TagContext {
    /// Creates a new TagContext
    pub fn new(config: Option<Config>) -> Self {
        let config_signal = Signal::new(config);
        TagContext {
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

    /// Fetches all tags
    pub async fn get_all_tags(&self) -> Result<Vec<Tag>, String> {
        let service = self.tag_service.cloned();
        service.get_all_tags().await
    }

    /// Fetches tags for a specific content item
    pub async fn get_tags_for_content(&self, content_id: i32) -> Result<Vec<Tag>, String> {
        let service = self.tag_service.cloned();
        service.get_tags_for_content(content_id).await
    }

    /// Adds a tag to a content item
    pub async fn add_tag_to_content(
        &mut self,
        request: ContentTagRequest,
    ) -> Result<ContentTag, String> {
        let mut service = self.tag_service.cloned();
        service.add_tag_to_content(request).await
    }

    /// Removes a tag from a content item
    pub async fn remove_tag_from_content(
        &mut self,
        content_id: i32,
        tag_id: i32,
    ) -> Result<(), String> {
        let mut service = self.tag_service.cloned();
        service.remove_tag_from_content(content_id, tag_id).await
    }

    /// Updates the tags for a content item (replaces all tags)
    pub async fn update_content_tags(
        &mut self,
        content_id: i32,
        tag_ids: Vec<i32>,
    ) -> Result<(), String> {
        let mut service = self.tag_service.cloned();
        service.update_content_tags(content_id, tag_ids).await
    }
}

impl Default for TagContext {
    fn default() -> Self {
        Self::new(None)
    }
}
