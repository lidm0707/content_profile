use crate::models::{AuthError, AuthResponse, LoginRequest, Session, User};
use crate::utils::config::Config;
use gloo_net::http::Headers;
use gloo_net::http::Request;

const AUTH_PATH: &str = "auth/v1";

#[derive(Clone)]
pub struct AuthService {
    base_url: Option<String>,
    anon_key: Option<String>,
}

impl AuthService {
    pub fn new(config: Option<Config>) -> Self {
        let base_url = config.as_ref().and_then(|c| c.supabase_url.clone());
        let anon_key = config.as_ref().and_then(|c| c.supabase_anon_key.clone());

        AuthService { base_url, anon_key }
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

    fn get_auth_headers(&self, access_token: &str) -> Result<Headers, String> {
        let headers = self.get_headers()?;
        headers.set("Authorization", &format!("Bearer {}", access_token));
        Ok(headers)
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
                .json::<AuthError>()
                .await
                .map_err(|e| format!("Failed to parse error response: {}", e))?;
            return Err(format!(
                "Login failed: {} - {}",
                error.error, error.error_description
            ));
        }

        let auth_response: AuthResponse = response
            .json::<AuthResponse>()
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
                .json::<AuthError>()
                .await
                .map_err(|e| format!("Failed to parse error response: {}", e))?;
            return Err(format!(
                "Signup failed: {} - {}",
                error.error, error.error_description
            ));
        }

        let auth_response: AuthResponse = response
            .json::<AuthResponse>()
            .await
            .map_err(|e| format!("Failed to parse auth response: {}", e))?;

        auth_response.into_session()
    }

    pub async fn logout(&self, access_token: &str) -> Result<(), String> {
        if !self.is_configured() {
            return Err(
                "Authentication is not configured. Please set up Supabase credentials.".to_string(),
            );
        }

        let url = format!("{}/logout", self.auth_url());
        let headers = self.get_auth_headers(access_token)?;

        let response = Request::post(&url)
            .headers(headers)
            .send()
            .await
            .map_err(|e| format!("Failed to send logout request: {}", e))?;

        if !response.ok() {
            let error: AuthError = response
                .json::<AuthError>()
                .await
                .map_err(|e| format!("Failed to parse error response: {}", e))?;
            return Err(format!(
                "Logout failed: {} - {}",
                error.error, error.error_description
            ));
        }

        Ok(())
    }

    pub async fn get_user(&self, access_token: &str) -> Result<User, String> {
        if !self.is_configured() {
            return Err(
                "Authentication is not configured. Please set up Supabase credentials.".to_string(),
            );
        }

        let url = format!("{}/user", self.auth_url());
        let headers = self.get_auth_headers(access_token)?;

        let response = Request::get(&url)
            .headers(headers)
            .send()
            .await
            .map_err(|e| format!("Failed to send get user request: {}", e))?;

        if !response.ok() {
            let error: AuthError = response
                .json::<AuthError>()
                .await
                .map_err(|e| format!("Failed to parse error response: {}", e))?;
            return Err(format!(
                "Get user failed: {} - {}",
                error.error, error.error_description
            ));
        }

        let user: User = response
            .json::<User>()
            .await
            .map_err(|e| format!("Failed to parse user response: {}", e))?;

        Ok(user)
    }

    pub async fn refresh_token(&self, refresh_token: &str) -> Result<Session, String> {
        if !self.is_configured() {
            return Err(
                "Authentication is not configured. Please set up Supabase credentials.".to_string(),
            );
        }

        let url = format!("{}/token?grant_type=refresh_token", self.auth_url());
        let body = serde_json::json!({ "refresh_token": refresh_token });
        let body_str = serde_json::to_string(&body)
            .map_err(|e| format!("Failed to serialize request: {}", e))?;

        let response = Request::post(&url)
            .headers(self.get_headers()?)
            .body(body_str)
            .map_err(|e| format!("Failed to build: {}", e))?
            .send()
            .await
            .map_err(|e| format!("Failed to send refresh token request: {}", e))?;

        if !response.ok() {
            let error: AuthError = response
                .json::<AuthError>()
                .await
                .map_err(|e| format!("Failed to parse error response: {}", e))?;
            return Err(format!(
                "Refresh token failed: {} - {}",
                error.error, error.error_description
            ));
        }

        let auth_response: AuthResponse = response
            .json::<AuthResponse>()
            .await
            .map_err(|e| format!("Failed to parse auth response: {}", e))?;

        auth_response
            .into_session()
            .map_err(|e| format!("Failed to into session: {}", e))
    }
}

impl Default for AuthService {
    fn default() -> Self {
        Self::new(None)
    }
}
