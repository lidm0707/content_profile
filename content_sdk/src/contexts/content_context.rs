use crate::models::{Content, ContentRequest};
use crate::pagination::{PaginatedResponse, PaginationParams};
use crate::services::ContentService;
use crate::utils::config::Config;
use dioxus::prelude::*;

/// Helper struct to hold authentication state that can be updated
#[derive(Clone, PartialEq)]
pub struct AuthState {
    pub jwt_token: Option<String>,
}

/// Content context for managing content state across the app
#[derive(Clone, PartialEq, Props)]
pub struct ContentContext {
    content_service: Signal<ContentService>,
    config: Signal<Option<Config>>,
}

impl ContentContext {
    /// Creates a new ContentContext
    pub fn new(config: Option<Config>) -> Self {
        let config_signal = Signal::new(config);
        ContentContext {
            content_service: Signal::new(ContentService::new(config_signal.read().clone())),
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
        let updated_service = ContentService::new((*config).clone());
        *self.content_service.write() = updated_service;
    }

    /// Gets the content service
    pub fn content_service(&self) -> ContentService {
        self.content_service.cloned()
    }

    /// Fetches all content items
    pub async fn get_all_content(&self) -> Result<Vec<Content>, String> {
        self.content_service
            .cloned()
            .get_paginated_content(
                &[],
                PaginationParams::default().page,
                PaginationParams::default().page_size,
            )
            .await
            .map(|r| r.data)
    }

    /// Fetches paginated content items
    pub async fn get_paginated_content(
        &self,
        filters: &[(&str, &str)],
        page: u32,
        page_size: u32,
    ) -> Result<PaginatedResponse<Content>, String> {
        self.content_service
            .cloned()
            .get_paginated_content(filters, page, page_size)
            .await
    }

    /// Counts total content items
    pub async fn count_content(&self, filters: &[(&str, &str)]) -> Result<u32, String> {
        self.content_service.cloned().count_content(filters).await
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

    /// Fetches content items by multiple IDs
    pub async fn get_content_by_ids(&self, ids: &[i32]) -> Result<Vec<Content>, String> {
        self.content_service.cloned().get_content_by_ids(ids).await
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
