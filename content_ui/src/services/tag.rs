use crate::models::{ContentTag, ContentTagRequest, Tag};
use crate::utils::config::get_config;
use dioxus::prelude::*;
use supabase_client::{ClientConfig, create, delete, get};
const TAGS_TABLE: &str = "tags";
const CONTENT_TAGS_TABLE: &str = "content_tags";

#[derive(Clone, PartialEq)]
pub struct TagService {
    remote_service: SupabaseTagService,
}

impl TagService {
    pub fn new() -> Self {
        TagService {
            remote_service: SupabaseTagService::new(),
        }
    }

    pub async fn get_all_tags(&self) -> Result<Vec<Tag>, String> {
        self.remote_service.get_all_tags().await
    }

    pub async fn get_tags_for_content(&self, content_id: i32) -> Result<Vec<Tag>, String> {
        self.remote_service.get_tags_for_content(content_id).await
    }

    pub async fn add_tag_to_content(
        &mut self,
        request: ContentTagRequest,
    ) -> Result<ContentTag, String> {
        self.remote_service.add_tag_to_content(request).await
    }

    pub async fn remove_tag_from_content(
        &mut self,
        content_id: i32,
        tag_id: i32,
    ) -> Result<(), String> {
        self.remote_service
            .remove_tag_from_content(content_id, tag_id)
            .await
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
        self.remote_service.create_tag(request).await
    }

    pub async fn update_tag(
        &mut self,
        id: i32,
        request: crate::models::TagRequest,
    ) -> Result<Tag, String> {
        self.remote_service.update_tag(id, request).await
    }

    pub async fn delete_tag(&mut self, id: i32) -> Result<(), String> {
        self.remote_service.delete_tag(id).await
    }

    pub async fn get_tag_by_id(&self, id: i32) -> Result<Option<Tag>, String> {
        self.remote_service.get_tag_by_id(id).await
    }
}

impl Default for TagService {
    fn default() -> Self {
        Self::new()
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
