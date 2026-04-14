//! Hook for fetching and managing content tags
//!
//! This hook provides a reactive interface for fetching tags assigned to specific content.
//! It fetches from the content_tags junction table and returns actual Tag objects.
//!
//! # Examples
//!
//! Basic usage:
//!
//! ```rust,ignore
//! use content_sdk::hooks::UseContentTags;
//! use supabase_client::{ClientConfig, client_config};
//! use dioxus::prelude::*;
//!
//! #[component]
//! fn ContentTagDisplay(content_id: i32) -> Element {
//!     let config = client_config(
//!         "https://your-project.supabase.co".to_string(),
//!         "your-anon-key".to_string(),
//!     );
//!     let tags = UseContentTags::new(config, content_id);
//!
//!     match tags.read() {
//!         Some(Ok(tags)) => rsx! {
//!             for tag in tags {
//!                 div { "{tag.name}" }
//!             }
//!         },
//!         Some(Err(e)) => rsx! { div { "Error: {e}" } },
//!         None => rsx! { div { "Loading..." } },
//!     }
//! }
//! ```
//!
//! With custom table names:
//!
//! ```rust,ignore
//! let tags = UseContentTags::with_tables(
//!     config.clone(),
//!     content_id,
//!     "custom_content_tags".to_string(),
//!     "custom_tags".to_string(),
//! );
//! ```
//!
//! Updating content_id:
//!
//! ```rust,ignore
//! let mut tags = UseContentTags::new(config, initial_content_id);
//!
//! button {
//!     onclick: move |_| {
//!         tags.set_content_id(new_content_id);
//!     },
//!     "Load tags for different content"
//! }
//! ```

use crate::models::{ContentTag, Tag};
use dioxus::prelude::*;
use supabase_client::{ClientConfig, get};
use tracing::{debug, error, info};

const DEFAULT_CONTENT_TAGS_TABLE: &str = "content_tags";
const DEFAULT_TAGS_TABLE: &str = "tags";

/// Hook for fetching and managing content tags
///
/// This hook provides a reactive interface for fetching tags assigned to a content item.
/// It queries the content_tags junction table and returns actual Tag objects.
pub struct UseContentTags {
    /// Resource for fetching tags
    resource: Resource<Result<Vec<Tag>, String>>,
    /// Signal for the content ID to fetch tags for
    content_id: Signal<i32>,
    /// Signal for filtering by tag IDs
    filter_ids: Signal<Option<Vec<i32>>>,
    /// Signal for search query
    search_query: Signal<Option<String>>,
    /// Supabase client config
    config: ClientConfig,
    /// Content tags table name (junction table)
    content_tags_table: String,
    /// Tags table name
    tags_table: String,
}

impl UseContentTags {
    /// Creates a new UseContentTags hook with default table names
    pub fn new(config: ClientConfig, content_id: i32) -> Self {
        Self::with_tables(
            config,
            content_id,
            DEFAULT_CONTENT_TAGS_TABLE.to_string(),
            DEFAULT_TAGS_TABLE.to_string(),
        )
    }

    /// Creates a new UseContentTags hook with custom table names
    pub fn with_tables(
        config: ClientConfig,
        content_id: i32,
        content_tags_table: String,
        tags_table: String,
    ) -> Self {
        let content_id_signal = use_signal(|| content_id);
        let content_tags_table_clone = content_tags_table.clone();
        let tags_table_clone = tags_table.clone();
        let config_clone = config.clone();

        let resource = use_resource(move || {
            let config = config_clone.clone();
            let current_content_id = *content_id_signal.read();
            let content_tags_table = content_tags_table_clone.clone();
            let tags_table = tags_table_clone.clone();

            async move {
                if current_content_id == 0 {
                    debug!("Creating new content - no content_tags to fetch");
                    return Ok(Vec::new());
                }

                debug!(
                    "Fetching content_tags for content_id: {} from table: {}",
                    current_content_id, content_tags_table
                );

                // Fetch content_tags (junction table records)
                let content_tags: Vec<ContentTag> = match get(
                    &config,
                    &content_tags_table,
                    &[("content_id", &current_content_id.to_string())],
                )
                .await
                {
                    Ok(tags) => {
                        info!(
                            "Successfully fetched {} content_tag records for content {}",
                            tags.len(),
                            current_content_id
                        );
                        tags
                    }
                    Err(e) => {
                        error!(
                            "Failed to fetch content_tags for content {}: {}",
                            current_content_id, e
                        );
                        return Err(e);
                    }
                };

                // Extract tag_ids from content_tags
                let tag_ids: Vec<i32> = content_tags.iter().map(|ct| ct.tag_id).collect();

                debug!("Extracted tag_ids: {:?}", tag_ids);

                if tag_ids.is_empty() {
                    info!("No tags assigned to content {}", current_content_id);
                    return Ok(Vec::new());
                }

                // Fetch all tags
                let all_tags: Vec<Tag> = match get(&config, &tags_table, &[]).await {
                    Ok(tags) => {
                        info!("Successfully fetched {} tags from tags table", tags.len());
                        tags
                    }
                    Err(e) => {
                        error!("Failed to fetch tags: {}", e);
                        return Err(e);
                    }
                };

                // Filter tags to only include those in tag_ids
                let filtered_tags: Vec<Tag> = all_tags
                    .into_iter()
                    .filter(|tag| tag.id.map_or(false, |id| tag_ids.contains(&id)))
                    .collect();

                info!(
                    "Returning {} tags for content {}",
                    filtered_tags.len(),
                    current_content_id
                );

                Ok(filtered_tags)
            }
        });

        Self {
            resource,
            content_id: content_id_signal,
            filter_ids: use_signal(|| None),
            search_query: use_signal(|| None),
            config,
            content_tags_table,
            tags_table,
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
        info!(
            "Refreshing content tags for content_id: {}",
            *self.content_id.read()
        );
        self.resource.restart();
    }

    /// Sets the content ID to fetch tags for
    pub fn set_content_id(&mut self, content_id: i32) {
        info!("Setting content_id: {}", content_id);
        *self.content_id.write() = content_id;
    }

    /// Gets current content ID
    pub fn get_content_id(&self) -> i32 {
        *self.content_id.read()
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

    /// Gets a list of tag IDs for the current content
    pub fn get_tag_ids(&self) -> Option<Vec<i32>> {
        match self.read() {
            Some(Ok(tags)) => Some(tags.iter().filter_map(|t| t.id).collect()),
            _ => None,
        }
    }

    /// Checks if a specific tag is assigned to the content
    pub fn has_tag(&self, tag_id: i32) -> bool {
        self.get_tag_ids()
            .map(|ids| ids.contains(&tag_id))
            .unwrap_or(false)
    }
}

/// Convenience function to create a UseContentTags hook
///
/// This is a shorthand for `UseContentTags::new(config, content_id)`
pub fn use_content_tags(config: ClientConfig, content_id: i32) -> UseContentTags {
    UseContentTags::new(config, content_id)
}

/// Convenience function to create a UseContentTags hook with custom table names
///
/// This is a shorthand for `UseContentTags::with_tables(config, content_id, content_tags_table, tags_table)`
pub fn use_content_tags_with_tables(
    config: ClientConfig,
    content_id: i32,
    content_tags_table: String,
    tags_table: String,
) -> UseContentTags {
    UseContentTags::with_tables(config, content_id, content_tags_table, tags_table)
}
