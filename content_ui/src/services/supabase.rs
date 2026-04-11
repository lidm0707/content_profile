use crate::models::{Content, ContentRequest};
use crate::services::session::SessionStorage;
use crate::utils::config::get_config;
use supabase_client::{
    ClientConfig, client_config, create, delete, get, get_by, get_by_id, update,
};

#[derive(Clone)]
pub struct SupabaseService {
    config: ClientConfig,
}

impl SupabaseService {
    pub fn new() -> Self {
        let app_config = get_config();
        let supabase_url = app_config
            .supabase_url
            .expect("SUPABASE_URL must be set in Supabase mode");
        let supabase_anon_key = app_config
            .supabase_anon_key
            .expect("SUPABASE_ANON_KEY must be set in Supabase mode");

        let config = client_config(supabase_url, supabase_anon_key);

        Self { config }
    }

    fn config_with_jwt(&self) -> ClientConfig {
        match SessionStorage::load_session() {
            Ok(Some(session)) => self.config.clone().with_jwt_token(session.access_token),
            _ => self.config.clone(),
        }
    }

    pub async fn get_all_content(&self) -> Result<Vec<Content>, String> {
        let config = self.config_with_jwt();
        get::<Content>(&config, "content", &[("order", "created_at.desc")]).await
    }

    pub async fn get_content_by_id(&self, id: i32) -> Result<Option<Content>, String> {
        let config = self.config_with_jwt();
        get_by_id::<Content>(&config, "content", id).await
    }

    pub async fn get_content_by_slug(&self, slug: &str) -> Result<Option<Content>, String> {
        let config = self.config_with_jwt();
        let results: Vec<Content> = get_by::<Content>(&config, "content", "slug", slug).await?;
        Ok(results.into_iter().next())
    }

    pub async fn create_content(&self, content_request: ContentRequest) -> Result<Content, String> {
        let config = self.config_with_jwt();
        let results: Vec<Content> =
            create::<ContentRequest, Content>(&config, "content", &content_request).await?;
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
        let config = self.config_with_jwt();
        let results: Vec<Content> =
            update::<ContentRequest, Content>(&config, "content", id, &content_request).await?;
        results
            .into_iter()
            .next()
            .ok_or_else(|| "No content returned".to_string())
    }

    pub async fn delete_content(&self, id: i32) -> Result<(), String> {
        let config = self.config_with_jwt();
        delete(&config, "content", id).await
    }

    pub async fn get_content_by_status(&self, status: &str) -> Result<Vec<Content>, String> {
        let config = self.config_with_jwt();
        get::<Content>(
            &config,
            "content",
            &[("status", status), ("order", "created_at.desc")],
        )
        .await
    }
}

impl Default for SupabaseService {
    fn default() -> Self {
        Self::new()
    }
}
