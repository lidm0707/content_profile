//! Content SDK Hooks
//!
//! This module provides reusable hooks for fetching and managing content.
//!
//! # Examples
//!
//! ```rust
//! use content_sdk::hooks::use_content;
//!
//! #[component]
//! fn ContentList() -> Element {
//!     let content = use_content();
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
pub mod use_tags;

// Re-export commonly used hooks for convenience
pub use use_content::UseContent;
pub use use_tags::UseTags;
