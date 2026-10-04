use crate::models::{LoginRequest, Session};
use crate::services::{AuthService, SessionStorage};
use crate::utils::config::Config;
use dioxus::prelude::*;

/// User context for managing authentication state across the app
#[derive(Clone)]
pub struct UserContext {
    auth_service: Signal<AuthService>,
}

impl UserContext {
    /// Creates a new UserContext
    pub fn new(config: Option<Config>) -> Self {
        UserContext {
            auth_service: Signal::new(AuthService::new(config)),
        }
    }

    /// Updates the JWT token used for backend API calls.
    pub fn update_jwt_token(&mut self, jwt_token: Option<String>) {
        self.auth_service.write().update_jwt_token(jwt_token);
    }

    /// Logs in a user with email and password
    pub async fn login(&self, email: String, password: String) -> Result<Session, String> {
        let request = LoginRequest { email, password };
        let session = self.auth_service().login(request).await?;
        self.persist_session(&session);
        Ok(session)
    }

    /// Logs in directly against Supabase using a per-user anon key and
    /// the per-user Supabase URL stored in settings
    pub async fn login_supabase(
        &self,
        supabase_url: String,
        email: String,
        password: String,
        anon_key: String,
    ) -> Result<Session, String> {
        let request = LoginRequest { email, password };
        let session = self
            .auth_service()
            .login_with_anon_key(&supabase_url, request, &anon_key)
            .await?;
        self.persist_session(&session);
        Ok(session)
    }

    /// Signs up a new user
    pub async fn signup(&self, email: String, password: String) -> Result<Session, String> {
        let request = LoginRequest { email, password };
        let session = self.auth_service().signup(request).await?;
        self.persist_session(&session);
        Ok(session)
    }

    /// Logs out the current user
    pub async fn logout(&self) -> Result<(), String> {
        // Backend mode: the session cookie authenticates the logout call.
        if self.is_backend_mode() {
            let _ = self.auth_service().logout("").await;
        } else if let Ok(Some(session)) = Self::load_saved_session() {
            let _ = self.auth_service().logout(&session.access_token).await;
        }
        SessionStorage::clear_session()
    }

    /// Recovers the session after a page reload in backend mode: the
    /// HttpOnly cookie identifies the user, nothing token-shaped is read
    /// from or written to storage.
    pub async fn restore_cookie_session(&self) -> Option<Session> {
        self.auth_service()
            .session_from_cookie()
            .await
            .ok()
            .flatten()
    }

    /// Backend mode keeps tokens server-side (pg) — the browser only
    /// holds the in-memory session. Other modes persist to localStorage.
    fn persist_session(&self, session: &Session) {
        if !self.is_backend_mode() {
            let _ = SessionStorage::save_session(session);
        }
    }

    pub fn is_backend_mode(&self) -> bool {
        self.auth_service.read().is_backend_mode()
    }

    /// Loads the saved session from storage
    pub fn load_saved_session() -> Result<Option<Session>, String> {
        SessionStorage::load_session()
    }

    /// Clears the saved session from storage
    pub fn clear_saved_session() -> Result<(), String> {
        SessionStorage::clear_session()
    }

    /// Checks if a saved session is valid
    pub fn has_valid_saved_session() -> bool {
        if let Ok(Some(session)) = Self::load_saved_session() {
            let now = chrono::Utc::now().timestamp();
            now < session.expires_at
        } else {
            false
        }
    }

    /// Returns the saved session, refreshing it when expired (or about to
    /// expire). Only clears the stored session if the refresh fails.
    pub async fn restore_saved_session(&self) -> Option<Session> {
        const EXPIRY_SKEW_SECS: i64 = 30;

        let session = Self::load_saved_session().ok().flatten()?;
        let now = chrono::Utc::now().timestamp();
        if now < session.expires_at - EXPIRY_SKEW_SECS {
            return Some(session);
        }
        if session.refresh_token.is_empty() {
            let _ = Self::clear_saved_session();
            return None;
        }
        match self
            .auth_service()
            .refresh_token(&session.refresh_token)
            .await
        {
            Ok(fresh) => {
                let _ = SessionStorage::save_session(&fresh);
                Some(fresh)
            }
            Err(_) => {
                let _ = Self::clear_saved_session();
                None
            }
        }
    }

    /// Clones the current auth service to avoid holding a read
    /// guard across await points.
    fn auth_service(&self) -> AuthService {
        self.auth_service.read().clone()
    }

    /// Checks if authentication (Supabase) is configured
    pub fn is_configured(&self) -> bool {
        self.auth_service.read().is_configured()
    }

    /// Loads this user's stored settings from the backend
    pub async fn load_settings(&self) -> Result<serde_json::Value, String> {
        self.auth_service().load_settings().await
    }

    /// Saves this user's settings on the backend
    pub async fn save_settings(&self, settings: serde_json::Value) -> Result<(), String> {
        self.auth_service().save_settings(settings).await
    }
}

impl Default for UserContext {
    fn default() -> Self {
        Self::new(None)
    }
}
