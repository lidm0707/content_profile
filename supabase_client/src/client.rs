use crate::config::ClientConfig;
use crate::models::{Content, ContentRequest};
use gloo_net::http::Headers;
use gloo_net::http::Request;
use tracing::{debug, error, info, trace, warn};
use url::Url;

const CONTENT_TABLE: &str = "content";
const API_KEY_HEADER: &str = "apikey";
const AUTHORIZATION_HEADER: &str = "Authorization";
const CONTENT_TYPE_HEADER: &str = "Content-Type";
const PREFER_HEADER: &str = "Prefer";
const RETURN_REPRESENTATION: &str = "return=representation";
const BEARER_PREFIX: &str = "Bearer ";
const APPLICATION_JSON: &str = "application/json";

/// Supabase client for content management
#[derive(Clone)]
pub struct SupabaseClient {
    config: ClientConfig,
}

impl SupabaseClient {
    pub fn new(config: ClientConfig) -> Self {
        info!("Initializing SupabaseClient...");
        let base_url = config.build_rest_url();
        info!("Supabase base URL: {}/{}", &base_url[..base_url.len().min(50)], CONTENT_TABLE);
        Self { config }
    }

    fn build_url(&self, path: &str, params: &[(&str, &str)]) -> Result<String, String> {
        let base_url = self.config.build_rest_url();
        let mut url = Url::parse(&format!("{}/{}", base_url, path))
            .map_err(|e| format!("Failed to parse URL: {}", e))?;

        for (key, value) in params {
            url.query_pairs_mut().append_pair(key, value);
        }

        Ok(url.to_string())
    }

    fn build_headers(&self) -> Result<Headers, String> {
        let headers = Headers::new();
        let anon_key = self.config.anon_key();

        headers.set(API_KEY_HEADER, anon_key);
        headers.set(AUTHORIZATION_HEADER, &format!("{}{}", BEARER_PREFIX, anon_key));
        headers.set(CONTENT_TYPE_HEADER, APPLICATION_JSON);
        headers.set(PREFER_HEADER, RETURN_REPRESENTATION);

        trace!("Supabase API headers prepared successfully");
        Ok(headers)
    }

    pub async fn get_all_content(&self) -> Result<Vec<Content>, String> {
        info!("Fetching all content from Supabase...");

        let url = self.build_url(CONTENT_TABLE, &[("order", "created_at.desc")])?;
        debug!("Making GET request to: {}", &url[..url.len().min(50)]);

        let response = Request::get(&url)
            .headers(self.build_headers()?)
            .send()
            .await
            .map_err(|e| {
                error!("Failed to send request to Supabase: {}", e);
                format!("Failed to fetch content: {}", e)
            })?;

        debug!("Received response with status: {}", response.status());

        response.json::<Vec<Content>>().await.map_err(|e| {
            error!("Failed to parse response JSON: {}", e);
            format!("Failed to parse response: {}", e)
        })
    }

    pub async fn get_content_by_id(&self, id: i32) -> Result<Option<Content>, String> {
        debug!("Fetching content by ID: {}", id);

        let url = self.build_url(CONTENT_TABLE, &[("id", &id.to_string())])?;

        let response = Request::get(&url)
            .headers(self.build_headers()?)
            .send()
            .await
            .map_err(|e| {
                error!("Failed to fetch content by ID {}: {}", id, e);
                format!("Failed to fetch content: {}", e)
            })?;

        let contents = response.json::<Vec<Content>>().await.map_err(|e| {
            error!("Failed to parse response: {}", e);
            format!("Failed to parse response: {}", e)
        })?;

        Ok(contents.into_iter().next())
    }

    pub async fn get_content_by_slug(&self, slug: &str) -> Result<Option<Content>, String> {
        let url = self.build_url(CONTENT_TABLE, &[("slug", slug)])?;

        let response = Request::get(&url)
            .headers(self.build_headers()?)
            .send()
            .await
            .map_err(|e| format!("Failed to fetch content: {}", e))?;

        let contents = response
            .json::<Vec<Content>>()
            .await
            .map_err(|e| format!("Failed to parse response: {}", e))?;

        Ok(contents.into_iter().next())
    }

    pub async fn create_content(&self, content_request: ContentRequest) -> Result<Content, String> {
        debug!("Creating new content in Supabase");

        let url = self.build_url(CONTENT_TABLE, &[])?;
        let body = serde_json::to_string(&content_request)
            .map_err(|e| format!("Failed to serialize request: {}", e))?;

        let response = Request::post(&url)
            .headers(self.build_headers()?)
            .body(body)
            .map_err(|e| format!("Failed to build: {}", e))?
            .send()
            .await
            .map_err(|e| {
                error!("Failed to create content: {}", e);
                format!("Failed to create content: {}", e)
            })?;

        debug!("Create content response status: {}", response.status());

        let contents = response.json::<Vec<Content>>().await.map_err(|e| {
            error!("Failed to parse create response: {}", e);
            format!("Failed to parse response: {}", e)
        })?;

        contents.into_iter().next().ok_or_else(|| {
            error!("No content returned from create operation");
            "No content returned".to_string()
        })
    }

    pub async fn update_content(
        &self,
        id: i32,
        content_request: ContentRequest,
    ) -> Result<Content, String> {
        debug!("Updating content ID: {}", id);

        let url = self.build_url(CONTENT_TABLE, &[("id", &id.to_string())])?;
        let body = serde_json::to_string(&content_request)
            .map_err(|e| format!("Failed to serialize request: {}", e))?;

        let response = Request::patch(&url)
            .headers(self.build_headers()?)
            .body(body)
            .map_err(|e| format!("Failed to build: {}", e))?
            .send()
            .await
            .map_err(|e| {
                error!("Failed to update content {}: {}", id, e);
                format!("Failed to update content: {}", e)
            })?;

        debug!("Update content response status: {}", response.status());

        let contents = response.json::<Vec<Content>>().await.map_err(|e| {
            error!("Failed to parse update response: {}", e);
            format!("Failed to parse response: {}", e)
        })?;

        contents.into_iter().next().ok_or_else(|| {
            error!("No content returned from update operation");
            "No content returned".to_string()
        })
    }

    pub async fn delete_content(&self, id: i32) -> Result<(), String> {
        let url = self.build_url(CONTENT_TABLE, &[("id", &id.to_string())])?;

        Request::delete(&url)
            .headers(self.build_headers()?)
            .send()
            .await
            .map_err(|e| format!("Failed to delete content: {}", e))?;

        Ok(())
    }

    pub async fn get_content_by_status(&self, status: &str) -> Result<Vec<Content>, String> {
        let url = self.build_url(
            CONTENT_TABLE,
            &[("status", status), ("order", "created_at.desc")],
        )?;

        Request::get(&url)
            .headers(self.build_headers()?)
            .send()
            .await
            .map_err(|e| format!("Failed to fetch content: {}", e))?
            .json::<Vec<Content>>()
            .await
            .map_err(|e| format!("Failed to parse response: {}", e))
    }
}
