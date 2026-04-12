use dioxus::prelude::Props;
use dioxus::signals::{ReadableExt, Signal};

use crate::models::{Content, ContentRequest};
use crate::services::ContentService;
use crate::utils::config::AppMode;
use dioxus::prelude::*;

/// Content context for managing content state across the app
#[derive(Clone, Props, PartialEq)]
pub struct ContentContext {
    content_service: Signal<ContentService>,
}

impl ContentContext {
    /// Creates a new ContentContext
    pub fn new() -> Self {
        ContentContext {
            content_service: Signal::new(ContentService::new()),
        }
    }

    /// Gets the content service
    pub fn content_service(&self) -> ContentService {
        self.content_service.cloned()
    }

    /// Gets the current mode
    pub fn mode(&self) -> AppMode {
        self.content_service.read().mode()
    }

    /// Checks if in office mode
    pub fn is_office_mode(&self) -> bool {
        self.content_service.read().is_office_mode()
    }

    /// Checks if in supabase mode
    pub fn is_supabase_mode(&self) -> bool {
        self.content_service.read().is_supabase_mode()
    }

    /// Fetches all content items
    pub async fn get_all_content(&self) -> Result<Vec<Content>, String> {
        self.content_service.read().get_all_content().await
    }

    /// Fetches a single content item by ID
    pub async fn get_content_by_id(&self, id: i32) -> Result<Option<Content>, String> {
        self.content_service.read().get_content_by_id(id).await
    }

    /// Fetches a single content item by slug
    pub async fn get_content_by_slug(&self, slug: &str) -> Result<Option<Content>, String> {
        self.content_service.read().get_content_by_slug(slug).await
    }

    /// Fetches content items by status
    pub async fn get_content_by_status(&self, status: &str) -> Result<Vec<Content>, String> {
        self.content_service
            .read()
            .get_content_by_status(status)
            .await
    }

    /// Creates a new content item
    pub async fn create_content(
        &mut self,
        content_request: ContentRequest,
    ) -> Result<Content, String> {
        Ok(self
            .content_service
            .write()
            .create_content(content_request)
            .await?)
    }

    /// Updates an existing content item
    pub async fn update_content(
        &mut self,
        id: i32,
        content_request: ContentRequest,
    ) -> Result<Content, String> {
        self.content_service
            .write()
            .update_content(id, content_request)
            .await
    }

    /// Deletes a content item
    pub async fn delete_content(&mut self, id: i32) -> Result<(), String> {
        Ok(self.content_service.write().delete_content(id).await?)
    }

    /// Gets the local storage service directly
    pub fn local_service(&self) -> crate::services::LocalStorageService {
        self.content_service.cloned().local_service().clone()
    }

    /// Gets the remote storage service directly
    pub fn remote_service(&self) -> crate::services::SupabaseService {
        self.content_service.cloned().remote_service().clone()
    }
}

impl Default for ContentContext {
    fn default() -> Self {
        Self::new()
    }
}
