use crate::models::{ContentTag, ContentTagRequest, Tag};
use crate::utils::config::get_config;
use dioxus::prelude::*;
use supabase_client::{ClientConfig, create, delete, get};

const TAGS_TABLE: &str = "tags";
const CONTENT_TAGS_TABLE: &str = "content_tags";

#[derive(Clone, PartialEq)]
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
        {
            return self.local_service.get_all_tags();
        }
    }

    pub async fn get_tags_for_content(&self, content_id: i32) -> Result<Vec<Tag>, String> {
        {
            return self.local_service.get_tags_for_content(content_id);
        }
    }

    pub async fn add_tag_to_content(
        &mut self,
        request: ContentTagRequest,
    ) -> Result<ContentTag, String> {
        {
            return self.local_service.add_tag_to_content(request);
        }
    }

    pub async fn remove_tag_from_content(
        &mut self,
        content_id: i32,
        tag_id: i32,
    ) -> Result<(), String> {
        {
            return self
                .local_service
                .remove_tag_from_content(content_id, tag_id);
        }
    }

    pub async fn update_content_tags(
        &mut self,
        content_id: i32,
        tag_ids: Vec<i32>,
    ) -> Result<(), String> {
        let current_tags = self.get_tags_for_content(content_id).await?;
        let current_tag_ids: Vec<i32> = current_tags
            .iter()
            .map(|t| t.id.unwrap_or_default())
            .collect();

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

    pub async fn create_tag(&mut self, request: crate::models::TagRequest) -> Result<Tag, String> {
        self.local_service.create_tag(request)
    }

    pub async fn update_tag(
        &mut self,
        id: i32,
        request: crate::models::TagRequest,
    ) -> Result<Tag, String> {
        self.local_service.update_tag(id, request)
    }

    pub async fn delete_tag(&mut self, id: i32) -> Result<(), String> {
        self.local_service.delete_tag(id)
    }

    pub async fn get_tag_by_id(&self, id: i32) -> Result<Option<Tag>, String> {
        self.local_service.get_tag_by_id(id)
    }
}

impl Default for TagService {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, PartialEq, Props)]
pub struct LocalTagService {
    tags: Signal<Vec<Tag>>,
    content_tags: Signal<Vec<ContentTag>>,
    next_tag_id: Signal<usize>,
    next_content_tag_id: Signal<usize>,
}

impl LocalTagService {
    const TAGS_KEY: &'static str = "cms_tags";
    const CONTENT_TAGS_KEY: &'static str = "cms_content_tags";
    const NEXT_TAG_ID_KEY: &'static str = "cms_next_tag_id";
    const NEXT_CONTENT_TAG_ID_KEY: &'static str = "cms_next_content_tag_id";

    pub fn new() -> Self {
        let mut service = LocalTagService {
            tags: Signal::new(Vec::new()),
            content_tags: Signal::new(Vec::new()),
            next_tag_id: Signal::new(1),
            next_content_tag_id: Signal::new(1),
        };
        service.load_from_persistence();
        service.initialize_default_tags();
        service
    }

    pub fn get_all_tags(&self) -> Result<Vec<Tag>, String> {
        let tags = self.tags.read();
        Ok(tags.clone())
    }

    pub fn get_tags_for_content(&self, content_id: i32) -> Result<Vec<Tag>, String> {
        let content_tags = self.content_tags.read();
        let tags = self.tags.read();

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

    pub fn add_tag_to_content(&mut self, request: ContentTagRequest) -> Result<ContentTag, String> {
        let content_tags = self.content_tags.read();

        if content_tags
            .iter()
            .any(|ct| ct.content_id == request.content_id && ct.tag_id == request.tag_id)
        {
            return Err("Tag already added to content".to_string());
        }

        drop(content_tags);

        let id = self.next_content_tag_id.cloned() as i32;
        let now = chrono::Utc::now();
        let content_tag = ContentTag {
            id: Some(id),
            content_id: request.content_id,
            tag_id: request.tag_id,
            created_at: Some(now),
        };

        let mut content_tags = self.content_tags.write();
        content_tags.push(content_tag.clone());
        drop(content_tags);

        self.save_to_persistence();

        Ok(content_tag)
    }

    pub fn remove_tag_from_content(&mut self, content_id: i32, tag_id: i32) -> Result<(), String> {
        let mut content_tags = self.content_tags.write();
        let initial_len = content_tags.len();
        content_tags.retain(|ct| ct.content_id != content_id || ct.tag_id != tag_id);

        if content_tags.len() == initial_len {
            return Err("ContentTag relationship not found".to_string());
        }

        drop(content_tags);

        self.save_to_persistence();

        Ok(())
    }

    pub fn create_tag(&mut self, request: crate::models::TagRequest) -> Result<Tag, String> {
        let tags = self.tags.read();

        if tags.iter().any(|t| t.slug == request.slug) {
            return Err("Tag with this slug already exists".to_string());
        }

        drop(tags);

        let id = self.next_tag_id.cloned() as i32;
        let now = chrono::Utc::now();
        let tag = Tag {
            id: Some(id),
            name: request.name.clone(),
            slug: request.slug.clone(),
            parent_id: request.parent_id,
            created_at: Some(now),
            updated_at: Some(now),
            synced_at: None,
        };

        let mut tags = self.tags.write();
        tags.push(tag.clone());
        drop(tags);

        *self.next_tag_id.write() += 1;
        self.save_to_persistence();

        Ok(tag)
    }

    pub fn update_tag(
        &mut self,
        id: i32,
        request: crate::models::TagRequest,
    ) -> Result<Tag, String> {
        let mut tags = self.tags.write();

        let tag_index = tags
            .iter()
            .position(|t| t.id == Some(id))
            .ok_or_else(|| "Tag not found".to_string())?;

        if tags
            .iter()
            .any(|t| t.slug == request.slug && t.id != Some(id))
        {
            return Err("Tag with this slug already exists".to_string());
        }

        let now = chrono::Utc::now();
        let updated_tag = Tag {
            id: Some(id),
            name: request.name.clone(),
            slug: request.slug.clone(),
            parent_id: request.parent_id,
            created_at: tags[tag_index].created_at,
            updated_at: Some(now),
            synced_at: None,
        };

        tags[tag_index] = updated_tag.clone();
        drop(tags);

        self.save_to_persistence();

        Ok(updated_tag)
    }

    pub fn delete_tag(&mut self, id: i32) -> Result<(), String> {
        let mut content_tags = self.content_tags.write();
        content_tags.retain(|ct| ct.tag_id != id);
        drop(content_tags);

        let mut tags = self.tags.write();
        let initial_len = tags.len();
        tags.retain(|t| t.id != Some(id));

        if tags.len() == initial_len {
            return Err("Tag not found".to_string());
        }

        drop(tags);

        self.save_to_persistence();

        Ok(())
    }

    pub fn get_tag_by_id(&self, id: i32) -> Result<Option<Tag>, String> {
        let tags = self.tags.read();
        Ok(tags.iter().find(|t| t.id == Some(id)).cloned())
    }

    fn save_to_persistence(&self) {
        if let Some(window) = web_sys::window()
            && let Ok(Some(storage)) = window.local_storage()
        {
            let tags = self.tags.read();
            if let Ok(json) = serde_json::to_string(&*tags) {
                let _ = storage.set_item(Self::TAGS_KEY, &json);
            }

            let content_tags = self.content_tags.read();
            if let Ok(json) = serde_json::to_string(&*content_tags) {
                let _ = storage.set_item(Self::CONTENT_TAGS_KEY, &json);
            }

            let next_tag_id = self.next_tag_id.cloned();
            let _ = storage.set_item(Self::NEXT_TAG_ID_KEY, &next_tag_id.to_string());

            let next_content_tag_id = self.next_content_tag_id.cloned();
            let _ = storage.set_item(
                Self::NEXT_CONTENT_TAG_ID_KEY,
                &next_content_tag_id.to_string(),
            );
        }
    }

    fn load_from_persistence(&mut self) {
        if let Some(window) = web_sys::window()
            && let Ok(Some(storage)) = window.local_storage()
        {
            if let Ok(Some(json)) = storage.get_item(Self::TAGS_KEY)
                && let Ok(loaded) = serde_json::from_str::<Vec<Tag>>(&json)
            {
                let mut tags = self.tags.write();
                *tags = loaded;
                if let Some(max_id) = tags.iter().filter_map(|t| t.id).max() {
                    *self.next_tag_id.write() = (max_id + 1) as usize;
                }
            }

            if let Ok(Some(json)) = storage.get_item(Self::CONTENT_TAGS_KEY)
                && let Ok(loaded) = serde_json::from_str::<Vec<ContentTag>>(&json)
            {
                let mut content_tags = self.content_tags.write();
                *content_tags = loaded;
                if let Some(max_id) = content_tags.iter().filter_map(|ct| ct.id).max() {
                    *self.next_content_tag_id.write() = (max_id + 1) as usize;
                }
            }
        }
    }

    fn initialize_default_tags(&mut self) {
        let tags = self.tags.read();
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

            let mut tags = self.tags.write();
            *tags = default_tags;
            *self.next_tag_id.write() = 15;
        }
        self.save_to_persistence();
    }
}

#[derive(Clone, PartialEq)]
pub struct SupabaseTagService;

impl SupabaseTagService {
    pub fn new() -> Self {
        SupabaseTagService
    }

    pub async fn get_all_tags(&self) -> Result<Vec<Tag>, String> {
        let app_config = get_config();
        let config = build_client_config(&app_config)?;

        get::<Tag>(&config, TAGS_TABLE, &[]).await
    }

    pub async fn get_tags_for_content(&self, content_id: i32) -> Result<Vec<Tag>, String> {
        let app_config = get_config();
        let config = build_client_config(&app_config)?;

        let content_tags: Vec<ContentTag> = get(
            &config,
            CONTENT_TAGS_TABLE,
            &[("content_id", &content_id.to_string())],
        )
        .await?;

        let tag_ids: Vec<i32> = content_tags.iter().map(|ct| ct.tag_id).collect();

        let all_tags: Vec<Tag> = self.get_all_tags().await?;

        Ok(all_tags
            .into_iter()
            .filter(|tag| tag.id.is_some_and(|id| tag_ids.contains(&id)))
            .collect())
    }

    pub async fn add_tag_to_content(
        &self,
        request: ContentTagRequest,
    ) -> Result<ContentTag, String> {
        let app_config = get_config();
        let config = build_client_config(&app_config)?;

        let result =
            create::<ContentTagRequest, ContentTag>(&config, CONTENT_TAGS_TABLE, &request).await?;
        result
            .into_iter()
            .next()
            .ok_or_else(|| "Failed to create content_tag".to_string())
    }

    pub async fn remove_tag_from_content(
        &self,
        content_id: i32,
        tag_id: i32,
    ) -> Result<(), String> {
        let app_config = get_config();
        let config = build_client_config(&app_config)?;

        let content_tags: Vec<ContentTag> = get(
            &config,
            CONTENT_TAGS_TABLE,
            &[
                ("content_id", &content_id.to_string()),
                ("tag_id", &tag_id.to_string()),
            ],
        )
        .await?;

        if let Some(content_tag) = content_tags.into_iter().next() {
            if let Some(id) = content_tag.id {
                delete(&config, CONTENT_TAGS_TABLE, id).await
            } else {
                Err("ContentTag has no ID".to_string())
            }
        } else {
            Err("ContentTag not found".to_string())
        }
    }

    pub async fn create_tag(&self, request: crate::models::TagRequest) -> Result<Tag, String> {
        let app_config = get_config();
        let config = build_client_config(&app_config)?;

        let now = chrono::Utc::now();
        let tag = Tag {
            id: None,
            name: request.name,
            slug: request.slug,
            parent_id: request.parent_id,
            created_at: Some(now),
            updated_at: Some(now),
            synced_at: None,
        };

        let result = create::<Tag, Tag>(&config, TAGS_TABLE, &tag).await?;
        result
            .into_iter()
            .next()
            .ok_or_else(|| "Failed to create tag".to_string())
    }

    pub async fn update_tag(
        &self,
        id: i32,
        request: crate::models::TagRequest,
    ) -> Result<Tag, String> {
        let app_config = get_config();
        let config = build_client_config(&app_config)?;

        let now = chrono::Utc::now();
        let tag = Tag {
            id: Some(id),
            name: request.name,
            slug: request.slug,
            parent_id: request.parent_id,
            created_at: None,
            updated_at: Some(now),
            synced_at: None,
        };

        let result = supabase_client::update::<Tag, Tag>(&config, TAGS_TABLE, id, &tag).await?;
        result
            .into_iter()
            .next()
            .ok_or_else(|| "Failed to update tag".to_string())
    }

    pub async fn delete_tag(&self, id: i32) -> Result<(), String> {
        let app_config = get_config();
        let config = build_client_config(&app_config)?;

        delete(&config, TAGS_TABLE, id).await
    }

    pub async fn get_tag_by_id(&self, id: i32) -> Result<Option<Tag>, String> {
        let app_config = get_config();
        let config = build_client_config(&app_config)?;

        let tags: Vec<Tag> = get(&config, TAGS_TABLE, &[("id", &id.to_string())]).await?;
        Ok(tags.into_iter().next())
    }
}

impl Default for LocalTagService {
    fn default() -> Self {
        Self::new()
    }
}

fn build_client_config(app_config: &crate::utils::config::Config) -> Result<ClientConfig, String> {
    let supabase_url = app_config
        .supabase_url
        .as_ref()
        .ok_or_else(|| "SUPABASE_URL must be set".to_string())?;
    let supabase_anon_key = app_config
        .supabase_anon_key
        .as_ref()
        .ok_or_else(|| "SUPABASE_ANON_KEY must be set".to_string())?;

    Ok(supabase_client::client_config(
        supabase_url.clone(),
        supabase_anon_key.clone(),
    ))
}
