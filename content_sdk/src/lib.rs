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
//! use content_sdk::hooks::UseBlogs;
//! use dioxus::prelude::*;
//!
//! #[component]
//! fn BlogPage() -> Element {
//!     let blogs = UseBlogs::new();
//!
//!     rsx! {
//!         match blogs.read() {
//!             Some(Ok(blog_list)) => rsx! {
//!                 for blog in blog_list {
//!                     div { "{blog.title}" }
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
//! - [`UseBlogs`] - Hook for fetching and filtering blogs
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
