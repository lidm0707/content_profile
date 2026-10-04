use std::net::SocketAddr;

const ENV_HTTP_ADDR: &str = "HTTP_ADDR";
const ENV_BACKEND_MODE: &str = "BACKEND_MODE";
const ENV_DATABASE_URL: &str = "DATABASE_URL";
const ENV_SUPABASE_URL: &str = "SUPABASE_URL";
const ENV_SUPABASE_ANON_KEY: &str = "SUPABASE_ANON_KEY";
const ENV_GOOGLE_OAUTH_CLIENT_ID: &str = "GOOGLE_OAUTH_CLIENT_ID";
const ENV_GOOGLE_OAUTH_CLIENT_SECRET: &str = "GOOGLE_OAUTH_CLIENT_SECRET";
const ENV_GOOGLE_OAUTH_REDIRECT_URL: &str = "GOOGLE_OAUTH_REDIRECT_URL";

const DEFAULT_ADDR: &str = "0.0.0.0:8080";
const DEFAULT_MODE: &str = "supabase";

#[derive(Debug, Clone)]
pub struct Config {
    pub http_addr: SocketAddr,
    pub backend_mode: String,
    pub database_url: String,
    pub supabase_url: Option<String>,
    pub supabase_anon_key: Option<String>,
    pub google_oauth_client_id: Option<String>,
    pub google_oauth_client_secret: Option<String>,
    pub google_oauth_redirect_url: Option<String>,
}

impl Config {
    /// Loads all backend config from process env (with .env fallback).
    pub fn from_env() -> Self {
        #[cfg(not(target_arch = "wasm32"))]
        dotenvy::dotenv().ok();
        Self {
            http_addr: env_or(ENV_HTTP_ADDR, DEFAULT_ADDR)
                .parse()
                .unwrap_or_else(|_| panic!("invalid {ENV_HTTP_ADDR}, expected SocketAddr")),
            backend_mode: env_or(ENV_BACKEND_MODE, DEFAULT_MODE),
            database_url: std::env::var(ENV_DATABASE_URL).expect("missing DATABASE_URL"),
            supabase_url: std::env::var(ENV_SUPABASE_URL).ok(),
            supabase_anon_key: std::env::var(ENV_SUPABASE_ANON_KEY).ok(),
            google_oauth_client_id: non_empty(ENV_GOOGLE_OAUTH_CLIENT_ID),
            google_oauth_client_secret: non_empty(ENV_GOOGLE_OAUTH_CLIENT_SECRET),
            google_oauth_redirect_url: non_empty(ENV_GOOGLE_OAUTH_REDIRECT_URL),
        }
    }

    pub fn supabase_config(&self) -> Option<crate::infra::supabase::SupabaseConfig> {
        let url = self.supabase_url.clone()?;
        let key = self.supabase_anon_key.clone()?;
        Some(crate::infra::supabase::SupabaseConfig::new(url, key))
    }
}

fn env_or(key: &str, default: &str) -> String {
    std::env::var(key).unwrap_or_else(|_| default.to_string())
}

fn non_empty(key: &str) -> Option<String> {
    std::env::var(key).ok().filter(|v| !v.is_empty())
}
