//! Google Drive image upload — implementation lives in the `oauth_google` crate.
pub use oauth_google::{acquire_token_promise, preload_gdrive, upload_with_token};
