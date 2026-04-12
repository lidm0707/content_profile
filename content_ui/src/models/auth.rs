use dioxus::prelude::*;
use serde::{Deserialize, Serialize};
/// User information from Supabase Auth
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Props)]
pub struct User {
    pub id: String,
    pub email: String,
    pub email_confirmed_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub last_sign_in_at: Option<String>,
}

/// Session information from Supabase Auth
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Props)]
pub struct Session {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_at: i64,
    pub token_type: String,
    pub user: User,
}

/// Login request payload
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Props)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

/// Supabase Auth API response for login/signup
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Props)]
pub struct AuthResponse {
    pub access_token: Option<String>,
    pub refresh_token: Option<String>,
    pub expires_at: Option<i64>,
    pub token_type: Option<String>,
    pub user: User,
}

/// Error response from Supabase Auth
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Props)]
pub struct AuthError {
    pub error: String,
    pub error_description: String,
}

impl AuthResponse {
    /// Converts AuthResponse to Session
    /// Returns Ok(Session) if tokens are present, Err if tokens are missing (email confirmation required)
    pub fn into_session(self) -> Result<Session, String> {
        let access_token = self.access_token
            .ok_or_else(|| "Email confirmation required. Please check your email to confirm your account before logging in.".to_string())?;

        let refresh_token = self.refresh_token
            .ok_or_else(|| "Email confirmation required. Please check your email to confirm your account before logging in.".to_string())?;

        let expires_at = self.expires_at
            .ok_or_else(|| "Email confirmation required. Please check your email to confirm your account before logging in.".to_string())?;

        let token_type = self.token_type
            .ok_or_else(|| "Email confirmation required. Please check your email to confirm your account before logging in.".to_string())?;

        Ok(Session {
            access_token,
            refresh_token,
            expires_at,
            token_type,
            user: self.user,
        })
    }
}
