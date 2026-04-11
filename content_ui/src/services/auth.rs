use crate::models::{AuthError, AuthResponse, LoginRequest, Session, User};
use crate::utils::config::get_config;
use gloo_net::http::Headers;
use gloo_net::http::Request;

const AUTH_PATH: &str = "auth/v1";

#[derive(Clone)]
pub struct AuthService {
    base_url: Option<String>,
    anon_key: Option<String>,
}

impl AuthService {
    pub fn new() -> Self {
        let config = get_config();
        let base_url = config.supabase_url;

        AuthService {
            base_url,
            anon_key: config.supabase_anon_key,
        }
    }

    fn auth_url(&self) -> String {
        self.base_url
            .as_ref()
            .map(|url| format!("{}/{}", url, AUTH_PATH))
            .unwrap_or_default()
    }

    pub fn is_configured(&self) -> bool {
        self.base_url.is_some()
            && self.anon_key.is_some()
            && self.base_url.as_ref().is_some_and(|u| !u.is_empty())
            && self.anon_key.as_ref().is_some_and(|k| !k.is_empty())
    }

    fn get_headers(&self) -> Result<Headers, String> {
        let headers = Headers::new();
        let anon_key = self
            .anon_key
            .as_ref()
            .filter(|k| !k.is_empty())
            .ok_or_else(|| "Supabase anon key not configured".to_string())?;

        headers.set("apikey", anon_key);
        headers.set("Content-Type", "application/json");

        Ok(headers)
    }

    fn get_auth_headers(&self, _access_token: &str) -> Result<Headers, String> {
        unimplemented!("get_auth_headers not implemented")
    }

    pub async fn login(&self, request: LoginRequest) -> Result<Session, String> {
        if !self.is_configured() {
            return Err(
                "Authentication is not configured. Please set up Supabase credentials.".to_string(),
            );
        }

        let url = format!("{}/token?grant_type=password", self.auth_url());
        let body = serde_json::to_string(&request)
            .map_err(|e| format!("Failed to serialize request: {}", e))?;

        let response = Request::post(&url)
            .headers(self.get_headers()?)
            .body(body)
            .map_err(|e| format!("Failed to build: {}", e))?
            .send()
            .await
            .map_err(|e| format!("Failed to send login request: {}", e))?;

        if !response.ok() {
            let error: AuthError = response
                .json()
                .await
                .map_err(|e| format!("Failed to parse error response: {}", e))?;
            return Err(format!(
                "Login failed: {} - {}",
                error.error, error.error_description
            ));
        }

        let auth_response: AuthResponse = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse auth response: {}", e))?;

        auth_response
            .into_session()
            .map_err(|e| format!("Failed to into session: {}", e))
    }

    pub async fn signup(&self, request: LoginRequest) -> Result<Session, String> {
        if !self.is_configured() {
            return Err(
                "Authentication is not configured. Please set up Supabase credentials.".to_string(),
            );
        }

        let url = format!("{}/signup", self.auth_url());
        let body = serde_json::to_string(&request)
            .map_err(|e| format!("Failed to serialize request: {}", e))?;

        let response = Request::post(&url)
            .headers(self.get_headers()?)
            .body(body)
            .map_err(|e| format!("Failed to build: {}", e))?
            .send()
            .await
            .map_err(|e| format!("Failed to send signup request: {}", e))?;

        if !response.ok() {
            let error: AuthError = response
                .json()
                .await
                .map_err(|e| format!("Failed to parse error response: {}", e))?;
            return Err(format!(
                "Signup failed: {} - {}",
                error.error, error.error_description
            ));
        }

        let auth_response: AuthResponse = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse auth response: {}", e))?;

        auth_response.into_session()
    }

    pub async fn logout(&self, _access_token: &str) -> Result<(), String> {
        unimplemented!("logout not implemented")
    }

    pub async fn get_user(&self, _access_token: &str) -> Result<User, String> {
        unimplemented!("get_user not implemented")
    }

    pub async fn refresh_token(&self, _refresh_token: &str) -> Result<Session, String> {
        unimplemented!("refresh_token not implemented")
    }
}

impl Default for AuthService {
    fn default() -> Self {
        Self::new()
    }
}
