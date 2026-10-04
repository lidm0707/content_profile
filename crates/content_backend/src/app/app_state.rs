use sqlx::PgPool;

use crate::app::google_oauth_service::GoogleOauthService;
use crate::config::Config;
use crate::infra::pg::google_oauth::PgGoogleTokenStore;
use crate::infra::pg::session_store::PgSessionStore;
use crate::infra::pg::supabase_tokens::PgSupabaseTokenStore;
use crate::infra::pg::user_settings::PgSettingsStore;
use crate::infra::supabase::auth::SupabaseRemoteAuth;
use crate::infra::{Backend, BackendMode};

const ERR_GOOGLE_NO_CLIENT_ID: &str = "google oauth client id not set (GOOGLE_OAUTH_CLIENT_ID)";
const ERR_GOOGLE_NO_SECRET: &str =
    "google oauth client secret not set (GOOGLE_OAUTH_CLIENT_SECRET)";
const ERR_GOOGLE_NO_REDIRECT: &str =
    "google oauth redirect url not set (GOOGLE_OAUTH_REDIRECT_URL)";

#[derive(Clone)]
pub struct AppState {
    pub backend: Backend,
    pub sessions: PgSessionStore,
    pub pool: PgPool,
    pub settings: PgSettingsStore,
    pub supabase_tokens: PgSupabaseTokenStore,
    /// Env-only Google OAuth config; pg stores just the resulting tokens.
    google_client_id: Option<String>,
    google_client_secret: Option<String>,
    google_redirect_url: Option<String>,
    pub supabase_auth: SupabaseRemoteAuth,
}

impl AppState {
    pub fn new(backend: Backend, pool: PgPool) -> Self {
        let cfg = Config::from_env();
        Self::with_config(backend, pool, &cfg)
    }

    /// Env-only Google OAuth config; pg stores just the resulting tokens.
    fn with_config(backend: Backend, pool: PgPool, cfg: &Config) -> Self {
        Self {
            backend,
            sessions: PgSessionStore::new(pool.clone()),
            settings: PgSettingsStore::new(pool.clone()),
            supabase_tokens: PgSupabaseTokenStore::new(pool.clone()),
            google_client_id: cfg.google_oauth_client_id.clone(),
            google_client_secret: cfg.google_oauth_client_secret.clone(),
            google_redirect_url: cfg.google_oauth_redirect_url.clone(),
            supabase_auth: SupabaseRemoteAuth::new(cfg.supabase_config()),
            pool,
        }
    }

    /// Builds the Google OAuth service for one user at request time. The
    /// client ID, secret and redirect URL always come from env — pg only
    /// stores the tokens after the flow completes.
    pub async fn google_service(&self, _username: &str) -> Result<GoogleOauthService, String> {
        Ok(GoogleOauthService::new(
            self.google_client_id
                .clone()
                .ok_or_else(|| ERR_GOOGLE_NO_CLIENT_ID.to_string())?,
            self.google_client_secret
                .clone()
                .ok_or_else(|| ERR_GOOGLE_NO_SECRET.to_string())?,
            self.google_redirect_url
                .clone()
                .ok_or_else(|| ERR_GOOGLE_NO_REDIRECT.to_string())?,
            PgGoogleTokenStore::new(self.pool.clone()),
        ))
    }

    /// Builds the full app state from env config: pool, migrations, backend.
    pub async fn from_config(cfg: &Config) -> Result<Self, String> {
        let pool = crate::infra::pg::pool_from_env();
        run_migrations(&pool).await?;
        let backend = Backend::new(BackendMode::parse(&cfg.backend_mode), cfg.supabase_config());
        Ok(Self::new(backend, pool))
    }
}

async fn run_migrations(pool: &sqlx::PgPool) -> Result<(), String> {
    sqlx::migrate!("./migrations")
        .run(pool)
        .await
        .map_err(|e| e.to_string())
}
