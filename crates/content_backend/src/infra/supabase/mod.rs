pub mod auth;
pub mod content;
pub mod content_tag;
#[cfg(not(target_arch = "wasm32"))]
pub mod sync;
pub mod tag;

pub use auth::SupabaseRemoteAuth;

use crate::port::http::HttpContext;

pub type SupabaseConfig = HttpContext;

/// Builds Supabase config from the centralized env config.
pub fn config_from_env() -> Option<SupabaseConfig> {
    crate::config::Config::from_env().supabase_config()
}
