//! Content SDK - A reusable Dioxus SDK for content management
//!
//! This SDK provides hooks and models for building content-focused
//! applications with Dioxus and Tailwind CSS.
//!
//! # Features
//!
//! - **Models**: Data structures for blogs, content, and tags with serialization support
//! - **Hooks**: Reusable hooks for fetching and managing content with filtering and search
//!
//! # Quick Start
//!
//! ## Using Hooks
//!
//! ```rust
//! use content_sdk::hooks::UseContent;
//! use content_sdk::models::Content;
//! use dioxus::prelude::*;
//! use supabase_client::client_config;
//!
//! #[component]
//! fn ContentPage() -> Element {
//!     let config = client_config(
//!         "https://your-project.supabase.co".to_string(),
//!         "your-anon-key".to_string(),
//!     );
//!     let content = UseContent::new(config);
//!
//!     rsx! {
//!         match content.read() {
//!             Some(Ok(content_list)) => rsx! {
//!                 for item in content_list {
//!                     div { "{item.title}" }
//!                 }
//!             },
//!             Some(Err(e)) => rsx! { div { "Error: {e}" } },
//!             None => rsx! { div { "Loading..." } },
//!         }
//!     }
//! }
//! ```
//!
//! # Module Organization
//!
//! ## Models
//!
//! Data structures for content with serialization support:
//! - [`Blog`] - Blog post model with status, author, category
//! - [`Content`] - General content model
//! - [`Tag`] - Tag model with color and count
//!
//! ## Hooks
//!
//! Reusable hooks for data fetching and state management:
//! - [`UseContent`] - Hook for fetching and filtering content
//! - [`UseTags`] - Hook for fetching and filtering tags
//!
//! # Customization
//!
//! All hooks support extensive customization:
//! - Custom API endpoints for data fetching
//! - Custom filtering and search parameters
//! - Custom callbacks for user interactions
//!
//! # Styling
//!
//! The SDK assumes Tailwind CSS v4 is configured in your project.

// Re-export models
pub mod models;

// Re-export hooks
pub mod hooks;

// Re-export commonly used models for convenience
pub use models::{
    content::{Content, ContentRequest, ContentStatus},
    tag::{Tag, TagRequest},
};

// Re-export commonly used hooks for convenience
pub use hooks::{UseContent, UseTags, use_content, use_tags};
