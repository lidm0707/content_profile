//! Hook for fetching and managing tags
//!
//! This hook provides a simple interface for fetching tags from Supabase.
//!
//! # Examples
//!
//! Basic usage:
//!
//! ```rust,ignore
//! use content_sdk::hooks::UseTags;
//! use supabase_client::{ClientConfig, client_config};
//! use dioxus::prelude::*;
//!
//! #[component]
//! fn TagList() -> Element {
//!     let config = client_config(
//!         "https://your-project.supabase.co".to_string(),
//!         "your-anon-key".to_string(),
//!     );
//!     let tags = UseTags::new(config);
//!
//!     rsx! {
//!         match tags.read() {
//!             Some(Ok(tags)) => rsx! {
//!                 for tag in tags {
//!                     div { "{tag.name}" }
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
//! let tags = UseTags::with_table(config.clone(), "custom_tags".to_string());
//! ```

use crate::models::Tag;
use dioxus::prelude::*;
use supabase_client::{ClientConfig, get};
use tracing::{debug, error, info};

const DEFAULT_TABLE_NAME: &str = "tags";

/// Hook for fetching and managing tags
///
/// This hook provides a reactive interface for fetching tags from Supabase.
/// It automatically refetches when dependencies change and provides methods
/// to manually refresh the data.
pub struct UseTags {
    /// Resource for fetching tags
    resource: Resource<Result<Vec<Tag>, String>>,
    /// Signal for filtering by tag IDs
    filter_ids: Signal<Option<Vec<i32>>>,
    /// Signal for search query
    search_query: Signal<Option<String>>,
    /// Supabase client config
    config: ClientConfig,
    /// Table name in Supabase
    table: String,
}

impl UseTags {
    /// Creates a new UseTags hook with default table name
    pub fn new(config: ClientConfig) -> Self {
        Self::with_table(config, DEFAULT_TABLE_NAME.to_string())
    }

    /// Creates a new UseTags hook with custom table name
    pub fn with_table(config: ClientConfig, table: String) -> Self {
        let table_clone = table.clone();
        let config_clone = config.clone();

        let resource = use_resource(move || {
            let config = config_clone.clone();
            let table = table_clone.clone();

            async move {
                debug!("Fetching tags from table: {}", table);

                match get::<Tag>(&config, &table, &[]).await {
                    Ok(tags) => {
                        info!("Successfully fetched {} tags", tags.len());
                        Ok(tags)
                    }
                    Err(e) => {
                        error!("Failed to fetch tags: {}", e);
                        Err(e)
                    }
                }
            }
        });

        Self {
            resource,
            filter_ids: use_signal(|| None),
            search_query: use_signal(|| None),
            config,
            table,
        }
    }

    /// Reads current tags value
    ///
    /// Returns:
    /// - `Some(Ok(tags))` if tags are successfully loaded
    /// - `Some(Err(e))` if there was an error
    /// - `None` if still loading
    pub fn read(&self) -> Option<Result<Vec<Tag>, String>> {
        self.resource.read().as_ref().cloned()
    }

    /// Refreshes tags by restarting fetch
    pub fn refresh(&mut self) {
        info!("Refreshing tags");
        self.resource.restart();
    }

    /// Sets ID filter
    pub fn set_id_filter(&mut self, ids: Option<Vec<i32>>) {
        info!("Setting ID filter: {:?}", ids);
        *self.filter_ids.write() = ids;
    }

    /// Gets current ID filter
    pub fn get_id_filter(&self) -> Option<Vec<i32>> {
        self.filter_ids.read().as_ref().cloned()
    }

    /// Sets the search query
    pub fn set_search_query(&mut self, query: Option<String>) {
        info!("Setting search query: {:?}", query);
        *self.search_query.write() = query;
    }

    /// Gets current search query
    pub fn get_search_query(&self) -> Option<String> {
        self.search_query.read().as_ref().cloned()
    }

    /// Gets filtered tags based on current filters
    ///
    /// Returns tags filtered by IDs and search query.
    /// Returns None if tags are still loading.
    pub fn get_filtered(&self) -> Option<Result<Vec<Tag>, String>> {
        match self.read() {
            Some(Ok(tags)) => {
                let mut filtered = tags;

                // Filter by IDs
                if let Some(filter_ids) = self.filter_ids.read().as_ref() {
                    if !filter_ids.is_empty() {
                        filtered = filtered
                            .into_iter()
                            .filter(|t| t.id.map_or(false, |id| filter_ids.contains(&id)))
                            .collect();
                    }
                }

                // Filter by search query
                if let Some(query) = self.search_query.read().as_ref() {
                    let query_lower = query.to_lowercase();
                    filtered = filtered
                        .into_iter()
                        .filter(|t| {
                            t.name.to_lowercase().contains(&query_lower)
                                || t.slug.to_lowercase().contains(&query_lower)
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

    /// Gets current tags without filters
    pub fn get_all(&self) -> Option<Result<Vec<Tag>, String>> {
        self.read()
    }

    /// Finds a tag by ID in the loaded tags
    pub fn find_by_id(&self, id: i32) -> Option<Tag> {
        match self.read() {
            Some(Ok(tags)) => tags.iter().find(|t| t.id == Some(id)).cloned(),
            _ => None,
        }
    }

    /// Finds a tag by slug in the loaded tags
    pub fn find_by_slug(&self, slug: &str) -> Option<Tag> {
        match self.read() {
            Some(Ok(tags)) => tags.iter().find(|t| t.slug == slug).cloned(),
            _ => None,
        }
    }

    /// Gets a single tag by ID from Supabase
    pub async fn get_by_id(&self, id: i32) -> Result<Option<Tag>, String> {
        use supabase_client::get_by;

        debug!("Fetching tag by id: {} from table: {}", id, self.table);
        let tags = get_by::<Tag>(&self.config, &self.table, "id", &id.to_string()).await?;
        Ok(tags.into_iter().next())
    }
}

/// Convenience function to create a UseTags hook
///
/// This is a shorthand for `UseTags::new(config)`
pub fn use_tags(config: ClientConfig) -> UseTags {
    UseTags::new(config)
}

/// Convenience function to create a UseTags hook with custom table
///
/// This is a shorthand for `UseTags::with_table(config, table)`
pub fn use_tags_with_table(config: ClientConfig, table: String) -> UseTags {
    UseTags::with_table(config, table)
}
