use sqlx::PgPool;

const SQL_INSERT: &str = "INSERT INTO sessions (token, username) VALUES ($1, $2) \
    ON CONFLICT (token) DO UPDATE SET username = EXCLUDED.username, expires_at = now() + INTERVAL '7 days'";
const SQL_GET: &str = "SELECT username FROM sessions \
    WHERE token = $1 AND expires_at > now()";
const SQL_REMOVE: &str = "DELETE FROM sessions WHERE token = $1";

/// pg-backed session store: tokens survive server restarts, unlike the
/// previous in-memory map (which invalidated every cookie on rebuild).
#[derive(Clone)]
pub struct PgSessionStore {
    pool: PgPool,
}

impl PgSessionStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn insert(&self, token: &str, username: &str) -> Result<(), String> {
        sqlx::query(SQL_INSERT)
            .bind(token)
            .bind(username)
            .execute(&self.pool)
            .await
            .map(|_| ())
            .map_err(|e| e.to_string())
    }

    pub async fn get(&self, token: &str) -> Result<Option<String>, String> {
        sqlx::query_scalar(SQL_GET)
            .bind(token)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| e.to_string())
    }

    pub async fn remove(&self, token: &str) -> Result<(), String> {
        sqlx::query(SQL_REMOVE)
            .bind(token)
            .execute(&self.pool)
            .await
            .map(|_| ())
            .map_err(|e| e.to_string())
    }
}
