use crate::models::{Content, ContentRequest};
use crate::services::{LocalStorageService, SupabaseService};
use crate::utils::config::{AppMode, Config};
use dioxus::prelude::*;
use tracing::{debug, info, trace};

/// Content service that abstracts storage backend (local vs Supabase)
#[derive(Clone)]
pub struct ContentService {
    local_service: LocalStorageService,
    remote_service: SupabaseService,
    mode: AppMode,
}

impl ContentService {
    pub fn new(config: Option<Config>) -> Self {
        let mode = config.as_ref().map(|c| c.mode).unwrap_or(AppMode::Office);
        info!("ContentService initialized with mode: {:?}", mode);

        ContentService {
            local_service: LocalStorageService::new(),
            remote_service: SupabaseService::new(config),
            mode,
        }
    }

    /// Fetches all content items
    pub async fn get_all_content(&self) -> Result<Vec<Content>, String> {
        debug!("Getting all content (mode: {:?})", self.mode);
        match self.mode {
            AppMode::Office => {
                trace!("Using LocalStorageService for get_all_content");
                self.local_service.get_all_content()
            }
            AppMode::Supabase => {
                trace!("Using SupabaseService for get_all_content");
                self.remote_service.get_all_content().await
            }
        }
    }

    /// Fetches content by ID
    pub async fn get_content_by_id(&self, id: i32) -> Result<Option<Content>, String> {
        debug!("Getting content by ID {} (mode: {:?})", id, self.mode);
        match self.mode {
            AppMode::Office => {
                trace!("Using LocalStorageService for get_content_by_id");
                self.local_service.get_content_by_id(id)
            }
            AppMode::Supabase => {
                trace!("Using SupabaseService for get_content_by_id");
                self.remote_service.get_content_by_id(id).await
            }
        }
    }

    /// Fetches content by slug
    pub async fn get_content_by_slug(&self, slug: &str) -> Result<Option<Content>, String> {
        debug!("Getting content by slug '{}' (mode: {:?})", slug, self.mode);
        match self.mode {
            AppMode::Office => {
                trace!("Using LocalStorageService for get_content_by_slug");
                self.local_service.get_content_by_slug(slug)
            }
            AppMode::Supabase => {
                trace!("Using SupabaseService for get_content_by_slug");
                let slug = slug.to_string();
                self.remote_service.get_content_by_slug(&slug).await
            }
        }
    }

    /// Creates a new content item
    pub async fn create_content(
        &mut self,
        content_request: ContentRequest,
    ) -> Result<Content, String> {
        debug!(
            "Creating content '{}' (mode: {:?})",
            content_request.title, self.mode
        );
        match self.mode {
            AppMode::Office => {
                trace!("Using LocalStorageService for create_content");
                self.local_service.create_content(content_request)
            }
            AppMode::Supabase => {
                trace!("Using SupabaseService for create_content");
                self.remote_service.create_content(content_request).await
            }
        }
    }

    /// Updates an existing content item
    pub async fn update_content(
        &mut self,
        id: i32,
        content_request: ContentRequest,
    ) -> Result<Content, String> {
        debug!("Updating content ID {} (mode: {:?})", id, self.mode);
        match self.mode {
            AppMode::Office => {
                trace!("Using LocalStorageService for update_content");
                self.local_service.update_content(id, content_request)
            }
            AppMode::Supabase => {
                trace!("Using SupabaseService for update_content");
                self.remote_service
                    .update_content(id, content_request)
                    .await
            }
        }
    }

    /// Deletes a content item
    pub async fn delete_content(&mut self, id: i32) -> Result<(), String> {
        match self.mode {
            AppMode::Office => self.local_service.delete_content(id),
            AppMode::Supabase => self.remote_service.delete_content(id).await,
        }
    }

    /// Fetches content by status
    pub async fn get_content_by_status(&self, status: &str) -> Result<Vec<Content>, String> {
        match self.mode {
            AppMode::Office => self.local_service.get_content_by_status(status),
            AppMode::Supabase => self.remote_service.get_content_by_status(status).await,
        }
    }

    /// Gets local storage service directly
    pub fn local_service(&self) -> &LocalStorageService {
        &self.local_service
    }

    /// Gets remote storage service directly
    pub fn remote_service(&self) -> &SupabaseService {
        &self.remote_service
    }
}

impl Default for ContentService {
    fn default() -> Self {
        Self::new(None)
    }
}
