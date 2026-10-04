use async_trait::async_trait;
use serde::de::DeserializeOwned;

use super::SupabaseConfig;
use crate::app::dto::{TagDto, UpsertTag};
use crate::infra::http::SupabaseHttp;
use crate::port::TagRepo;
use crate::port::http::{Http, HttpContext};

const TABLE_TAG: &str = "tags";
const COL_ID: &str = "id";
const COL_SLUG: &str = "slug";
const FILTER_IN: &str = "in.";
const ERR_NOT_CONFIGURED: &str = "supabase not configured";
const ERR_NOT_FOUND: &str = "no row returned";

#[derive(Clone)]
pub struct SupabaseTagRepo {
    http: SupabaseHttp,
    ctx: Option<SupabaseConfig>,
}

impl SupabaseTagRepo {
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
        let rows = self.http.get(self.ctx()?, TABLE_TAG, query).await?;
        rows.into_iter()
            .map(|row| serde_json::from_value(row).map_err(|e| e.to_string()))
            .collect()
    }

    async fn one<T: DeserializeOwned>(&self, query: &[(&str, &str)]) -> Result<Option<T>, String> {
        self.rows(query).await.map(|mut rows| rows.drain(..).next())
    }
}

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
impl TagRepo for SupabaseTagRepo {
    async fn list(&self) -> Result<Vec<TagDto>, String> {
        self.rows(&[]).await
    }

    async fn get(&self, id: i32) -> Result<Option<TagDto>, String> {
        self.one(&[("id", &id.to_string())]).await
    }

    async fn get_by_slug(&self, slug: &str) -> Result<Option<TagDto>, String> {
        self.one(&[(COL_SLUG, slug)]).await
    }

    async fn create(&self, data: UpsertTag) -> Result<TagDto, String> {
        let body = serde_json::to_value(&data).map_err(|e| e.to_string())?;
        let rows = self.http.create(self.ctx()?, TABLE_TAG, &body).await?;
        rows.into_iter()
            .next()
            .map(|row| serde_json::from_value(row).map_err(|e| e.to_string()))
            .unwrap_or_else(|| Err(ERR_NOT_FOUND.to_string()))
    }

    async fn update(&self, id: i32, data: UpsertTag) -> Result<TagDto, String> {
        let body = serde_json::to_value(&data).map_err(|e| e.to_string())?;
        let rows = self.http.update(self.ctx()?, TABLE_TAG, id, &body).await?;
        rows.into_iter()
            .next()
            .map(|row| serde_json::from_value(row).map_err(|e| e.to_string()))
            .unwrap_or_else(|| Err(ERR_NOT_FOUND.to_string()))
    }

    async fn delete(&self, id: i32) -> Result<(), String> {
        self.http.delete(self.ctx()?, TABLE_TAG, id).await
    }

    async fn list_by_ids(&self, ids: &[i32]) -> Result<Vec<TagDto>, String> {
        if ids.is_empty() {
            return Ok(Vec::new());
        }
        let list = ids
            .iter()
            .map(|id| id.to_string())
            .collect::<Vec<_>>()
            .join(",");
        let filter = format!("{FILTER_IN}{list}");
        self.rows(&[(COL_ID, filter.as_str())]).await
    }
}
