//! Hook for fetching and managing content
//!
//! This hook provides a simple interface for fetching content from Supabase.
//!
//! # Examples
//!
//! Basic usage:
//!
//! ```rust
//! use content_sdk::hooks::UseContent;
//! use supabase_client::{ClientConfig, client_config};
//! use dioxus::prelude::*;
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
//!     rsx! {
//!         match content.read() {
//!             Some(Ok(items)) => rsx! {
//!                 for item in items {
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
//! With custom table name:
//!
//! ```rust,ignore
//! let content = UseContent::with_table(config(), "custom_content".to_string());
//! ```

use crate::models::Content;
use dioxus::prelude::*;
use supabase_client::{ClientConfig, get};
use tracing::{debug, error, info};

const DEFAULT_TABLE_NAME: &str = "content";

/// Hook for fetching and managing content
///
/// This hook provides a reactive interface for fetching content from Supabase.
/// It automatically refetches when dependencies change and provides methods
/// to manually refresh the data.
pub struct UseContent {
    /// Resource for fetching content
    resource: Resource<Result<Vec<Content>, String>>,
    /// Signal for filtering by status
    filter_status: Signal<Option<String>>,
    /// Signal for search query
    search_query: Signal<Option<String>>,
    /// Supabase client config
    config: ClientConfig,
    /// Table name in Supabase
    table: String,
}

impl UseContent {
    /// Creates a new UseContent hook with default table name
    pub fn new(config: ClientConfig) -> Self {
        Self::with_table(config, DEFAULT_TABLE_NAME.to_string())
    }

    /// Creates a new UseContent hook with custom table name
    pub fn with_table(config: ClientConfig, table: String) -> Self {
        let table_clone = table.clone();
        let config_clone = config.clone();

        let resource = use_resource(move || {
            let config = config_clone.clone();
            let table = table_clone.clone();

            async move {
                debug!("Fetching content from table: {}", table);

                match get::<Content>(&config, &table, &[]).await {
                    Ok(items) => {
                        info!("Successfully fetched {} content items", items.len());
                        Ok(items)
                    }
                    Err(e) => {
                        error!("Failed to fetch content: {}", e);
                        Err(e)
                    }
                }
            }
        });

        Self {
            resource,
            filter_status: use_signal(|| None),
            search_query: use_signal(|| None),
            config,
            table,
        }
    }

    /// Reads current content value
    ///
    /// Returns:
    /// - `Some(Ok(items))` if content are successfully loaded
    /// - `Some(Err(e))` if there was an error
    /// - `None` if still loading
    pub fn read(&self) -> Option<Result<Vec<Content>, String>> {
        self.resource.read().as_ref().cloned()
    }

    /// Refreshes content by restarting the fetch
    pub fn refresh(&mut self) {
        info!("Refreshing content");
        self.resource.restart();
    }

    /// Sets status filter
    pub fn set_status_filter(&mut self, status: Option<String>) {
        info!("Setting status filter: {:?}", status);
        *self.filter_status.write() = status;
    }

    /// Gets current status filter
    pub fn get_status_filter(&self) -> Option<String> {
        self.filter_status.read().clone()
    }

    /// Sets search query
    pub fn set_search_query(&mut self, query: Option<String>) {
        info!("Setting search query: {:?}", query);
        *self.search_query.write() = query;
    }

    /// Gets current search query
    pub fn get_search_query(&self) -> Option<String> {
        self.search_query.read().as_ref().cloned()
    }

    /// Gets filtered content based on current filters
    ///
    /// Returns content filtered by status, tags, content type, and search query.
    /// Returns None if content is still loading.
    pub fn get_filtered(&self) -> Option<Result<Vec<Content>, String>> {
        match self.read() {
            Some(Ok(items)) => {
                let mut filtered = items;

                // Filter by status
                if let Some(status) = self.filter_status.read().as_ref() {
                    filtered = filtered
                        .into_iter()
                        .filter(|c| c.status == *status)
                        .collect();
                }

                // Filter by search query
                if let Some(query) = self.search_query.read().as_ref() {
                    let query_lower = query.to_lowercase();
                    filtered = filtered
                        .into_iter()
                        .filter(|c| {
                            c.title.to_lowercase().contains(&query_lower)
                                || c.body.to_lowercase().contains(&query_lower)
                        })
                        .collect();
                }

                Some(Ok(filtered))
            }
            Some(Err(e)) => Some(Err(e.clone())),
            None => None,
        }
    }

    /// Checks if currently loading
    pub fn is_loading(&self) -> bool {
        self.resource.read().is_none()
    }

    /// Gets current content without filters
    pub fn get_all(&self) -> Option<Result<Vec<Content>, String>> {
        self.read()
    }

    /// Gets a single content item by ID from Supabase
    pub async fn get_by_id(&self, id: i32) -> Result<Option<Content>, String> {
        use supabase_client::get_by;

        debug!("Fetching content by id: {} from table: {}", id, self.table);
        let items = get_by::<Content>(&self.config, &self.table, "id", &id.to_string()).await?;
        Ok(items.into_iter().next())
    }
}

/// Convenience function to create a UseContent hook
///
/// This is a shorthand for `UseContent::new(config)`
pub fn use_content(config: ClientConfig) -> UseContent {
    UseContent::new(config)
}

/// Convenience function to create a UseContent hook with custom table
///
/// This is a shorthand for `UseContent::with_table(config, table)`
pub fn use_content_with_table(config: ClientConfig, table: String) -> UseContent {
    UseContent::with_table(config, table)
}
