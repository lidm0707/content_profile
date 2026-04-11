use crate::models::{Content, ContentRequest};
use crate::utils::config::get_config;
use gloo_net::http::Headers;
use gloo_net::http::Request;
use tracing::{debug, error, info, trace, warn};
use url::Url;

/// Supabase service for content management
#[derive(Clone)]
pub struct SupabaseService {
    base_url: Option<String>,
    anon_key: Option<String>,
}

impl SupabaseService {
    /// Creates a new Supabase service instance
    pub fn new() -> Self {
        info!("Initializing SupabaseService...");
        let config = get_config();

        let base_url = config.supabase_url.map(|url| format!("{}/rest/v1", url));

        if let Some(ref url) = base_url {
            info!(
                "Supabase base URL configured: {}/rest/v1",
                &url[..url.len().min(30)]
            );
        } else {
            warn!("Supabase base URL not configured - service will fail on operations");
        }

        if config.supabase_anon_key.is_some() {
            debug!("Supabase anon key configured");
        } else {
            warn!("Supabase anon key not configured - service will fail on operations");
        }

        SupabaseService {
            base_url,
            anon_key: config.supabase_anon_key,
        }
    }

    /// Builds table URL for content
    const fn content_table_url(&self) -> &str {
        "content"
    }

    /// Builds URL with query parameters
    fn build_url_with_query(&self, path: &str, params: &[(&str, &str)]) -> Result<String, String> {
        let base_url = self
            .base_url
            .as_ref()
            .ok_or_else(|| "Supabase URL not configured".to_string())?;

        let mut url = Url::parse(&format!("{}/{}", base_url, path))
            .map_err(|e| format!("Failed to parse URL: {}", e))?;

        for (key, value) in params {
            url.query_pairs_mut().append_pair(key, value);
        }

        let mut url_string = url.to_string();
        if url_string.ends_with('&') {
            url_string.pop();
        }

        Ok(url_string)
    }

    /// Checks if Supabase service is properly configured
    fn is_configured(&self) -> bool {
        self.base_url.is_some()
            && self.anon_key.is_some()
            && self.base_url.as_ref().is_some_and(|u| !u.is_empty())
            && self.anon_key.as_ref().is_some_and(|k| !k.is_empty())
    }

    /// Creates headers for Supabase API requests
    fn get_headers(&self) -> Result<Headers, String> {
        let headers = Headers::new();

        let anon_key = self
            .anon_key
            .as_ref()
            .filter(|k| !k.is_empty())
            .ok_or_else(|| {
                error!("Supabase anon key not configured");
                "Supabase anon key not configured".to_string()
            })?;

        headers.set("apikey", anon_key);
        headers.set("Authorization", &format!("Bearer {}", anon_key));
        headers.set("Content-Type", "application/json");
        headers.set("Prefer", "return=representation");

        trace!("Supabase API headers prepared successfully");
        Ok(headers)
    }

    /// Fetches all content items
    pub async fn get_all_content(&self) -> Result<Vec<Content>, String> {
        info!("Fetching all content from Supabase...");

        if !self.is_configured() {
            return Err(
                "Supabase is not configured. Please set up Supabase credentials.".to_string(),
            );
        }

        let url =
            self.build_url_with_query(self.content_table_url(), &[("order", "created_at.desc")])?;
        debug!("Making GET request to: {}", &url[..url.len().min(50)]);

        let response = Request::get(&url)
            .headers(self.get_headers()?)
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

    /// Fetches content by ID
    pub async fn get_content_by_id(&self, id: i32) -> Result<Option<Content>, String> {
        debug!("Fetching content by ID: {}", id);

        if !self.is_configured() {
            return Err(
                "Supabase is not configured. Please set up Supabase credentials.".to_string(),
            );
        }

        let id_filter = format!("eq.{}", id);
        let url = self.build_url_with_query(self.content_table_url(), &[("id", &id_filter)])?;

        let response = Request::get(&url)
            .headers(self.get_headers()?)
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

    /// Fetches content by slug
    pub async fn get_content_by_slug(&self, slug: &str) -> Result<Option<Content>, String> {
        if !self.is_configured() {
            return Err(
                "Supabase is not configured. Please set up Supabase credentials.".to_string(),
            );
        }

        let slug_filter = format!("eq.{}", slug);
        let url = self.build_url_with_query(self.content_table_url(), &[("slug", &slug_filter)])?;

        let response = Request::get(&url)
            .headers(self.get_headers()?)
            .send()
            .await
            .map_err(|e| format!("Failed to fetch content: {}", e))?;

        let contents = response
            .json::<Vec<Content>>()
            .await
            .map_err(|e| format!("Failed to parse response: {}", e))?;

        Ok(contents.into_iter().next())
    }

    /// Creates a new content item
    pub async fn create_content(&self, content_request: ContentRequest) -> Result<Content, String> {
        debug!("Creating new content in Supabase");

        if !self.is_configured() {
            return Err(
                "Supabase is not configured. Please set up Supabase credentials.".to_string(),
            );
        }

        let url = self.build_url_with_query(self.content_table_url(), &[])?;
        let body = serde_json::to_string(&content_request)
            .map_err(|e| format!("Failed to serialize request: {}", e))?;

        let response = Request::post(&url)
            .headers(self.get_headers()?)
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

    /// Updates an existing content item
    pub async fn update_content(
        &self,
        id: i32,
        content_request: ContentRequest,
    ) -> Result<Content, String> {
        debug!("Updating content ID: {}", id);

        if !self.is_configured() {
            return Err(
                "Supabase is not configured. Please set up Supabase credentials.".to_string(),
            );
        }

        let id_filter = format!("eq.{}", id);
        let url = self.build_url_with_query(self.content_table_url(), &[("id", &id_filter)])?;
        let body = serde_json::to_string(&content_request)
            .map_err(|e| format!("Failed to serialize request: {}", e))?;

        let response = Request::patch(&url)
            .headers(self.get_headers()?)
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

    /// Deletes a content item
    pub async fn delete_content(&self, id: i32) -> Result<(), String> {
        if !self.is_configured() {
            return Err(
                "Supabase is not configured. Please set up Supabase credentials.".to_string(),
            );
        }

        let id_filter = format!("eq.{}", id);
        let url = self.build_url_with_query(self.content_table_url(), &[("id", &id_filter)])?;

        Request::delete(&url)
            .headers(self.get_headers()?)
            .send()
            .await
            .map_err(|e| format!("Failed to delete content: {}", e))?;

        Ok(())
    }

    /// Fetches content by status
    pub async fn get_content_by_status(&self, status: &str) -> Result<Vec<Content>, String> {
        if !self.is_configured() {
            return Err(
                "Supabase is not configured. Please set up Supabase credentials.".to_string(),
            );
        }

        let url = self.build_url_with_query(
            self.content_table_url(),
            &[("status", status), ("order", "created_at.desc")],
        )?;

        Request::get(&url)
            .headers(self.get_headers()?)
            .send()
            .await
            .map_err(|e| format!("Failed to fetch content: {}", e))?
            .json::<Vec<Content>>()
            .await
            .map_err(|e| format!("Failed to parse response: {}", e))
    }
}

impl Default for SupabaseService {
    fn default() -> Self {
        Self::new()
    }
}
