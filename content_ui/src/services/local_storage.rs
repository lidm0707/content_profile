use crate::models::{Content, ContentRequest};
use dioxus::prelude::*;

/// Local storage service for office mode
#[derive(Clone, PartialEq)]
pub struct LocalStorageService {
    contents: Signal<Vec<Content>>,
    next_id: Signal<usize>,
}

impl LocalStorageService {
    /// Creates a new local storage service instance
    pub fn new() -> Self {
        let mut service = LocalStorageService {
            contents: Signal::new(Vec::new()),
            next_id: Signal::new(1),
        };
        service.load_from_persistence();
        service
    }

    /// Fetches all content items
    pub fn get_all_content(&self) -> Result<Vec<Content>, String> {
        let contents = self.contents.read().cloned();
        Ok(contents.clone())
    }

    /// Fetches content by ID
    pub fn get_content_by_id(&self, id: i32) -> Result<Option<Content>, String> {
        let contents = self.contents.read().cloned();
        Ok(contents.iter().find(|c| c.id == Some(id)).cloned())
    }

    /// Fetches content by slug
    pub fn get_content_by_slug(&self, slug: &str) -> Result<Option<Content>, String> {
        let contents = self.contents.read().cloned();
        Ok(contents.iter().find(|c| c.slug == slug).cloned())
    }

    /// Creates a new content item
    pub fn create_content(&mut self, content_request: ContentRequest) -> Result<Content, String> {
        *self.next_id.write() += 1;
        let id = *self.next_id.read();
        let now = chrono::Utc::now();

        let mut content = Content::new(
            content_request.title,
            content_request.slug,
            content_request.body,
        );

        content.id = Some(
            id.try_into()
                .map_err(|e| format!("{e} try into i32 from usize failed"))?,
        );
        content = content.with_status(content_request.status);
        content.tags = content_request.tags;
        content.created_at = Some(now);
        content.updated_at = Some(now);

        let mut contents = self.contents.write();
        contents.push(content.clone());
        drop(contents);

        self.save_to_persistence();

        Ok(content)
    }

    /// Updates an existing content item
    pub fn update_content(
        &mut self,
        id: i32,
        content_request: ContentRequest,
    ) -> Result<Content, String> {
        let mut contents = self.contents.write();

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
    pub fn delete_content(&mut self, id: i32) -> Result<(), String> {
        let mut contents = self.contents.write();
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
        let contents = self.contents.read();
        Ok(contents
            .iter()
            .filter(|c| c.status == status)
            .cloned()
            .collect())
    }

    /// Syncs local content with remote content
    pub fn sync_content(&mut self, remote_contents: Vec<Content>) -> Result<Vec<Content>, String> {
        let mut contents = self.contents.write();

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
        let contents = self.contents.read();
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

    /// Marks content as synced by updating with remote ID and timestamp
    pub fn mark_as_synced(&mut self, local_id: i32, remote_id: i32) -> Result<Content, String> {
        let mut contents = self.contents.write();

        let index = contents
            .iter()
            .position(|c| c.id == Some(local_id))
            .ok_or_else(|| format!("Content with local id {} not found", local_id))?;

        let now = chrono::Utc::now();
        contents[index].id = Some(remote_id);
        contents[index].synced_at = Some(now);

        let content = contents[index].clone();
        drop(contents);

        self.save_to_persistence();

        Ok(content)
    }

    /// Clears all local content
    pub fn clear_all(&mut self) -> Result<(), String> {
        let mut contents = self.contents.write();
        contents.clear();
        drop(contents);
        self.save_to_persistence();
        Ok(())
    }

    /// Saves data to persistent storage (localStorage on web)
    fn save_to_persistence(&mut self) {
        if let Some(window) = web_sys::window()
            && let Ok(Some(storage)) = window.local_storage()
        {
            let contents = self.contents.read();
            if let Ok(json) = serde_json::to_string(&*contents) {
                let _ = storage.set_item("cms_content", &json);
            }
            let next_id = self.next_id;
            let _ = storage.set_item("cms_next_id", &next_id.to_string());
        }
    }

    /// Loads data from persistent storage (localStorage on web)
    fn load_from_persistence(&mut self) {
        if let Some(window) = web_sys::window()
            && let Ok(Some(storage)) = window.local_storage()
            && let Ok(Some(json)) = storage.get_item("cms_content")
            && let Ok(loaded) = serde_json::from_str::<Vec<Content>>(&json)
        {
            let mut contents = self.contents.write();
            *contents = loaded;
            if let Some(max_id) = contents.iter().filter_map(|c| c.id).max() {
                *self.next_id.write() = (max_id + 1) as usize;
                // maybe something wrong
            }
        }
    }
}

impl Default for LocalStorageService {
    fn default() -> Self {
        Self::new()
    }
}
