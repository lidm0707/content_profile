//! Content SDK Hooks
//!
//! This module provides reusable hooks for fetching and managing content.
//!
//! # Examples
//!
//! ```rust
//! use content_sdk::hooks::UseContent;
//! use dioxus::prelude::*;
//! use supabase_client::{ClientConfig, client_config};
//!
//! #[component]
//! fn ContentList() -> Element {
//!     let config = use_signal(|| {
//!         client_config(
//!             "https://your-project.supabase.co".to_string(),
//!             "your-anon-key".to_string(),
//!         )
//!     });
//!     let content = UseContent::new(config());
//!
//!     match content.read() {
//!         Some(Ok(items)) => rsx! {
//!             for item in items {
//!                 div { "{item.title}" }
//!             }
//!         },
//!         Some(Err(e)) => rsx! { div { "Error: {e}" } },
//!         None => rsx! { div { "Loading..." } },
//!     }
//! }
//! ```

pub mod use_content;
pub mod use_content_tags;
pub mod use_tags;
// Re-export commonly used hooks for convenience
pub use use_content::UseContent;
pub use use_content_tags::UseContentTags;
pub use use_tags::UseTags;
