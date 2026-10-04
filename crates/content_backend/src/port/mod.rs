pub mod auth_port;
pub mod database_port;
pub mod http;

pub use auth_port::{PasswordHasher, RemoteAuth, SettingsStore, TokenIssuer, UserStore};
pub use database_port::{ContentRepo, ContentTagRepo, TagRepo};

/// gloo/js futures are !Send on wasm32, so the Send bound only applies on native.
#[cfg(not(target_arch = "wasm32"))]
pub trait NativeSend: Sync {}
#[cfg(not(target_arch = "wasm32"))]
impl<T: Sync + ?Sized> NativeSend for T {}
#[cfg(target_arch = "wasm32")]
pub trait NativeSend {}
#[cfg(target_arch = "wasm32")]
impl<T: ?Sized> NativeSend for T {}
