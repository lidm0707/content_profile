use async_trait::async_trait;
use serde_json::Value;
use sqlx::PgPool;

use crate::port::SettingsStore;

const SQL_UPSERT: &str = "INSERT INTO user_settings (username, settings) VALUES ($1, $2) \
    ON CONFLICT (username) DO UPDATE SET settings = EXCLUDED.settings, updated_at = now()";
const SQL_FIND: &str = "SELECT settings FROM user_settings WHERE username = $1";

#[derive(Clone)]
pub struct PgSettingsStore {
    pool: PgPool,
}

impl PgSettingsStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl SettingsStore for PgSettingsStore {
    async fn load(&self, username: &str) -> Result<Value, String> {
        sqlx::query_scalar::<_, Value>(SQL_FIND)
            .bind(username)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| e.to_string())
            .map(|row| row.unwrap_or_else(default_settings))
    }

    async fn save(&self, username: &str, settings: &Value) -> Result<(), String> {
        sqlx::query(SQL_UPSERT)
            .bind(username)
            .bind(settings)
            .execute(&self.pool)
            .await
            .map(|_| ())
            .map_err(|e| e.to_string())
    }
}

fn default_settings() -> Value {
    serde_json::json!({})
}
