use crate::models::{Content, ContentRequest};
use crate::services::ContentService;
use crate::utils::config::Config;
use dioxus::prelude::*;

/// Content context for managing content state across the app
#[derive(Clone, Props, PartialEq)]
pub struct ContentContext {
    content_service: Signal<ContentService>,
}

impl ContentContext {
    /// Creates a new ContentContext
    pub fn new(config: Option<Config>) -> Self {
        ContentContext {
            content_service: Signal::new(ContentService::new(config)),
        }
    }

    /// Gets the content service
    pub fn content_service(&self) -> ContentService {
        self.content_service.cloned()
    }

    /// Fetches all content items
    pub async fn get_all_content(&self) -> Result<Vec<Content>, String> {
        self.content_service.cloned().get_all_content().await
    }

    /// Fetches a single content item by ID
    pub async fn get_content_by_id(&self, id: i32) -> Result<Option<Content>, String> {
        self.content_service.cloned().get_content_by_id(id).await
    }

    /// Fetches a single content item by slug
    pub async fn get_content_by_slug(&self, slug: &str) -> Result<Option<Content>, String> {
        self.content_service
            .cloned()
            .get_content_by_slug(slug)
            .await
    }

    /// Fetches content items by status
    pub async fn get_content_by_status(&self, status: &str) -> Result<Vec<Content>, String> {
        self.content_service
            .cloned()
            .get_content_by_status(status)
            .await
    }

    /// Creates a new content item
    pub async fn create_content(
        &mut self,
        content_request: ContentRequest,
    ) -> Result<Content, String> {
        self.content_service
            .cloned()
            .create_content(content_request)
            .await
    }

    /// Updates an existing content item
    pub async fn update_content(
        &mut self,
        id: i32,
        content_request: ContentRequest,
    ) -> Result<Content, String> {
        self.content_service
            .cloned()
            .update_content(id, content_request)
            .await
    }

    /// Deletes a content item
    pub async fn delete_content(&mut self, id: i32) -> Result<(), String> {
        self.content_service.cloned().delete_content(id).await
    }
}

impl Default for ContentContext {
    fn default() -> Self {
        Self::new(None)
    }
}
