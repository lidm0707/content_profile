use crate::models::{Content, ContentRequest};
use crate::utils::config::Config;
use supabase_client::{ClientConfig, create, delete, get, get_by, get_by_id, update};

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
        let results: Vec<Content> =
            create::<ContentRequest, Content>(config, "content", &content_request).await?;
        results
            .into_iter()
            .next()
            .ok_or_else(|| "No content returned".to_string())
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
}

impl Default for SupabaseService {
    fn default() -> Self {
        Self::new(None)
    }
}
