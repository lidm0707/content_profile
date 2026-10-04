use chrono::{DateTime, Duration, Utc};
use sqlx::PgPool;

const DEFAULT_TTL_SECS: i64 = 3600;

const SQL_UPSERT: &str = "INSERT INTO supabase_tokens \
    (username, access_token, refresh_token, expires_at) \
    VALUES ($1, $2, $3, $4) \
    ON CONFLICT (username) DO UPDATE SET \
    access_token = EXCLUDED.access_token, \
    refresh_token = COALESCE(EXCLUDED.refresh_token, supabase_tokens.refresh_token), \
    expires_at = EXCLUDED.expires_at, \
    updated_at = now()";
const SQL_FIND: &str = "SELECT access_token, refresh_token, expires_at \
    FROM supabase_tokens WHERE username = $1";

/// One user's stored Supabase auth token pair.
#[derive(Debug, Clone)]
pub struct SupabaseTokenRow {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_at: DateTime<Utc>,
}

#[derive(Clone)]
pub struct PgSupabaseTokenStore {
    pool: PgPool,
}

impl PgSupabaseTokenStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn save(
        &self,
        username: &str,
        access_token: &str,
        refresh_token: Option<&str>,
        expires_at: DateTime<Utc>,
    ) -> Result<(), String> {
        sqlx::query(SQL_UPSERT)
            .bind(username)
            .bind(access_token)
            .bind(refresh_token)
            .bind(expires_at)
            .execute(&self.pool)
            .await
            .map(|_| ())
            .map_err(|e| e.to_string())
    }

    pub async fn find(&self, username: &str) -> Result<Option<SupabaseTokenRow>, String> {
        sqlx::query_as::<_, (String, Option<String>, DateTime<Utc>)>(SQL_FIND)
            .bind(username)
            .fetch_optional(&self.pool)
            .await
            .map(|row| {
                row.map(
                    |(access_token, refresh_token, expires_at)| SupabaseTokenRow {
                        access_token,
                        refresh_token,
                        expires_at,
                    },
                )
            })
            .map_err(|e| e.to_string())
    }
}

/// Builds the expiry timestamp from a GoTrue `expires_in`-style value.
pub fn expires_from_ttl(expires_in: Option<i64>) -> DateTime<Utc> {
    Utc::now() + Duration::seconds(expires_in.unwrap_or(DEFAULT_TTL_SECS))
}
