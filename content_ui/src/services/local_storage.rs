use crate::models::{Content, ContentRequest};
use dioxus::prelude::*;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, RwLock};

/// Local storage service for office mode
#[derive(Clone)]
pub struct LocalStorageService {
    contents: Arc<RwLock<Vec<Content>>>,
    next_id: Arc<AtomicUsize>,
}

impl LocalStorageService {
    /// Creates a new local storage service instance
    pub fn new() -> Self {
        let service = LocalStorageService {
            contents: Arc::new(RwLock::new(Vec::new())),
            next_id: Arc::new(AtomicUsize::new(1)),
        };
        service.load_from_persistence();
        service
    }

    /// Fetches all content items
    pub fn get_all_content(&self) -> Result<Vec<Content>, String> {
        let contents = self.contents.read().map_err(|e| e.to_string())?;
        Ok(contents.clone())
    }

    /// Fetches content by ID
    pub fn get_content_by_id(&self, id: i32) -> Result<Option<Content>, String> {
        let contents = self.contents.read().map_err(|e| e.to_string())?;
        Ok(contents.iter().find(|c| c.id == Some(id)).cloned())
    }

    /// Fetches content by slug
    pub fn get_content_by_slug(&self, slug: &str) -> Result<Option<Content>, String> {
        let contents = self.contents.read().map_err(|e| e.to_string())?;
        Ok(contents.iter().find(|c| c.slug == slug).cloned())
    }

    /// Creates a new content item
    pub fn create_content(&self, content_request: ContentRequest) -> Result<Content, String> {
        let id = self.next_id.fetch_add(1, Ordering::SeqCst) as i32;

        let now = chrono::Utc::now();
        let content = Content {
            id: Some(id),
            title: content_request.title,
            slug: content_request.slug,
            body: content_request.body,
            status: content_request.status,
            tags: content_request.tags,
            created_at: Some(now),
            updated_at: Some(now),
            synced_at: None,
        };

        let mut contents = self.contents.write().map_err(|e| e.to_string())?;
        contents.push(content.clone());
        drop(contents);

        self.save_to_persistence();

        Ok(content)
    }

    /// Updates an existing content item
    pub fn update_content(
        &self,
        id: i32,
        content_request: ContentRequest,
    ) -> Result<Content, String> {
        let mut contents = self.contents.write().map_err(|e| e.to_string())?;

        let index = contents
            .iter()
            .position(|c| c.id == Some(id))
            .ok_or_else(|| format!("Content with id {} not found", id))?;

        let now = chrono::Utc::now();
        contents[index] = Content {
            id: Some(id),
            title: content_request.title,
            slug: content_request.slug,
            body: content_request.body,
            status: content_request.status,
            tags: content_request.tags,
            created_at: contents[index].created_at,
            updated_at: Some(now),
            synced_at: contents[index].synced_at,
        };

        let content = contents[index].clone();
        drop(contents);

        self.save_to_persistence();

        Ok(content)
    }

    /// Deletes a content item
    pub fn delete_content(&self, id: i32) -> Result<(), String> {
        let mut contents = self.contents.write().map_err(|e| e.to_string())?;
        let initial_len = contents.len();
        contents.retain(|c| c.id != Some(id));

        if contents.len() == initial_len {
            return Err(format!("Content with id {} not found", id));
        }

        drop(contents);
        self.save_to_persistence();

        Ok(())
    }

    /// Fetches content by status
    pub fn get_content_by_status(&self, status: &str) -> Result<Vec<Content>, String> {
        let contents = self.contents.read().map_err(|e| e.to_string())?;
        Ok(contents
            .iter()
            .filter(|c| c.status == status)
            .cloned()
            .collect())
    }

    /// Syncs local content with remote content
    pub fn sync_content(&self, remote_contents: Vec<Content>) -> Result<Vec<Content>, String> {
        let mut contents = self.contents.write().map_err(|e| e.to_string())?;

        for remote in remote_contents {
            if let Some(local_index) = contents.iter().position(|c| c.id == remote.id) {
                if contents[local_index].updated_at < remote.updated_at {
                    contents[local_index] = remote;
                }
            } else {
                contents.push(remote);
            }
        }

        let synced = contents.clone();
        drop(contents);

        self.save_to_persistence();

        Ok(synced)
    }

    /// Gets unsynced content (content that doesn't exist remotely)
    pub fn get_unsynced_content(&self, remote_ids: &[i32]) -> Result<Vec<Content>, String> {
        let contents = self.contents.read().map_err(|e| e.to_string())?;
        Ok(contents
            .iter()
            .filter(|c| {
                if let Some(id) = c.id {
                    !remote_ids.contains(&id)
                } else {
                    true
                }
            })
            .cloned()
            .collect())
    }

    /// Clears all local content
    pub fn clear_all(&self) -> Result<(), String> {
        let mut contents = self.contents.write().map_err(|e| e.to_string())?;
        contents.clear();
        drop(contents);
        self.save_to_persistence();
        Ok(())
    }

    /// Saves data to persistent storage (localStorage on web)
    fn save_to_persistence(&self) {
        if let Some(window) = web_sys::window() {
            if let Ok(Some(storage)) = window.local_storage() {
                let contents = self.contents.read().unwrap();
                if let Ok(json) = serde_json::to_string(&*contents) {
                    let _ = storage.set_item("cms_content", &json);
                }
                let next_id = self.next_id.load(Ordering::SeqCst);
                let _ = storage.set_item("cms_next_id", &next_id.to_string());
            }
        }
    }

    /// Loads data from persistent storage (localStorage on web)
    fn load_from_persistence(&self) {
        if let Some(window) = web_sys::window() {
            if let Ok(Some(storage)) = window.local_storage() {
                if let Ok(Some(json)) = storage.get_item("cms_content") {
                    if let Ok(loaded) = serde_json::from_str::<Vec<Content>>(&json) {
                        let mut contents = self.contents.write().unwrap();
                        *contents = loaded;
                        if let Some(max_id) = contents.iter().filter_map(|c| c.id).max() {
                            self.next_id.store(max_id as usize + 1, Ordering::SeqCst);
                        }
                    }
                }
            }
        }
    }
}

impl Default for LocalStorageService {
    fn default() -> Self {
        Self::new()
    }
}
