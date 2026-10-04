#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HttpContext {
    pub base_url: String,
    pub anon_key: String,
    pub jwt_token: Option<String>,
    pub service_role_key: Option<String>,
}

impl HttpContext {
    pub fn new(base_url: String, anon_key: String) -> Self {
        Self {
            base_url,
            anon_key,
            jwt_token: None,
            service_role_key: None,
        }
    }

    pub fn with_jwt_token(mut self, token: String) -> Self {
        self.jwt_token = Some(token);
        self
    }

    pub fn with_service_role_key(mut self, key: String) -> Self {
        self.service_role_key = Some(key);
        self
    }
}
