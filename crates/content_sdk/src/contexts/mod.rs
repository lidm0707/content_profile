//! Context module for managing global state
//!
//! This module provides context providers for managing application state across
//! the component tree, including content, tags, and user authentication.

pub mod content_context;
pub mod content_tags_context;
pub mod tag_context;
pub mod user_context;

// Re-export contexts for convenience
pub use content_context::ContentContext;
pub use content_tags_context::ContentTagsContext;
pub use tag_context::TagContext;
pub use user_context::UserContext;
