use crate::models::{Content, ContentTag, ContentTagRequest, Tag};
use crate::utils::config::Config;
use dioxus::prelude::*;
use gloo_net::http::Request;
use supabase_client::{ClientConfig, build_headers, create, delete, get, get_by_in};
const TAGS_TABLE: &str = "tags";
const CONTENT_TAGS_TABLE: &str = "content_tags";

#[derive(Clone, PartialEq)]
pub struct TagService {
    remote_service: SupabaseTagService,
}

impl TagService {
    pub fn new(config: Option<Config>) -> Self {
        TagService {
            remote_service: SupabaseTagService::new(config),
        }
    }

    pub async fn get_all_tags(&self) -> Result<Vec<Tag>, String> {
        self.remote_service.get_all_tags().await
    }

    pub async fn get_tags_for_content(&self, content_id: i32) -> Result<Vec<Tag>, String> {
        self.remote_service.get_tags_for_content(content_id).await
    }

    pub async fn get_content_tags_for_content(
        &self,
        content_id: i32,
    ) -> Result<Vec<ContentTag>, String> {
        self.remote_service
            .get_content_tags_for_content(content_id)
            .await
    }

    pub async fn get_content_tags_for_tag(&self, tag_id: i32) -> Result<Vec<ContentTag>, String> {
        self.remote_service.get_content_tags_for_tag(tag_id).await
    }

    pub async fn get_content_ids_for_tag(&self, tag_id: i32) -> Result<Vec<i32>, String> {
        self.remote_service.get_content_ids_for_tag(tag_id).await
    }

    pub async fn get_content_for_tag(&self, tag_id: i32) -> Result<Vec<Content>, String> {
        self.remote_service.get_content_for_tag(tag_id).await
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
        let current_content_tags = self.get_content_tags_for_content(content_id).await?;
        let current_tag_ids: Vec<i32> = current_content_tags.iter().map(|ct| ct.tag_id).collect();

        let config = self
            .remote_service
            .config
            .as_ref()
            .ok_or("Supabase not configured")?;

        let ids_to_delete: Vec<i32> = current_content_tags
            .iter()
            .filter(|ct| !tag_ids.contains(&ct.tag_id))
            .filter_map(|ct| ct.id)
            .collect();

        for id in ids_to_delete {
            delete(config, CONTENT_TAGS_TABLE, id).await?;
        }

        for tag_id in &tag_ids {
            if !current_tag_ids.contains(tag_id) {
                self.add_tag_to_content(ContentTagRequest {
                    content_id,
                    tag_id: *tag_id,
                })
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
        Self::new(None)
    }
}
#[derive(Clone, PartialEq)]
pub struct SupabaseTagService {
    config: Option<ClientConfig>,
}

impl SupabaseTagService {
    pub fn new(config: Option<Config>) -> Self {
        let client_config = config.and_then(|c| {
            let url = c.supabase_url?;
            let anon_key = c.supabase_anon_key?;
            Some(supabase_client::ClientConfig {
                base_url: url,
                anon_key,
                service_role_key: None,
                jwt_token: c.jwt_token,
            })
        });

        Self {
            config: client_config,
        }
    }

    pub async fn get_all_tags(&self) -> Result<Vec<Tag>, String> {
        let config = self.config.as_ref().ok_or("Supabase not configured")?;
        get::<Tag>(config, TAGS_TABLE, &[]).await
    }

    pub async fn get_tags_for_content(&self, content_id: i32) -> Result<Vec<Tag>, String> {
        let config = self.config.as_ref().ok_or("Supabase not configured")?;

        let content_tags: Vec<ContentTag> = get(
            config,
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

    pub async fn get_content_tags_for_content(
        &self,
        content_id: i32,
    ) -> Result<Vec<ContentTag>, String> {
        let config = self.config.as_ref().ok_or("Supabase not configured")?;

        get(
            config,
            CONTENT_TAGS_TABLE,
            &[("content_id", &content_id.to_string())],
        )
        .await
    }

    pub async fn add_tag_to_content(
        &self,
        request: ContentTagRequest,
    ) -> Result<ContentTag, String> {
        let config = self.config.as_ref().ok_or("Supabase not configured")?;

        let url = format!("{}/{}", config.rest_url(), CONTENT_TAGS_TABLE);
        tracing::debug!("Adding tag to content at URL: {}", url);

        let body = serde_json::to_string(&request)
            .map_err(|e| format!("Failed to serialize request: {}", e))?;
        tracing::debug!("Request body: {}", body);

        let request: gloo_net::http::Request = Request::post(&url)
            .headers(build_headers(config, true, None, false)?)
            .body(body)
            .map_err(|e| format!("Failed to build request: {}", e))?;

        let response = request
            .send()
            .await
            .map_err(|e| format!("Failed to create data: {}", e))?;

        let response_text = response
            .text()
            .await
            .map_err(|e| format!("Failed to read response text: {}", e))?;

        tracing::debug!("Supabase raw response: {}", response_text);

        match serde_json::from_str::<Vec<ContentTag>>(&response_text) {
            Ok(results) => {
                tracing::debug!("Parsed {} content_tag items from response", results.len());
                results
                    .into_iter()
                    .next()
                    .ok_or_else(|| "Failed to create content_tag".to_string())
            }
            Err(e) => {
                tracing::error!("Failed to parse response JSON: {}", e);
                tracing::error!("Response was: {}", response_text);
                Err(format!("Failed to parse response: {}", e))
            }
        }
    }

    pub async fn remove_tag_from_content(
        &self,
        content_id: i32,
        tag_id: i32,
    ) -> Result<(), String> {
        let config = self.config.as_ref().ok_or("Supabase not configured")?;

        let content_tags: Vec<ContentTag> = get(
            config,
            CONTENT_TAGS_TABLE,
            &[
                ("content_id", &content_id.to_string()),
                ("tag_id", &tag_id.to_string()),
            ],
        )
        .await?;

        if let Some(content_tag) = content_tags.into_iter().next() {
            if let Some(id) = content_tag.id {
                delete(config, CONTENT_TAGS_TABLE, id).await
            } else {
                Err("ContentTag has no ID".to_string())
            }
        } else {
            Err("ContentTag not found".to_string())
        }
    }

    pub async fn create_tag(&self, request: crate::models::TagRequest) -> Result<Tag, String> {
        let config = self.config.as_ref().ok_or("Supabase not configured")?;

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

        let result = create::<Tag, Tag>(config, TAGS_TABLE, &tag).await?;
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
        let config = self.config.as_ref().ok_or("Supabase not configured")?;

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

        let result = supabase_client::update::<Tag, Tag>(config, TAGS_TABLE, id, &tag).await?;
        result
            .into_iter()
            .next()
            .ok_or_else(|| "Failed to update tag".to_string())
    }

    pub async fn delete_tag(&self, id: i32) -> Result<(), String> {
        let config = self.config.as_ref().ok_or("Supabase not configured")?;
        delete(config, TAGS_TABLE, id).await
    }

    pub async fn get_tag_by_id(&self, id: i32) -> Result<Option<Tag>, String> {
        let config = self.config.as_ref().ok_or("Supabase not configured")?;

        let tags: Vec<Tag> = get(config, TAGS_TABLE, &[("id", &id.to_string())]).await?;
        Ok(tags.into_iter().next())
    }

    pub async fn get_content_tags_for_tag(&self, tag_id: i32) -> Result<Vec<ContentTag>, String> {
        let config = self.config.as_ref().ok_or("Supabase not configured")?;

        get(
            config,
            CONTENT_TAGS_TABLE,
            &[("tag_id", &tag_id.to_string())],
        )
        .await
    }

    pub async fn get_content_ids_for_tag(&self, tag_id: i32) -> Result<Vec<i32>, String> {
        let config = self.config.as_ref().ok_or("Supabase not configured")?;

        let content_tags: Vec<ContentTag> = get(
            config,
            CONTENT_TAGS_TABLE,
            &[("tag_id", &tag_id.to_string())],
        )
        .await?;

        Ok(content_tags.into_iter().map(|ct| ct.content_id).collect())
    }

    pub async fn get_content_for_tag(&self, tag_id: i32) -> Result<Vec<Content>, String> {
        let config = self.config.as_ref().ok_or("Supabase not configured")?;

        let content_ids = self.get_content_ids_for_tag(tag_id).await?;

        if content_ids.is_empty() {
            return Ok(Vec::new());
        }

        get_by_in::<Content>(config, "content", "id", &content_ids).await
    }
}
