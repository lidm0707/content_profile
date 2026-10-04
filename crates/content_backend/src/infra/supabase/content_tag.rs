use async_trait::async_trait;
use serde_json::json;

use super::SupabaseConfig;
use crate::app::dto::ContentTagDto;
use crate::infra::http::SupabaseHttp;
use crate::port::ContentTagRepo;
use crate::port::http::{Http, HttpContext};

const TABLE_CONTENT_TAGS: &str = "content_tags";
const COL_CONTENT_ID: &str = "content_id";
const COL_TAG_ID: &str = "tag_id";
const ERR_NOT_CONFIGURED: &str = "supabase not configured";

#[derive(Clone)]
pub struct SupabaseContentTagRepo {
    http: SupabaseHttp,
    ctx: Option<SupabaseConfig>,
}

impl SupabaseContentTagRepo {
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

    async fn rows(&self, query: &[(&str, &str)]) -> Result<Vec<ContentTagDto>, String> {
        let rows = self
            .http
            .get(self.ctx()?, TABLE_CONTENT_TAGS, query)
            .await?;
        rows.into_iter()
            .map(|row| serde_json::from_value(row).map_err(|e| e.to_string()))
            .collect()
    }
}

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
impl ContentTagRepo for SupabaseContentTagRepo {
    async fn list_for_content(&self, content_id: i32) -> Result<Vec<ContentTagDto>, String> {
        self.rows(&[(COL_CONTENT_ID, &content_id.to_string())])
            .await
    }

    async fn list_for_tag(&self, tag_id: i32) -> Result<Vec<ContentTagDto>, String> {
        self.rows(&[(COL_TAG_ID, &tag_id.to_string())]).await
    }

    async fn add(&self, content_id: i32, tag_id: i32) -> Result<ContentTagDto, String> {
        let body = json!({ COL_CONTENT_ID: content_id, COL_TAG_ID: tag_id });
        let rows = self
            .http
            .create(self.ctx()?, TABLE_CONTENT_TAGS, &body)
            .await?;
        match rows.into_iter().next() {
            Some(row) => serde_json::from_value(row).map_err(|e| e.to_string()),
            None => Ok(ContentTagDto {
                id: None,
                content_id,
                tag_id,
                created_at: None,
            }),
        }
    }

    async fn remove(&self, id: i32) -> Result<(), String> {
        self.http.delete(self.ctx()?, TABLE_CONTENT_TAGS, id).await
    }
}
