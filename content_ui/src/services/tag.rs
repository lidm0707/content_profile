use crate::models::{ContentTag, ContentTagRequest, Tag};
use dioxus::prelude::*;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, RwLock};

#[derive(Clone)]
pub struct TagService {
    local_service: LocalTagService,
    remote_service: SupabaseTagService,
}

impl TagService {
    pub fn new() -> Self {
        TagService {
            local_service: LocalTagService::new(),
            remote_service: SupabaseTagService::new(),
        }
    }

    pub async fn get_all_tags(&self) -> Result<Vec<Tag>, String> {
        #[cfg(not(target_arch = "wasm32"))]
        {
            return self.remote_service.get_all_tags().await;
        }
        #[cfg(target_arch = "wasm32")]
        {
            return self.local_service.get_all_tags();
        }
    }

    pub async fn get_tags_for_content(&self, content_id: i32) -> Result<Vec<Tag>, String> {
        #[cfg(not(target_arch = "wasm32"))]
        {
            return self.remote_service.get_tags_for_content(content_id).await;
        }
        #[cfg(target_arch = "wasm32")]
        {
            return self.local_service.get_tags_for_content(content_id);
        }
    }

    pub async fn add_tag_to_content(
        &self,
        request: ContentTagRequest,
    ) -> Result<ContentTag, String> {
        #[cfg(not(target_arch = "wasm32"))]
        {
            return self.remote_service.add_tag_to_content(request).await;
        }
        #[cfg(target_arch = "wasm32")]
        {
            return self.local_service.add_tag_to_content(request);
        }
    }

    pub async fn remove_tag_from_content(
        &self,
        content_id: i32,
        tag_id: i32,
    ) -> Result<(), String> {
        #[cfg(not(target_arch = "wasm32"))]
        {
            return self
                .remote_service
                .remove_tag_from_content(content_id, tag_id)
                .await;
        }
        #[cfg(target_arch = "wasm32")]
        {
            return self
                .local_service
                .remove_tag_from_content(content_id, tag_id);
        }
    }

    pub async fn update_content_tags(
        &self,
        content_id: i32,
        tag_ids: Vec<i32>,
    ) -> Result<(), String> {
        let current_tags = self.get_tags_for_content(content_id).await?;
        let current_tag_ids: Vec<i32> = current_tags.iter().map(|t| t.id.unwrap()).collect();

        for tag_id in &tag_ids {
            if !current_tag_ids.contains(tag_id) {
                self.add_tag_to_content(ContentTagRequest {
                    content_id,
                    tag_id: *tag_id,
                })
                .await?;
            }
        }

        for current_tag_id in current_tag_ids {
            if !tag_ids.contains(&current_tag_id) {
                self.remove_tag_from_content(content_id, current_tag_id)
                    .await?;
            }
        }

        Ok(())
    }
}

impl Default for TagService {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone)]
pub struct LocalTagService {
    tags: Arc<RwLock<Vec<Tag>>>,
    content_tags: Arc<RwLock<Vec<ContentTag>>>,
    next_tag_id: Arc<AtomicUsize>,
    next_content_tag_id: Arc<AtomicUsize>,
}

impl LocalTagService {
    const TAGS_KEY: &'static str = "cms_tags";
    const CONTENT_TAGS_KEY: &'static str = "cms_content_tags";
    const NEXT_TAG_ID_KEY: &'static str = "cms_next_tag_id";
    const NEXT_CONTENT_TAG_ID_KEY: &'static str = "cms_next_content_tag_id";

    pub fn new() -> Self {
        let service = LocalTagService {
            tags: Arc::new(RwLock::new(Vec::new())),
            content_tags: Arc::new(RwLock::new(Vec::new())),
            next_tag_id: Arc::new(AtomicUsize::new(1)),
            next_content_tag_id: Arc::new(AtomicUsize::new(1)),
        };
        service.load_from_persistence();
        service.initialize_default_tags();
        service
    }

    pub fn get_all_tags(&self) -> Result<Vec<Tag>, String> {
        let tags = self.tags.read().map_err(|e| e.to_string())?;
        Ok(tags.clone())
    }

    pub fn get_tags_for_content(&self, content_id: i32) -> Result<Vec<Tag>, String> {
        let content_tags = self.content_tags.read().map_err(|e| e.to_string())?;
        let tags = self.tags.read().map_err(|e| e.to_string())?;

        let tag_ids: Vec<i32> = content_tags
            .iter()
            .filter(|ct| ct.content_id == content_id)
            .map(|ct| ct.tag_id)
            .collect();

        Ok(tag_ids
            .iter()
            .filter_map(|tag_id| tags.iter().find(|t| t.id == Some(*tag_id)).cloned())
            .collect())
    }

    pub fn add_tag_to_content(&self, request: ContentTagRequest) -> Result<ContentTag, String> {
        let content_tags = self.content_tags.read().map_err(|e| e.to_string())?;

        if content_tags
            .iter()
            .any(|ct| ct.content_id == request.content_id && ct.tag_id == request.tag_id)
        {
            return Err("Tag already added to content".to_string());
        }

        drop(content_tags);

        let id = self.next_content_tag_id.fetch_add(1, Ordering::SeqCst) as i32;
        let now = chrono::Utc::now();
        let content_tag = ContentTag {
            id: Some(id),
            content_id: request.content_id,
            tag_id: request.tag_id,
            created_at: Some(now),
        };

        let mut content_tags = self.content_tags.write().map_err(|e| e.to_string())?;
        content_tags.push(content_tag.clone());
        drop(content_tags);

        self.save_to_persistence();

        Ok(content_tag)
    }

    pub fn remove_tag_from_content(&self, content_id: i32, tag_id: i32) -> Result<(), String> {
        let mut content_tags = self.content_tags.write().map_err(|e| e.to_string())?;
        let initial_len = content_tags.len();
        content_tags.retain(|ct| ct.content_id != content_id || ct.tag_id != tag_id);

        if content_tags.len() == initial_len {
            return Err("ContentTag relationship not found".to_string());
        }

        drop(content_tags);

        self.save_to_persistence();

        Ok(())
    }

    fn save_to_persistence(&self) {
        if let Some(window) = web_sys::window() {
            if let Ok(Some(storage)) = window.local_storage() {
                let tags = self.tags.read().unwrap();
                if let Ok(json) = serde_json::to_string(&*tags) {
                    let _ = storage.set_item(Self::TAGS_KEY, &json);
                }

                let content_tags = self.content_tags.read().unwrap();
                if let Ok(json) = serde_json::to_string(&*content_tags) {
                    let _ = storage.set_item(Self::CONTENT_TAGS_KEY, &json);
                }

                let next_tag_id = self.next_tag_id.load(Ordering::SeqCst);
                let _ = storage.set_item(Self::NEXT_TAG_ID_KEY, &next_tag_id.to_string());

                let next_content_tag_id = self.next_content_tag_id.load(Ordering::SeqCst);
                let _ = storage.set_item(
                    Self::NEXT_CONTENT_TAG_ID_KEY,
                    &next_content_tag_id.to_string(),
                );
            }
        }
    }

    fn load_from_persistence(&self) {
        if let Some(window) = web_sys::window() {
            if let Ok(Some(storage)) = window.local_storage() {
                if let Ok(Some(json)) = storage.get_item(Self::TAGS_KEY) {
                    if let Ok(loaded) = serde_json::from_str::<Vec<Tag>>(&json) {
                        let mut tags = self.tags.write().unwrap();
                        *tags = loaded;
                        if let Some(max_id) = tags.iter().filter_map(|t| t.id).max() {
                            self.next_tag_id
                                .store(max_id as usize + 1, Ordering::SeqCst);
                        }
                    }
                }

                if let Ok(Some(json)) = storage.get_item(Self::CONTENT_TAGS_KEY) {
                    if let Ok(loaded) = serde_json::from_str::<Vec<ContentTag>>(&json) {
                        let mut content_tags = self.content_tags.write().unwrap();
                        *content_tags = loaded;
                        if let Some(max_id) = content_tags.iter().filter_map(|ct| ct.id).max() {
                            self.next_content_tag_id
                                .store(max_id as usize + 1, Ordering::SeqCst);
                        }
                    }
                }
            }
        }
    }

    fn initialize_default_tags(&self) {
        let tags = self.tags.read().unwrap();
        if tags.is_empty() {
            drop(tags);
            let now = chrono::Utc::now();
            let default_tags = vec![
                Tag {
                    id: Some(1),
                    name: "Technology".to_string(),
                    slug: "technology".to_string(),
                    parent_id: None,
                    created_at: Some(now),
                    updated_at: Some(now),
                    synced_at: None,
                },
                Tag {
                    id: Some(2),
                    name: "Business".to_string(),
                    slug: "business".to_string(),
                    parent_id: None,
                    created_at: Some(now),
                    updated_at: Some(now),
                    synced_at: None,
                },
                Tag {
                    id: Some(3),
                    name: "Design".to_string(),
                    slug: "design".to_string(),
                    parent_id: None,
                    created_at: Some(now),
                    updated_at: Some(now),
                    synced_at: None,
                },
                Tag {
                    id: Some(4),
                    name: "Marketing".to_string(),
                    slug: "marketing".to_string(),
                    parent_id: None,
                    created_at: Some(now),
                    updated_at: Some(now),
                    synced_at: None,
                },
                Tag {
                    id: Some(5),
                    name: "Programming".to_string(),
                    slug: "programming".to_string(),
                    parent_id: Some(1),
                    created_at: Some(now),
                    updated_at: Some(now),
                    synced_at: None,
                },
                Tag {
                    id: Some(6),
                    name: "DevOps".to_string(),
                    slug: "devops".to_string(),
                    parent_id: Some(1),
                    created_at: Some(now),
                    updated_at: Some(now),
                    synced_at: None,
                },
                Tag {
                    id: Some(7),
                    name: "Web Development".to_string(),
                    slug: "web-development".to_string(),
                    parent_id: Some(1),
                    created_at: Some(now),
                    updated_at: Some(now),
                    synced_at: None,
                },
                Tag {
                    id: Some(8),
                    name: "Entrepreneurship".to_string(),
                    slug: "entrepreneurship".to_string(),
                    parent_id: Some(2),
                    created_at: Some(now),
                    updated_at: Some(now),
                    synced_at: None,
                },
                Tag {
                    id: Some(9),
                    name: "Finance".to_string(),
                    slug: "finance".to_string(),
                    parent_id: Some(2),
                    created_at: Some(now),
                    updated_at: Some(now),
                    synced_at: None,
                },
                Tag {
                    id: Some(10),
                    name: "UI Design".to_string(),
                    slug: "ui-design".to_string(),
                    parent_id: Some(3),
                    created_at: Some(now),
                    updated_at: Some(now),
                    synced_at: None,
                },
                Tag {
                    id: Some(11),
                    name: "UX Design".to_string(),
                    slug: "ux-design".to_string(),
                    parent_id: Some(3),
                    created_at: Some(now),
                    updated_at: Some(now),
                    synced_at: None,
                },
                Tag {
                    id: Some(12),
                    name: "Graphic Design".to_string(),
                    slug: "graphic-design".to_string(),
                    parent_id: Some(3),
                    created_at: Some(now),
                    updated_at: Some(now),
                    synced_at: None,
                },
                Tag {
                    id: Some(13),
                    name: "Digital Marketing".to_string(),
                    slug: "digital-marketing".to_string(),
                    parent_id: Some(4),
                    created_at: Some(now),
                    updated_at: Some(now),
                    synced_at: None,
                },
                Tag {
                    id: Some(14),
                    name: "Content Marketing".to_string(),
                    slug: "content-marketing".to_string(),
                    parent_id: Some(4),
                    created_at: Some(now),
                    updated_at: Some(now),
                    synced_at: None,
                },
            ];

            let mut tags = self.tags.write().unwrap();
            *tags = default_tags;
            self.next_tag_id.store(15, Ordering::SeqCst);
            drop(tags);
            self.save_to_persistence();
        }
    }
}

#[derive(Clone)]
pub struct SupabaseTagService;

impl SupabaseTagService {
    pub fn new() -> Self {
        SupabaseTagService
    }

    pub async fn get_all_tags(&self) -> Result<Vec<Tag>, String> {
        todo!("Supabase integration not implemented yet")
    }

    pub async fn get_tags_for_content(&self, _content_id: i32) -> Result<Vec<Tag>, String> {
        todo!("Supabase integration not implemented yet")
    }

    pub async fn add_tag_to_content(
        &self,
        _request: ContentTagRequest,
    ) -> Result<ContentTag, String> {
        todo!("Supabase integration not implemented yet")
    }

    pub async fn remove_tag_from_content(
        &self,
        _content_id: i32,
        _tag_id: i32,
    ) -> Result<(), String> {
        todo!("Supabase integration not implemented yet")
    }
}

impl Default for LocalTagService {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for SupabaseTagService {
    fn default() -> Self {
        Self::new()
    }
}
