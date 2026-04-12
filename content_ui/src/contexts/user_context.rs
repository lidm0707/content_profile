use crate::models::{LoginRequest, Session};
use crate::services::{AuthService, SessionStorage};
use std::sync::Arc;

/// User context for managing authentication state across the app
#[derive(Clone)]
pub struct UserContext {
    auth_service: Arc<AuthService>,
}

impl UserContext {
    /// Creates a new UserContext
    pub fn new() -> Self {
        UserContext {
            auth_service: Arc::new(AuthService::new()),
        }
    }

    /// Gets the auth service (for internal use)
    pub fn auth_service(&self) -> &AuthService {
        &self.auth_service
    }

    /// Logs in a user with email and password
    pub async fn login(&self, email: String, password: String) -> Result<Session, String> {
        let request = LoginRequest { email, password };
        let session = self.auth_service.login(request).await?;
        SessionStorage::save_session(&session)?;
        Ok(session)
    }

    /// Signs up a new user
    pub async fn signup(&self, email: String, password: String) -> Result<Session, String> {
        let request = LoginRequest { email, password };
        let session = self.auth_service.signup(request).await?;
        SessionStorage::save_session(&session)?;
        Ok(session)
    }

    /// Logs out the current user
    pub async fn logout(&self) -> Result<(), String> {
        if let Ok(Some(session)) = Self::load_saved_session() {
            let _ = self.auth_service.logout(&session.access_token).await;
        }
        SessionStorage::clear_session()
    }

    /// Refreshes the access token
    pub async fn refresh_token(
        &self,
        refresh_token: &str,
    ) -> Result<crate::models::Session, String> {
        let session = self.auth_service.refresh_token(refresh_token).await?;
        SessionStorage::save_session(&session)?;
        Ok(session)
    }

    /// Gets the current user from the server
    pub async fn get_user(&self, access_token: &str) -> Result<crate::models::User, String> {
        self.auth_service.get_user(access_token).await
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

    /// Checks if authentication (Supabase) is configured
    pub fn is_configured(&self) -> bool {
        self.auth_service.is_configured()
    }
}

impl Default for UserContext {
    fn default() -> Self {
        Self::new()
    }
}
