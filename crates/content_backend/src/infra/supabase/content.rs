use async_trait::async_trait;
use serde::de::DeserializeOwned;

use super::SupabaseConfig;
use crate::app::dto::{ContentDto, UpsertContent};
use crate::infra::http::SupabaseHttp;
use crate::port::ContentRepo;
use crate::port::http::{Http, HttpContext};

const TABLE_CONTENT: &str = "content";
const COL_ID: &str = "id";
const COL_SLUG: &str = "slug";
const ORDER_CREATED_AT_DESC: &str = "created_at.desc";
const KEY_ORDER: &str = "order";
const KEY_OFFSET: &str = "offset";
const KEY_LIMIT: &str = "limit";
const FILTER_IN: &str = "in.";
const ERR_NOT_CONFIGURED: &str = "supabase not configured";
const ERR_NOT_FOUND: &str = "no row returned";

#[derive(Clone)]
pub struct SupabaseContentRepo {
    http: SupabaseHttp,
    ctx: Option<SupabaseConfig>,
}

impl SupabaseContentRepo {
    pub fn new(config: Option<SupabaseConfig>) -> Self {
        Self {
            http: SupabaseHttp,
            ctx: config,
        }
    }

    fn ctx(&self) -> Result<&HttpContext, String> {
        self.ctx
            .as_ref()
            .ok_or_else(|| ERR_NOT_CONFIGURED.to_string())
    }

    async fn rows<T: DeserializeOwned>(&self, query: &[(&str, &str)]) -> Result<Vec<T>, String> {
        let rows = self.http.get(self.ctx()?, TABLE_CONTENT, query).await?;
        rows.into_iter()
            .map(|row| serde_json::from_value(row).map_err(|e| e.to_string()))
            .collect()
    }

    async fn one<T: DeserializeOwned>(&self, query: &[(&str, &str)]) -> Result<Option<T>, String> {
        self.rows(query).await.map(|mut rows| rows.drain(..).next())
    }

    fn id_in_filter(&self, ids: &[i32]) -> String {
        let list = ids
            .iter()
            .map(|id| id.to_string())
            .collect::<Vec<_>>()
            .join(",");
        format!("{FILTER_IN}{list}")
    }

    pub async fn list_by_ids(&self, ids: &[i32]) -> Result<Vec<ContentDto>, String> {
        if ids.is_empty() {
            return Ok(Vec::new());
        }
        let filter = self.id_in_filter(ids);
        self.rows(&[(COL_ID, filter.as_str())]).await
    }
}

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
impl ContentRepo for SupabaseContentRepo {
    async fn list(&self) -> Result<Vec<ContentDto>, String> {
        self.rows(&[("order", ORDER_CREATED_AT_DESC)]).await
    }

    async fn get(&self, id: i32) -> Result<Option<ContentDto>, String> {
        self.one(&[("id", &id.to_string())]).await
    }

    async fn get_by_slug(&self, slug: &str) -> Result<Option<ContentDto>, String> {
        self.one(&[(COL_SLUG, slug)]).await
    }

    async fn create(&self, data: UpsertContent) -> Result<ContentDto, String> {
        let body = serde_json::to_value(&data).map_err(|e| e.to_string())?;
        let rows = self.http.create(self.ctx()?, TABLE_CONTENT, &body).await?;
        rows.into_iter()
            .next()
            .map(|row| serde_json::from_value(row).map_err(|e| e.to_string()))
            .unwrap_or_else(|| Err(ERR_NOT_FOUND.to_string()))
    }

    async fn update(&self, id: i32, data: UpsertContent) -> Result<ContentDto, String> {
        let body = serde_json::to_value(&data).map_err(|e| e.to_string())?;
        let rows = self
            .http
            .update(self.ctx()?, TABLE_CONTENT, id, &body)
            .await?;
        rows.into_iter()
            .next()
            .map(|row| serde_json::from_value(row).map_err(|e| e.to_string()))
            .unwrap_or_else(|| Err(ERR_NOT_FOUND.to_string()))
    }

    async fn delete(&self, id: i32) -> Result<(), String> {
        self.http.delete(self.ctx()?, TABLE_CONTENT, id).await
    }

    async fn list_page(&self, offset: u32, limit: u32) -> Result<Vec<ContentDto>, String> {
        let offset = offset.to_string();
        let limit = limit.to_string();
        self.rows(&[
            (KEY_ORDER, ORDER_CREATED_AT_DESC),
            (KEY_OFFSET, offset.as_str()),
            (KEY_LIMIT, limit.as_str()),
        ])
        .await
    }

    async fn count(&self) -> Result<u32, String> {
        self.http.count(self.ctx()?, TABLE_CONTENT, &[]).await
    }

    async fn list_by_ids(&self, ids: &[i32]) -> Result<Vec<ContentDto>, String> {
        if ids.is_empty() {
            return Ok(Vec::new());
        }
        let filter = self.id_in_filter(ids);
        self.rows(&[(COL_ID, filter.as_str())]).await
    }
}
