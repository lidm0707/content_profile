use crate::models::{Content, ContentRequest};
use crate::utils::config::Config;
use content_backend::infra::http::SupabaseHttp;
use content_backend::port::http::{Http, HttpContext};
use serde_json::Value;

const CONTENT_TABLE: &str = "content";
const ERR_NOT_CONFIGURED: &str = "Supabase not configured";
const ORDER_CREATED_DESC: &str = "created_at.desc";
const ORDER_KEY: &str = "order";
const STATUS_KEY: &str = "status";
const ID_KEY: &str = "id";
const OFFSET_KEY: &str = "offset";
const LIMIT_KEY: &str = "limit";

#[derive(Clone)]
pub struct SupabaseService {
    ctx: Option<HttpContext>,
}

impl SupabaseService {
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

    pub async fn get_all_content(&self) -> Result<Vec<Content>, String> {
        let (http, ctx) = self.http()?;
        let rows = http
            .get(ctx, CONTENT_TABLE, &[(ORDER_KEY, ORDER_CREATED_DESC)])
            .await?;
        decode_rows(rows)
    }

    pub async fn get_content_by_id(&self, id: i32) -> Result<Option<Content>, String> {
        let (http, ctx) = self.http()?;
        let id_str = id.to_string();
        let rows = http.get(ctx, CONTENT_TABLE, &[(ID_KEY, &id_str)]).await?;
        decode_first(rows)
    }

    pub async fn get_content_by_slug(&self, slug: &str) -> Result<Option<Content>, String> {
        let (http, ctx) = self.http()?;
        let rows = http.get(ctx, CONTENT_TABLE, &[("slug", slug)]).await?;
        decode_first(rows)
    }

    pub async fn create_content(&self, content_request: ContentRequest) -> Result<Content, String> {
        let (http, ctx) = self.http()?;

        let body = serde_json::to_value(&content_request)
            .map_err(|e| format!("Failed to serialize request: {}", e))?;
        tracing::debug!("Creating content with body: {:?}", body);

        let rows = http.create(ctx, CONTENT_TABLE, &body).await?;
        tracing::debug!("Parsed {} content items from response", rows.len());

        decode_first(rows)?.ok_or_else(|| "No content returned".to_string())
    }

    pub async fn update_content(
        &self,
        id: i32,
        content_request: ContentRequest,
    ) -> Result<Content, String> {
        let (http, ctx) = self.http()?;
        let body = serde_json::to_value(&content_request)
            .map_err(|e| format!("Failed to serialize request: {}", e))?;

        let rows = http.update(ctx, CONTENT_TABLE, id, &body).await?;
        decode_first(rows)?.ok_or_else(|| "No content returned".to_string())
    }

    pub async fn delete_content(&self, id: i32) -> Result<(), String> {
        let (http, ctx) = self.http()?;
        http.delete(ctx, CONTENT_TABLE, id).await
    }

    pub async fn get_content_by_status(&self, status: &str) -> Result<Vec<Content>, String> {
        let (http, ctx) = self.http()?;
        let rows = http
            .get(
                ctx,
                CONTENT_TABLE,
                &[(STATUS_KEY, status), (ORDER_KEY, ORDER_CREATED_DESC)],
            )
            .await?;
        decode_rows(rows)
    }

    pub async fn get_content_by_ids(&self, ids: &[i32]) -> Result<Vec<Content>, String> {
        if ids.is_empty() {
            return Ok(Vec::new());
        }
        let (http, ctx) = self.http()?;
        let in_filter = format!("in.{}", join_ids(ids));
        let rows = http
            .get(ctx, CONTENT_TABLE, &[(ID_KEY, &in_filter)])
            .await?;
        decode_rows(rows)
    }

    pub async fn get_paginated_content(
        &self,
        filters: &[(&str, &str)],
        offset: u32,
        limit: u32,
    ) -> Result<Vec<Content>, String> {
        let (http, ctx) = self.http()?;
        let query = paginate(filters, offset, limit);
        let rows = http.get(ctx, CONTENT_TABLE, &as_query(&query)).await?;
        decode_rows(rows)
    }

    pub async fn get_paginated_content_with_count(
        &self,
        filters: &[(&str, &str)],
        offset: u32,
        limit: u32,
    ) -> Result<(Vec<Content>, u32), String> {
        let (http, ctx) = self.http()?;
        let query = paginate(filters, offset, limit);
        let rows = http.get(ctx, CONTENT_TABLE, &as_query(&query)).await?;
        let total = http.count(ctx, CONTENT_TABLE, filters).await?;
        Ok((decode_rows(rows)?, total))
    }

    pub async fn count_content(&self, filters: &[(&str, &str)]) -> Result<u32, String> {
        let (http, ctx) = self.http()?;
        http.count(ctx, CONTENT_TABLE, filters).await
    }
}

fn join_ids(ids: &[i32]) -> String {
    ids.iter()
        .map(|id| id.to_string())
        .collect::<Vec<_>>()
        .join(",")
}

fn paginate<'a>(
    filters: &'a [(&'a str, &'a str)],
    offset: u32,
    limit: u32,
) -> Vec<(&'a str, String)> {
    filters
        .iter()
        .map(|(k, v)| (*k, v.to_string()))
        .chain([
            (OFFSET_KEY, offset.to_string()),
            (LIMIT_KEY, limit.to_string()),
        ])
        .collect()
}

fn as_query<'a>(pairs: &'a [(&'a str, String)]) -> Vec<(&'a str, &'a str)> {
    pairs.iter().map(|(k, v)| (*k, v.as_str())).collect()
}

fn decode_rows<T: serde::de::DeserializeOwned>(rows: Vec<Value>) -> Result<Vec<T>, String> {
    rows.into_iter()
        .map(|row| serde_json::from_value(row).map_err(|e| e.to_string()))
        .collect()
}

fn decode_first<T: serde::de::DeserializeOwned>(rows: Vec<Value>) -> Result<Option<T>, String> {
    rows.into_iter()
        .next()
        .map(|row| serde_json::from_value(row).map_err(|e| e.to_string()))
        .transpose()
}

impl Default for SupabaseService {
    fn default() -> Self {
        Self::new(None)
    }
}
