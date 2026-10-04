use chrono::{DateTime, Utc};
use sqlx::PgPool;

const SQL_UPSERT: &str = "INSERT INTO google_oauth_tokens \
    (username, access_token, refresh_token, expires_at, scope) \
    VALUES ($1, $2, $3, $4, $5) \
    ON CONFLICT (username) DO UPDATE SET \
    access_token = EXCLUDED.access_token, \
    refresh_token = COALESCE(EXCLUDED.refresh_token, google_oauth_tokens.refresh_token), \
    expires_at = EXCLUDED.expires_at, \
    scope = EXCLUDED.scope, \
    updated_at = now()";
const SQL_FIND: &str = "SELECT access_token, refresh_token, expires_at \
    FROM google_oauth_tokens WHERE username = $1";

/// One user's stored Google OAuth token pair.
#[derive(Debug, Clone)]
pub struct GoogleTokenRow {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_at: DateTime<Utc>,
}

#[derive(Clone)]
pub struct PgGoogleTokenStore {
    pool: PgPool,
}

impl PgGoogleTokenStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Saves a token set. The existing refresh token is kept when Google
    /// omits `refresh_token` from a refresh response (which it always does).
    pub async fn save(
        &self,
        username: &str,
        access_token: &str,
        refresh_token: Option<&str>,
        expires_at: DateTime<Utc>,
        scope: &str,
    ) -> Result<(), String> {
        sqlx::query(SQL_UPSERT)
            .bind(username)
            .bind(access_token)
            .bind(refresh_token)
            .bind(expires_at)
            .bind(scope)
            .execute(&self.pool)
            .await
            .map(|_| ())
            .map_err(|e| e.to_string())
    }

    pub async fn find(&self, username: &str) -> Result<Option<GoogleTokenRow>, String> {
        sqlx::query_as::<_, (String, Option<String>, DateTime<Utc>)>(SQL_FIND)
            .bind(username)
            .fetch_optional(&self.pool)
            .await
            .map(|row| {
                row.map(|(access_token, refresh_token, expires_at)| GoogleTokenRow {
                    access_token,
                    refresh_token,
                    expires_at,
                })
            })
            .map_err(|e| e.to_string())
    }
}
