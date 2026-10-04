pub mod content;
pub mod google_oauth;
pub mod session_store;
pub mod supabase_tokens;
pub mod tag;
pub mod user_settings;
pub mod user_store;

use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;

const MAX_CONNECTIONS: u32 = 5;
const ERR_DATABASE_URL: &str = "invalid DATABASE_URL";

pub(crate) fn pool_from_env() -> PgPool {
    let url = crate::config::Config::from_env().database_url;
    PgPoolOptions::new()
        .max_connections(MAX_CONNECTIONS)
        .connect_lazy(&url)
        .expect(ERR_DATABASE_URL)
}
