/// Configuration for Supabase client
#[derive(Clone, Debug, PartialEq)]
pub struct ClientConfig {
    base_url: String,
    anon_key: String,
}

impl ClientConfig {
    const DEFAULT_API_PATH: &str = "rest/v1";

    pub fn new(base_url: String, anon_key: String) -> Self {
        Self { base_url, anon_key }
    }

    pub fn build_rest_url(&self) -> String {
        format!("{}/{}", self.base_url, Self::DEFAULT_API_PATH)
    }

    pub fn anon_key(&self) -> &str {
        &self.anon_key
    }
}

/// Builder for ClientConfig
pub struct ClientConfigBuilder {
    base_url: Option<String>,
    anon_key: Option<String>,
}

impl ClientConfigBuilder {
    pub fn new() -> Self {
        Self {
            base_url: None,
            anon_key: None,
        }
    }

    pub fn base_url(mut self, url: String) -> Self {
        self.base_url = Some(url);
        self
    }

    pub fn anon_key(mut self, key: String) -> Self {
        self.anon_key = Some(key);
        self
    }

    pub fn build(self) -> Result<ClientConfig, String> {
        let base_url = self
            .base_url
            .ok_or_else(|| "base_url is required".to_string())?;
        let anon_key = self
            .anon_key
            .ok_or_else(|| "anon_key is required".to_string())?;

        Ok(ClientConfig { base_url, anon_key })
    }
}

impl Default for ClientConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}
