//! Pure Supabase client functions for content management
//!
//! This library provides pure async functions to interact with Supabase's REST API.

pub mod client;
pub mod config;

pub use client::*;
pub use config::{ClientConfig, client_config};
