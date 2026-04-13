use content_sdk::models::{LoginRequest, Session};
use content_sdk::services::{AuthService, SessionStorage};
use content_sdk::utils::config::Config;

/// User context for managing authentication state across the app
#[derive(Clone)]
pub struct UserContext {
    auth_service: AuthService,
}

impl UserContext {
    /// Creates a new UserContext
    pub fn new(config: Option<Config>) -> Self {
        UserContext {
            auth_service: AuthService::new(config),
        }
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
        Self::new(None)
    }
}
