const DEFAULT_API_PATH: &str = "rest/v1";

#[derive(Clone, Debug, PartialEq)]
pub struct ClientConfig {
    pub base_url: String,
    pub anon_key: String,
    pub service_role_key: Option<String>,
    pub jwt_token: Option<String>,
}

impl ClientConfig {
    pub fn new(base_url: String, anon_key: String) -> Self {
        Self {
            base_url,
            anon_key,
            service_role_key: None,
            jwt_token: None,
        }
    }

    pub fn with_service_role_key(mut self, key: String) -> Self {
        self.service_role_key = Some(key);
        self
    }

    pub fn with_jwt_token(mut self, token: String) -> Self {
        self.jwt_token = Some(token);
        self
    }

    pub fn rest_url(&self) -> String {
        format!("{}/{}", self.base_url, DEFAULT_API_PATH)
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.base_url.is_empty() {
            return Err("base_url cannot be empty".to_string());
        }

        if self.anon_key.is_empty() {
            return Err("anon_key cannot be empty".to_string());
        }

        if let Some(ref key) = self.service_role_key {
            if key.is_empty() {
                return Err("service_role_key cannot be empty if provided".to_string());
            }
        }

        if let Some(ref token) = self.jwt_token {
            if token.is_empty() {
                return Err("jwt_token cannot be empty if provided".to_string());
            }
        }

        Ok(())
    }
}

pub fn client_config(base_url: String, anon_key: String) -> ClientConfig {
    ClientConfig::new(base_url, anon_key)
}
