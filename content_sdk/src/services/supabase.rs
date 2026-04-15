use crate::models::{Content, ContentRequest};
use crate::utils::config::Config;
use gloo_net::http::Request;
use supabase_client::{
    ClientConfig, build_headers, count, create, delete, get, get_by, get_by_id, get_by_in,
    get_paginated, update,
};

#[derive(Clone)]
pub struct SupabaseService {
    config: Option<ClientConfig>,
}

impl SupabaseService {
    pub fn new(config: Option<Config>) -> Self {
        let client_config = config.and_then(|c| {
            let url = c.supabase_url?;
            let anon_key = c.supabase_anon_key?;
            Some(ClientConfig {
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

    pub async fn get_all_content(&self) -> Result<Vec<Content>, String> {
        let config = self.config.as_ref().ok_or("Supabase not configured")?;
        get::<Content>(config, "content", &[("order", "created_at.desc")]).await
    }

    pub async fn get_content_by_id(&self, id: i32) -> Result<Option<Content>, String> {
        let config = self.config.as_ref().ok_or("Supabase not configured")?;
        get_by_id::<Content>(config, "content", id).await
    }

    pub async fn get_content_by_slug(&self, slug: &str) -> Result<Option<Content>, String> {
        let config = self.config.as_ref().ok_or("Supabase not configured")?;
        let results: Vec<Content> = get_by::<Content>(config, "content", "slug", slug).await?;
        Ok(results.into_iter().next())
    }

    pub async fn create_content(&self, content_request: ContentRequest) -> Result<Content, String> {
        let config = self.config.as_ref().ok_or("Supabase not configured")?;

        let url = format!("{}/content", config.rest_url());
        tracing::debug!("Creating content at URL: {}", url);

        let body = serde_json::to_string(&content_request)
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

        match serde_json::from_str::<Vec<Content>>(&response_text) {
            Ok(results) => {
                tracing::debug!("Parsed {} content items from response", results.len());
                results
                    .into_iter()
                    .next()
                    .ok_or_else(|| "No content returned".to_string())
            }
            Err(e) => {
                tracing::error!("Failed to parse response JSON: {}", e);
                tracing::error!("Response was: {}", response_text);
                Err(format!("Failed to parse response: {}", e))
            }
        }
    }

    pub async fn update_content(
        &self,
        id: i32,
        content_request: ContentRequest,
    ) -> Result<Content, String> {
        let config = self.config.as_ref().ok_or("Supabase not configured")?;
        let results: Vec<Content> =
            update::<ContentRequest, Content>(config, "content", id, &content_request).await?;
        results
            .into_iter()
            .next()
            .ok_or_else(|| "No content returned".to_string())
    }

    pub async fn delete_content(&self, id: i32) -> Result<(), String> {
        let config = self.config.as_ref().ok_or("Supabase not configured")?;
        delete(config, "content", id).await
    }

    pub async fn get_content_by_status(&self, status: &str) -> Result<Vec<Content>, String> {
        let config = self.config.as_ref().ok_or("Supabase not configured")?;
        get::<Content>(
            config,
            "content",
            &[("status", status), ("order", "created_at.desc")],
        )
        .await
    }

    pub async fn get_content_by_ids(&self, ids: &[i32]) -> Result<Vec<Content>, String> {
        let config = self.config.as_ref().ok_or("Supabase not configured")?;
        if ids.is_empty() {
            return Ok(Vec::new());
        }
        get_by_in::<Content>(config, "content", "id", ids).await
    }

    pub async fn get_paginated_content(
        &self,
        filters: &[(&str, &str)],
        offset: u32,
        limit: u32,
    ) -> Result<Vec<Content>, String> {
        let config = self.config.as_ref().ok_or("Supabase not configured")?;
        get_paginated::<Content>(config, "content", filters, offset, limit).await
    }

    pub async fn count_content(&self, filters: &[(&str, &str)]) -> Result<u32, String> {
        let config = self.config.as_ref().ok_or("Supabase not configured")?;
        count(config, "content", filters).await
    }
}

impl Default for SupabaseService {
    fn default() -> Self {
        Self::new(None)
    }
}
