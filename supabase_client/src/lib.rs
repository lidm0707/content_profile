//! Supabase client library for content management
//!
//! This library provides a client for interacting with Supabase's REST API
//! to manage content items.

pub mod client;
pub mod config;
pub mod models;

pub use client::SupabaseClient;
pub use config::{ClientConfig, ClientConfigBuilder};
pub use models::status;
pub use models::{Content, ContentRequest};
