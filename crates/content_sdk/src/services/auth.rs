use crate::models::{AuthError, AuthResponse, LoginRequest, Session, User};
use crate::services::backend_api::BackendApiService;
use crate::utils::config::{AppMode, Config};
use gloo_net::http::Headers;
use gloo_net::http::Request;

const AUTH_PATH: &str = "auth/v1";
const BEARER: &str = "Bearer";
const ERR_SETTINGS_BACKEND_ONLY: &str = "Settings are only available in backend mode";
const MAX_RAW_ERROR_LEN: usize = 200;

/// Extracts a human-readable message from an error response, tolerating the
/// different GoTrue error body formats (and non-JSON bodies).
async fn error_message(response: &gloo_net::http::Response) -> String {
    let text = response.text().await.unwrap_or_default();
    match serde_json::from_str::<AuthError>(&text) {
        Ok(error) => error.message(),
        Err(_) if !text.trim().is_empty() => text.trim().chars().take(MAX_RAW_ERROR_LEN).collect(),
        Err(_) => format!("HTTP {}", response.status()),
    }
}

#[derive(Clone)]
pub struct AuthService {
    config: Option<Config>,
    base_url: Option<String>,
    anon_key: Option<String>,
    mode: AppMode,
    backend: BackendApiService,
}

impl AuthService {
    pub fn new(config: Option<Config>) -> Self {
        let mode = config.as_ref().map(|c| c.mode).unwrap_or(AppMode::Office);
        let backend = BackendApiService::new(config.clone());
        AuthService {
            base_url: config.as_ref().and_then(|c| c.supabase_url.clone()),
            anon_key: config.as_ref().and_then(|c| c.supabase_anon_key.clone()),
            config,
            mode,
            backend,
        }
    }

    /// Rebuilds the backend service with a new JWT token.
    pub fn update_jwt_token(&mut self, jwt_token: Option<String>) {
        let mut config = self
            .config
            .clone()
            .unwrap_or_else(|| Config::new("backend", "", "", jwt_token.clone(), None));
        config.jwt_token = jwt_token;
        self.backend = BackendApiService::new(Some(config.clone()));
        self.config = Some(config);
    }

    fn auth_url(&self) -> String {
        self.base_url
            .as_ref()
            .map(|url| format!("{}/{}", url, AUTH_PATH))
            .unwrap_or_default()
    }

    pub fn is_configured(&self) -> bool {
        if self.mode == AppMode::Backend {
            return true;
        }
        self.base_url.is_some()
            && self.anon_key.is_some()
            && self.base_url.as_ref().is_some_and(|u| !u.is_empty())
            && self.anon_key.as_ref().is_some_and(|k| !k.is_empty())
    }

    pub fn is_backend_mode(&self) -> bool {
        self.mode == AppMode::Backend
    }

    /// Builds the in-memory session from the HttpOnly cookie probe.
    /// Token fields stay empty — the cookie authenticates requests, so
    /// nothing secret lives in JS.
    pub async fn session_from_cookie(&self) -> Result<Option<Session>, String> {
        if self.mode != AppMode::Backend {
            return Ok(None);
        }
        let Some(username) = self.backend.get_session().await? else {
            return Ok(None);
        };
        Ok(Some(Session {
            access_token: String::new(),
            refresh_token: String::new(),
            expires_at: i64::MAX,
            token_type: BEARER.to_string(),
            user: User {
                id: username.clone(),
                email: username,
                email_confirmed_at: None,
                created_at: String::new(),
                updated_at: String::new(),
                last_sign_in_at: None,
            },
        }))
    }

    /// Loads the per-user settings stored on the backend.
    pub async fn load_settings(&self) -> Result<serde_json::Value, String> {
        if self.mode != AppMode::Backend {
            return Err(ERR_SETTINGS_BACKEND_ONLY.to_string());
        }
        self.backend.get_user_settings().await
    }

    /// Saves the per-user settings on the backend.
    pub async fn save_settings(&self, settings: serde_json::Value) -> Result<(), String> {
        if self.mode != AppMode::Backend {
            return Err(ERR_SETTINGS_BACKEND_ONLY.to_string());
        }
        self.backend.put_user_settings(settings).await
    }

    fn get_headers_for(&self, anon_key: &str) -> Result<Headers, String> {
        if anon_key.is_empty() {
            return Err("Supabase anon key not configured".to_string());
        }
        let headers = Headers::new();
        headers.set("apikey", anon_key);
        headers.set("Content-Type", "application/json");
        Ok(headers)
    }

    fn get_headers(&self) -> Result<Headers, String> {
        let anon_key = self
            .anon_key
            .as_ref()
            .filter(|k| !k.is_empty())
            .ok_or_else(|| "Supabase anon key not configured".to_string())?;
        self.get_headers_for(anon_key)
    }

    fn get_auth_headers(&self, access_token: &str) -> Result<Headers, String> {
        let headers = self.get_headers()?;
        headers.set("Authorization", &format!("Bearer {}", access_token));
        Ok(headers)
    }

    pub async fn login(&self, request: LoginRequest) -> Result<Session, String> {
        if self.mode == AppMode::Backend {
            return self.backend.login(request).await;
        }
        if !self.is_configured() {
            return Err(
                "Authentication is not configured. Please set up Supabase credentials.".to_string(),
            );
        }
        self.password_grant(&self.auth_url(), request, None).await
    }

    /// Logs in against Supabase with a per-user anon key (Settings flow).
    /// `supabase_url` comes from the user's stored settings. In backend
    /// mode the Supabase token is exchanged for a backend session so
    /// authenticated backend APIs accept the saved session.
    pub async fn login_with_anon_key(
        &self,
        supabase_url: &str,
        request: LoginRequest,
        anon_key: &str,
    ) -> Result<Session, String> {
        if supabase_url.is_empty() {
            return Err("Supabase URL is not configured".to_string());
        }
        let session = self
            .password_grant(supabase_url, request, Some(anon_key))
            .await?;
        if self.mode != AppMode::Backend {
            return Ok(session);
        }
        self.backend.exchange_supabase_session(&session).await
    }

    async fn password_grant(
        &self,
        supabase_url: &str,
        request: LoginRequest,
        anon_key: Option<&str>,
    ) -> Result<Session, String> {
        let base = supabase_url.trim_end_matches('/');
        let url = format!("{base}/{AUTH_PATH}/token?grant_type=password");
        let body = serde_json::to_string(&request)
            .map_err(|e| format!("Failed to serialize request: {}", e))?;

        let headers = match anon_key {
            Some(key) => self.get_headers_for(key)?,
            None => self.get_headers()?,
        };

        let response = Request::post(&url)
            .headers(headers)
            .body(body)
            .map_err(|e| format!("Failed to build: {}", e))?
            .send()
            .await
            .map_err(|e| format!("Failed to send login request: {}", e))?;

        if !response.ok() {
            let msg = error_message(&response).await;
            return Err(format!("Login failed: {msg}"));
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
        if self.mode == AppMode::Backend {
            return self.backend.signup(request).await;
        }
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
            let msg = error_message(&response).await;
            return Err(format!("Signup failed: {msg}"));
        }

        let auth_response: AuthResponse = response
            .json::<AuthResponse>()
            .await
            .map_err(|e| format!("Failed to parse auth response: {}", e))?;

        auth_response.into_session()
    }

    pub async fn logout(&self, access_token: &str) -> Result<(), String> {
        if self.mode == AppMode::Backend {
            return self.backend.logout().await;
        }
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
            let msg = error_message(&response).await;
            return Err(format!("Logout failed: {msg}"));
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
            let msg = error_message(&response).await;
            return Err(format!("Get user failed: {msg}"));
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
            let msg = error_message(&response).await;
            return Err(format!("Refresh token failed: {msg}"));
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
