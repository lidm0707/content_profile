use crate::models::{Content, ContentTag, ContentTagRequest, Tag, TagRequest};
use crate::services::BackendApiService;
use crate::utils::config::{AppMode, Config};
use content_backend::infra::http::SupabaseHttp;
use content_backend::port::http::{Http, HttpContext};

const TAGS_TABLE: &str = "tags";
const CONTENT_TAGS_TABLE: &str = "content_tags";
const CONTENT_TABLE: &str = "content";
const ERR_NOT_CONFIGURED: &str = "Supabase not configured";

#[derive(Clone)]
pub struct TagService {
    remote_service: SupabaseTagService,
    backend_service: BackendApiService,
    mode: AppMode,
}

impl TagService {
    pub fn new(config: Option<Config>) -> Self {
        let mode = config.as_ref().map(|c| c.mode).unwrap_or(AppMode::Office);
        TagService {
            remote_service: SupabaseTagService::new(config.clone()),
            backend_service: BackendApiService::new(config),
            mode,
        }
    }

    pub async fn get_all_tags(&self) -> Result<Vec<Tag>, String> {
        match self.mode {
            AppMode::Backend => self.backend_service.get_all_tags().await,
            _ => self.remote_service.get_all_tags().await,
        }
    }

    pub async fn get_tags_for_content(&self, content_id: i32) -> Result<Vec<Tag>, String> {
        match self.mode {
            AppMode::Backend => self.backend_service.get_tags_for_content(content_id).await,
            _ => self.remote_service.get_tags_for_content(content_id).await,
        }
    }

    pub async fn get_content_tags_for_content(
        &self,
        content_id: i32,
    ) -> Result<Vec<ContentTag>, String> {
        match self.mode {
            AppMode::Backend => {
                self.backend_service
                    .get_content_tags_for_content(content_id)
                    .await
            }
            _ => {
                self.remote_service
                    .get_content_tags_for_content(content_id)
                    .await
            }
        }
    }

    pub async fn get_content_tags_for_tag(&self, tag_id: i32) -> Result<Vec<ContentTag>, String> {
        match self.mode {
            AppMode::Backend => self.backend_service.get_content_tags_for_tag(tag_id).await,
            _ => self.remote_service.get_content_tags_for_tag(tag_id).await,
        }
    }

    pub async fn get_content_ids_for_tag(&self, tag_id: i32) -> Result<Vec<i32>, String> {
        match self.mode {
            AppMode::Backend => self.backend_service.get_content_ids_for_tag(tag_id).await,
            _ => self.remote_service.get_content_ids_for_tag(tag_id).await,
        }
    }

    pub async fn get_content_for_tag(&self, tag_id: i32) -> Result<Vec<Content>, String> {
        match self.mode {
            AppMode::Backend => self.backend_service.get_content_for_tag(tag_id).await,
            _ => self.remote_service.get_content_for_tag(tag_id).await,
        }
    }

    pub async fn add_tag_to_content(
        &mut self,
        request: ContentTagRequest,
    ) -> Result<ContentTag, String> {
        match self.mode {
            AppMode::Backend => self.backend_service.add_tag_to_content(request).await,
            _ => self.remote_service.add_tag_to_content(request).await,
        }
    }

    pub async fn remove_tag_from_content(
        &mut self,
        content_id: i32,
        tag_id: i32,
    ) -> Result<(), String> {
        match self.mode {
            AppMode::Backend => {
                self.backend_service
                    .remove_tag_from_content(content_id, tag_id)
                    .await
            }
            _ => {
                self.remote_service
                    .remove_tag_from_content(content_id, tag_id)
                    .await
            }
        }
    }

    pub async fn update_content_tags(
        &mut self,
        content_id: i32,
        tag_ids: Vec<i32>,
    ) -> Result<(), String> {
        let current_content_tags = self.get_content_tags_for_content(content_id).await?;
        let current_tag_ids: Vec<i32> = current_content_tags.iter().map(|ct| ct.tag_id).collect();

        let ids_to_delete: Vec<i32> = current_content_tags
            .iter()
            .filter(|ct| !tag_ids.contains(&ct.tag_id))
            .filter_map(|ct| ct.id)
            .collect();

        for id in ids_to_delete {
            match self.mode {
                AppMode::Backend => self.backend_service.delete_content_tag(id).await?,
                _ => self.remote_service.delete_content_tag(id).await?,
            }
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

    pub async fn create_tag(&mut self, request: TagRequest) -> Result<Tag, String> {
        match self.mode {
            AppMode::Backend => self.backend_service.create_tag(request).await,
            _ => self.remote_service.create_tag(request).await,
        }
    }

    pub async fn update_tag(&mut self, id: i32, request: TagRequest) -> Result<Tag, String> {
        match self.mode {
            AppMode::Backend => self.backend_service.update_tag(id, request).await,
            _ => self.remote_service.update_tag(id, request).await,
        }
    }

    pub async fn delete_tag(&mut self, id: i32) -> Result<(), String> {
        match self.mode {
            AppMode::Backend => self.backend_service.delete_tag(id).await,
            _ => self.remote_service.delete_tag(id).await,
        }
    }

    pub async fn get_tag_by_id(&self, id: i32) -> Result<Option<Tag>, String> {
        match self.mode {
            AppMode::Backend => self.backend_service.get_tag_by_id(id).await,
            _ => self.remote_service.get_tag_by_id(id).await,
        }
    }
}

impl Default for TagService {
    fn default() -> Self {
        Self::new(None)
    }
}

#[derive(Clone, PartialEq)]
pub struct SupabaseTagService {
    ctx: Option<HttpContext>,
}

impl SupabaseTagService {
    pub fn new(config: Option<Config>) -> Self {
        let ctx = config.and_then(|c| {
            let base_url = c.supabase_url.clone()?;
            let anon_key = c.supabase_anon_key.clone()?;
            let mut ctx = HttpContext::new(base_url, anon_key);
            if let Some(token) = c.jwt_token.clone() {
                ctx = ctx.with_jwt_token(token);
            }
            Some(ctx)
        });

        Self { ctx }
    }

    fn http(&self) -> Result<(SupabaseHttp, &HttpContext), String> {
        let ctx = self.ctx.as_ref().ok_or(ERR_NOT_CONFIGURED)?;
        Ok((SupabaseHttp, ctx))
    }

    pub async fn get_all_tags(&self) -> Result<Vec<Tag>, String> {
        let (http, ctx) = self.http()?;
        let rows = http.get(ctx, TAGS_TABLE, &[]).await?;
        decode_rows(rows)
    }

    pub async fn get_tag_by_id(&self, id: i32) -> Result<Option<Tag>, String> {
        let (http, ctx) = self.http()?;
        let id_str = id.to_string();
        let rows = http.get(ctx, TAGS_TABLE, &[("id", &id_str)]).await?;
        decode_first(rows)
    }

    pub async fn create_tag(&self, request: TagRequest) -> Result<Tag, String> {
        let (http, ctx) = self.http()?;
        let tag = tag_from_request(None, request);
        let body = serde_json::to_value(&tag).map_err(|e| e.to_string())?;
        let rows = http.create(ctx, TAGS_TABLE, &body).await?;
        decode_first(rows)?.ok_or_else(|| "Failed to create tag".to_string())
    }

    pub async fn update_tag(&self, id: i32, request: TagRequest) -> Result<Tag, String> {
        let (http, ctx) = self.http()?;
        let tag = tag_from_request(Some(id), request);
        let body = serde_json::to_value(&tag).map_err(|e| e.to_string())?;
        let rows = http.update(ctx, TAGS_TABLE, id, &body).await?;
        decode_first(rows)?.ok_or_else(|| "Failed to update tag".to_string())
    }

    pub async fn delete_tag(&self, id: i32) -> Result<(), String> {
        let (http, ctx) = self.http()?;
        http.delete(ctx, TAGS_TABLE, id).await
    }

    pub async fn get_content_tags_for_content(
        &self,
        content_id: i32,
    ) -> Result<Vec<ContentTag>, String> {
        let (http, ctx) = self.http()?;
        let id_str = content_id.to_string();
        let rows = http
            .get(ctx, CONTENT_TAGS_TABLE, &[("content_id", &id_str)])
            .await?;
        decode_rows(rows)
    }

    pub async fn get_content_tags_for_tag(&self, tag_id: i32) -> Result<Vec<ContentTag>, String> {
        let (http, ctx) = self.http()?;
        let id_str = tag_id.to_string();
        let rows = http
            .get(ctx, CONTENT_TAGS_TABLE, &[("tag_id", &id_str)])
            .await?;
        decode_rows(rows)
    }

    pub async fn get_content_ids_for_tag(&self, tag_id: i32) -> Result<Vec<i32>, String> {
        Ok(self
            .get_content_tags_for_tag(tag_id)
            .await?
            .into_iter()
            .map(|ct| ct.content_id)
            .collect())
    }

    pub async fn add_tag_to_content(
        &self,
        request: ContentTagRequest,
    ) -> Result<ContentTag, String> {
        let (http, ctx) = self.http()?;
        let body = serde_json::to_value(&request)
            .map_err(|e| format!("Failed to serialize request: {}", e))?;
        tracing::debug!("Adding tag to content: {:?}", body);

        let rows = http.create(ctx, CONTENT_TAGS_TABLE, &body).await?;
        decode_first(rows)?.ok_or_else(|| "Failed to create content_tag".to_string())
    }

    pub async fn remove_tag_from_content(
        &self,
        content_id: i32,
        tag_id: i32,
    ) -> Result<(), String> {
        let (http, ctx) = self.http()?;
        let content_id_str = content_id.to_string();
        let tag_id_str = tag_id.to_string();
        let rows = http
            .get(
                ctx,
                CONTENT_TAGS_TABLE,
                &[("content_id", &content_id_str), ("tag_id", &tag_id_str)],
            )
            .await?;

        let content_tag: ContentTag =
            decode_first(rows)?.ok_or_else(|| "ContentTag not found".to_string())?;
        let id = content_tag.id.ok_or("ContentTag has no ID")?;
        http.delete(ctx, CONTENT_TAGS_TABLE, id).await
    }

    pub async fn delete_content_tag(&self, id: i32) -> Result<(), String> {
        let (http, ctx) = self.http()?;
        http.delete(ctx, CONTENT_TAGS_TABLE, id).await
    }

    pub async fn get_tags_for_content(&self, content_id: i32) -> Result<Vec<Tag>, String> {
        let tag_ids: Vec<i32> = self
            .get_content_tags_for_content(content_id)
            .await?
            .into_iter()
            .map(|ct| ct.tag_id)
            .collect();

        Ok(self
            .get_all_tags()
            .await?
            .into_iter()
            .filter(|tag| tag.id.is_some_and(|id| tag_ids.contains(&id)))
            .collect())
    }

    pub async fn get_content_for_tag(&self, tag_id: i32) -> Result<Vec<Content>, String> {
        let content_ids = self.get_content_ids_for_tag(tag_id).await?;
        if content_ids.is_empty() {
            return Ok(Vec::new());
        }

        let (http, ctx) = self.http()?;
        let in_filter = format!("in.({})", join_ids(&content_ids));
        let rows = http.get(ctx, CONTENT_TABLE, &[("id", &in_filter)]).await?;
        decode_rows(rows)
    }
}

fn tag_from_request(id: Option<i32>, request: TagRequest) -> Tag {
    let now = chrono::Utc::now();
    Tag {
        id,
        name: request.name,
        slug: request.slug,
        parent_id: request.parent_id,
        created_at: if id.is_none() { Some(now) } else { None },
        updated_at: Some(now),
        synced_at: None,
    }
}

fn join_ids(ids: &[i32]) -> String {
    ids.iter()
        .map(|id| id.to_string())
        .collect::<Vec<_>>()
        .join(",")
}

fn decode_rows<T: serde::de::DeserializeOwned>(
    rows: Vec<serde_json::Value>,
) -> Result<Vec<T>, String> {
    rows.into_iter()
        .map(|row| serde_json::from_value(row).map_err(|e| e.to_string()))
        .collect()
}

fn decode_first<T: serde::de::DeserializeOwned>(
    rows: Vec<serde_json::Value>,
) -> Result<Option<T>, String> {
    rows.into_iter()
        .next()
        .map(|row| serde_json::from_value(row).map_err(|e| e.to_string()))
        .transpose()
}
