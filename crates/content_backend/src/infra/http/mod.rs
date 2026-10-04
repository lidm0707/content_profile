#[cfg(not(target_arch = "wasm32"))]
pub mod api;
#[cfg(not(target_arch = "wasm32"))]
pub mod auth;
pub mod handler;
#[cfg(not(target_arch = "wasm32"))]
pub mod routes;
#[cfg(not(target_arch = "wasm32"))]
pub mod server;

pub use handler::SupabaseHttp;

#[cfg(not(target_arch = "wasm32"))]
pub use auth::AuthUser;
#[cfg(not(target_arch = "wasm32"))]
pub use server::{AppState, router, serve};
