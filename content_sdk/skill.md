# Content SDK - A reusable Dioxus SDK for content management

## Introduction

Content SDK is a reusable library for building content-focused applications with Dioxus and Supabase. It provides:

- **Models**: Data structures for content and tags with serialization support
- **Hooks**: Reusable hooks for fetching and managing content with filtering and search
- **Supabase Integration**: Direct use of supabase_client for data operations

## Quick Start

### Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
content_sdk = { path = "./content_sdk" }
dioxus = { version = "0.7.1", features = ["web"] }
supabase_client = { path = "./supabase_client" }
```

### Basic Application

```rust
use content_sdk::hooks::UseContent;
use supabase_client::{ClientConfig, client_config};
use dioxus::prelude::*;

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    let config = use_signal(|| {
        client_config(
            "https://your-project.supabase.co".to_string(),
            "your-anon-key".to_string(),
        )
    });
    
    let content = UseContent::new(config());
    
    rsx! {
        match content.read() {
            Some(Ok(items)) => rsx! {
                for item in items {
                    div { "{item.title}" }
                }
            },
            Some(Err(e)) => rsx! { div { "Error: {e}" } },
            None => rsx! { div { "Loading..." } },
        }
    }
}
```

## Models

### Content Model

The Content model represents content items in your application.

```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Content {
    pub id: Option<i32>,
    pub title: String,
    pub slug: String,
    pub body: String,
    pub status: String,
    pub tags: Option<Vec<i32>>,
    pub featured_image: Option<String>,
    pub excerpt: Option<String>,
    pub author: Option<String>,
    pub content_type: Option<String>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
    pub published_at: Option<DateTime<Utc>>,
    pub synced_at: Option<DateTime<Utc>>,
}
```

#### Content Creation

```rust
use content_sdk::models::{Content, ContentStatus};

let content = Content::new(
    "My Blog Post".to_string(),
    "my-blog-post".to_string(),
    "# Hello World\n\nThis is my first post.".to_string(),
);

// Add additional properties
let content = content
    .with_status(ContentStatus::Published)
    .with_author("John Doe".to_string())
    .with_excerpt("A brief summary...".to_string())
    .with_content_type("blog".to_string());
```

#### Content Status

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContentStatus {
    Draft,
    Published,
    Archived,
}

// Check status
if content.is_published() {
    // Display published content
}

// Get status enum
if let Some(status) = content.get_status() {
    match status {
        ContentStatus::Draft => // Handle draft
        ContentStatus::Published => // Handle published
        ContentStatus::Archived => // Handle archived
    }
}
```

#### Slug Generation

```rust
let slug = Content::generate_slug("My Awesome Blog Post!");
// Result: "my-awesome-blog-post"
```

#### Excerpt Generation

```rust
let excerpt = content.get_excerpt(150);
// Generates excerpt from body if none exists
```

### Tag Model

The Tag model represents tags that can be associated with content.

```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Tag {
    pub id: Option<i32>,
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    pub color: Option<String>,
    pub count: Option<i32>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}
```

#### Tag Creation

```rust
use content_sdk::models::Tag;

let tag = Tag::new(
    "Rust".to_string(),
    "rust".to_string(),
)
.with_description("Programming language".to_string())
.with_color("#dea584".to_string());
```

#### Tag Validation

```rust
// Validate tag name
if Tag::is_valid_name("Rust") {
    // Valid name
}

// Validate slug
if Tag::is_valid_slug("rust-programming") {
    // Valid slug
}

// Validate color (hex format)
if Tag::is_valid_color("#dea584") {
    // Valid color
}
```

## Hooks

### UseContent Hook

The UseContent hook provides reactive data fetching for content from Supabase.

```rust
use content_sdk::hooks::UseContent;
use supabase_client::ClientConfig;

let content = UseContent::new(config);

// Read content
match content.read() {
    Some(Ok(items)) => rsx! {
        for item in items {
            div { "{item.title}" }
        }
    },
    Some(Err(e)) => rsx! { div { "Error: {e}" } },
    None => rsx! { div { "Loading..." } },
}
```

#### Filtering

```rust
// Filter by status
content.set_status_filter(Some(ContentStatus::Published));

// Filter by content type
content.set_content_type_filter(Some("blog".to_string()));

// Filter by tags
content.set_tag_filter(Some(vec![1, 2, 3]));

// Search query
content.set_search_query(Some("rust".to_string()));

// Get filtered results
match content.get_filtered() {
    Some(Ok(items)) => rsx! { /* ... */ },
    _ => rsx! { /* ... */ },
}
```

#### Custom Table Name

```rust
// Use a different Supabase table
let content = UseContent::with_table(config, "custom_content".to_string());
```

#### Fetch by ID

```rust
let item = content.get_by_id(123).await?;
```

#### Refresh Data

```rust
content.refresh();
```

### UseTags Hook

The UseTags hook provides reactive data fetching for tags from Supabase.

```rust
use content_sdk::hooks::UseTags;

let tags = UseTags::new(config);

// Read tags
match tags.read() {
    Some(Ok(tags)) => rsx! {
        for tag in tags {
            div { "{tag.name}" }
        }
    },
    Some(Err(e)) => rsx! { div { "Error: {e}" } },
    None => rsx! { div { "Loading..." } },
}
```

#### Filtering

```rust
// Filter by IDs
tags.set_id_filter(Some(vec![1, 2, 3]));

// Search tags
tags.set_search_query(Some("rust".to_string()));

// Get filtered results
match tags.get_filtered() {
    Some(Ok(tags)) => rsx! { /* ... */ },
    _ => rsx! { /* ... */ },
}
```

#### Finding Tags

```rust
// Find by ID
if let Some(tag) = tags.find_by_id(123) {
    // Use tag
}

// Find by slug
if let Some(tag) = tags.find_by_slug("rust") {
    // Use tag
}

// Fetch by ID from Supabase
let tag = tags.get_by_id(123).await?;
```

#### Content by Tags

Efficiently fetch all content items that have specific tags using batch operations:

```rust
use content_sdk::contexts::{ContentContext, ContentTagsContext, TagContext};

#[component]
fn ContentByTag(tag_name: String) -> Element {
    let content_context: ContentContext = use_context();
    let tag_context: TagContext = use_context();
    let content_tags_context: ContentTagsContext = use_context();

    let mut contents = use_resource(move || {
        let tag_name = tag_name.clone();
        let tag_context = tag_context.clone();
        let content_tags_context = content_tags_context.clone();
        let content_context = content_context.clone();

        async move {
            // Step 1: Get tag by name
            let all_tags = tag_context.get_all_tags().await?;
            let tag = all_tags
                .iter()
                .find(|t| t.name == tag_name)
                .ok_or_else(|| format!("Tag '{}' not found", tag_name))?;

            let tag_id = tag.id.ok_or_else(|| format!("Tag '{}' has no ID", tag_name))?;

            // Step 2: Get content IDs for the tag
            let content_ids = content_tags_context.get_content_ids_for_tag(tag_id).await?;

            if content_ids.is_empty() {
                return Ok(Vec::new());
            }

            // Step 3: Batch fetch all content items
            content_context.get_content_by_ids(&content_ids).await
        }
    });

    rsx! {
        div {
            match contents.read().as_ref() {
                None => rsx! { div { "Loading..." } },
                Some(Ok(content_list)) => rsx! {
                    div {
                        for content in content_list {
                            ContentCard { content }
                        }
                    }
                },
                Some(Err(e)) => rsx! { div { "Error: {e}" } },
            }
        }
    }
}
```

This pattern efficiently fetches all content for a specific tag using:
- `TagContext::get_all_tags()` to find the tag by name
- `ContentTagsContext::get_content_ids_for_tag()` to get junction records
- `ContentContext::get_content_by_ids()` to batch fetch all content in one request

This prevents N+1 query problems by fetching all needed records in a single batch request instead of making individual requests for each content item.

#### Batch Fetching by IDs

When you have multiple content IDs and need to fetch them efficiently:

```rust
use content_sdk::contexts::ContentContext;

async fn fetch_content_batch(
    content_context: &ContentContext,
    content_ids: Vec<i32>,
) -> Result<Vec<Content>, String> {
    // Empty check
    if content_ids.is_empty() {
        return Ok(Vec::new());
    }

    // Batch fetch all content in one request
    content_context.get_content_by_ids(&content_ids).await
}
```

Use this pattern when:
- Fetching related content from junction tables
- Displaying multiple content items from a list
- Implementing batch operations that would otherwise cause N+1 queries

## Pagination

The Content SDK provides built-in pagination support for efficiently handling large datasets. Pagination is handled through `PaginationParams` and `PaginatedResponse` types, and integrated into `ContentContext`.

### Pagination Types

```rust
use content_sdk::pagination::{PaginationParams, PaginatedResponse};

/// Request parameters for pagination
#[derive(Debug, Clone, PartialEq)]
pub struct PaginationParams {
    pub page: u32,
    pub page_size: u32,
}

impl PaginationParams {
    pub fn new(page: u32, page_size: u32) -> Self;
}

impl Default for PaginationParams {
    fn default() -> Self {
        Self { page: 1, page_size: 10 }
    }
}

/// Response containing paginated data and metadata
#[derive(Debug, Clone, PartialEq)]
pub struct PaginatedResponse<T> {
    pub data: Vec<T>,
    pub total: u32,
    pub page: u32,
    pub page_size: u32,
    pub total_pages: u32,
}
```

### Using Pagination with ContentContext

```rust
use content_sdk::contexts::ContentContext;
use content_sdk::pagination::PaginationParams;
use dioxus::prelude::*;

#[component]
fn ContentList() -> Element {
    let content_context: ContentContext = use_context();
    let mut current_page = use_signal(|| 1);
    let page_size = 9;

    // Fetch paginated content
    let mut contents = use_resource(move || {
        let content_context = content_context.clone();
        let page = current_page();
        async move {
            content_context
                .get_paginated_content(&[], page, page_size)
                .await
        }
    });

    // Fetch total count
    let mut total_count = use_resource(move || {
        let content_context = content_context.clone();
        async move { content_context.count_content(&[]).await }
    });

    rsx! {
        div {
            // Display content items
            if let Some(result) = contents.read().as_ref() {
                match result {
                    Ok(response) => rsx! {
                        div {
                            class: "grid grid-cols-1 md:grid-cols-3 gap-6",
                            for item in response.data.iter() {
                                ContentCard { content: item.clone() }
                            }
                        }

                        // Pagination controls
                        PaginationControls {
                            current_page: current_page(),
                            total_count: total_count.read().as_ref().and_then(|r| r.as_ref().ok()).copied().unwrap_or(0),
                            page_size,
                            on_previous: move |_| {
                                if current_page() > 1 {
                                    current_page -= 1;
                                }
                            },
                            on_next: move |_| {
                                let current = current_page();
                                let total = total_count.read().as_ref().and_then(|r| r.as_ref().ok()).copied().unwrap_or(0);
                                let max_page = if total > 0 {
                                    (total + page_size - 1) / page_size
                                } else {
                                    1
                                };
                                if current < max_page {
                                    current_page += 1;
                                }
                            },
                        }
                    },
                    Err(e) => rsx! { div { "Error: {e}" } },
                }
            } else {
                div { "Loading..." }
            }
        }
    }
}

#[component]
fn PaginationControls(
    current_page: u32,
    total_count: u32,
    page_size: u32,
    on_previous: EventHandler<MouseEvent>,
    on_next: EventHandler<MouseEvent>,
) -> Element {
    let max_page = if total_count > 0 {
        (total_count + page_size - 1) / page_size
    } else {
        1
    };

    if max_page <= 1 {
        return rsx! { None };
    }

    rsx! {
        div {
            class: "flex items-center justify-between border-t border-gray-200 pt-4",

            div {
                p {
                    class: "text-sm text-gray-700",
                    "Showing ",
                    span { class: "font-medium", "{(current_page - 1) * page_size + 1}" },
                    " to ",
                    span { class: "font-medium", "{current_page * page_size.min(total_count)}" },
                    " of ",
                    span { class: "font-medium", "{total_count}" },
                    " results"
                }
            }

            div {
                class: "inline-flex rounded-md shadow-sm -space-x-px",

                button {
                    disabled: current_page == 1,
                    onclick: on_previous,
                    class: if current_page == 1 {
                        "relative inline-flex items-center px-2 py-2 rounded-l-md border border-gray-300 bg-gray-50 text-sm font-medium text-gray-300 cursor-not-allowed"
                    } else {
                        "relative inline-flex items-center px-2 py-2 rounded-l-md border border-gray-300 bg-white text-sm font-medium text-gray-500 hover:bg-gray-50"
                    },
                    "Previous"
                }

                span {
                    class: "relative inline-flex items-center px-4 py-2 border border-gray-300 bg-white text-sm font-medium text-gray-700",
                    "Page {current_page} of {max_page}"
                }

                button {
                    disabled: current_page == max_page,
                    onclick: on_next,
                    class: if current_page == max_page {
                        "relative inline-flex items-center px-2 py-2 rounded-r-md border border-gray-300 bg-gray-50 text-sm font-medium text-gray-300 cursor-not-allowed"
                    } else {
                        "relative inline-flex items-center px-2 py-2 rounded-r-md border border-gray-300 bg-white text-sm font-medium text-gray-500 hover:bg-gray-50"
                    },
                    "Next"
                }
            }
        }
    }
}
```

### Pagination with Filters

```rust
use content_sdk::contexts::ContentContext;
use dioxus::prelude::*;

#[component]
fn FilteredContentList() -> Element {
    let content_context: ContentContext = use_context();
    let mut current_page = use_signal(|| 1);
    let page_size = 10;
    let mut status_filter = use_signal(|| None::<String>);

    let mut contents = use_resource(move || {
        let content_context = content_context.clone();
        let page = current_page();
        let status = status_filter();
        async move {
            let filters = if let Some(status) = status {
                vec![("status", status.as_str())]
            } else {
                vec![]
            };
            content_context
                .get_paginated_content(&filters, page, page_size)
                .await
        }
    });

    rsx! {
        div {
            // Status filter
            select {
                value: "{status_filter.read().as_deref().unwrap_or_default()}",
                onchange: move |e| {
                    status_filter.set(if e.value().is_empty() { None } else { Some(e.value()) });
                    current_page.set(1); // Reset to first page when filter changes
                },
                option { value: "", "All Status" }
                option { value: "published", "Published" }
                option { value: "draft", "Draft" }
                option { value: "archived", "Archived" }
            }

            // Content grid
            if let Some(result) = contents.read().as_ref() {
                match result {
                    Ok(response) => rsx! {
                        div {
                            class: "grid grid-cols-1 md:grid-cols-3 gap-6",
                            for item in response.data.iter() {
                                ContentCard { content: item.clone() }
                            }
                        }
                    },
                    Err(e) => rsx! { div { "Error: {e}" } },
                }
            }
        }
    }
}
```

### Pagination Best Practices

#### Page Size Selection

```rust
// ✅ GOOD - Choose page size based on your layout
const PAGE_SIZE: u32 = 9; // 3x3 grid
const PAGE_SIZE: u32 = 12; // 3x4 grid or 4x3 grid
const PAGE_SIZE: u32 = 20; // List view

// ❌ BAD - Too large page size causes performance issues
const PAGE_SIZE: u32 = 1000; // Will be slow
```

#### Reset Page on Filter Change

```rust
// ✅ GOOD - Reset page when filters change
onchange: move |e| {
    status_filter.set(Some(e.value()));
    current_page.set(1); // Reset to first page
}

// ❌ BAD - Don't reset page
onchange: move |e| {
    status_filter.set(Some(e.value()));
    // User might be on page 5 with only 1 page of filtered results
}
```

#### Handle Empty Results

```rust
// ✅ GOOD - Show appropriate message when no results
if response.data.is_empty() {
    return rsx! {
        div {
            class: "text-center py-8",
            p { class: "text-gray-500", "No content found" }
        }
    };
}

// ❌ BAD - Empty state without feedback
if response.data.is_empty() {
    return rsx! { div { } }; // Users won't know what happened
}
```

#### Use Total Count for Stats

```rust
// ✅ GOOD - Use count_content() for accurate stats
let total_count = use_resource(move || {
    let content_context = content_context.clone();
    async move { content_context.count_content(&[]).await }
});

rsx! {
    StatCard {
        label: "Total Content".to_string(),
        value: total_count.read().as_ref().and_then(|r| r.as_ref().ok()).copied().unwrap_or(0).to_string(),
    }
}

// ❌ BAD - Use current page data for total stats
StatCard {
    label: "Total Content".to_string(),
    value: response.data.len().to_string(), // Only shows current page
}
```

### Pagination Validation

```rust
use content_sdk::pagination::PaginationParams;

// PaginationParams automatically validates:
// - page is clamped to minimum 1
// - page_size is clamped between 1 and 100

// These will be automatically corrected:
let params = PaginationParams::new(0, 50); // page becomes 1
let params = PaginationParams::new(1, 0);   // page_size becomes 1
let params = PaginationParams::new(1, 200); // page_size becomes 100
```

### Performance Considerations

1. **Office Mode**: Pagination is performed in-memory on all fetched data
2. **Supabase Mode**: Pagination is performed server-side with `offset` and `limit` query parameters
3. **Separate Count Call**: `count_content()` makes a separate GET request for accurate total counts
4. **Filter Support**: Pagination works seamlessly with filters - filters are applied before pagination

## Supabase Client Configuration

### Basic Configuration

```rust
use supabase_client::{ClientConfig, client_config};

let config = client_config(
    "https://your-project.supabase.co".to_string(),
    "your-anon-key".to_string(),
);
```

### Advanced Configuration

```rust
let config = client_config(
    "https://your-project.supabase.co".to_string(),
    "your-anon-key".to_string(),
)
.with_service_role_key("your-service-role-key".to_string())
.with_jwt_token("your-jwt-token".to_string());
```

### Validation

```rust
if let Err(e) = config.validate() {
    eprintln!("Invalid config: {}", e);
}
```

## Common Patterns

### Content List with Filters

```rust
#[component]
fn ContentList() -> Element {
    let config = use_signal(|| {
        client_config(
            "https://your-project.supabase.co".to_string(),
            "your-anon-key".to_string(),
        )
    });
    
    let mut content = UseContent::new(config());
    
    // Only show published blog posts
    content.set_status_filter(Some(ContentStatus::Published));
    content.set_content_type_filter(Some("blog".to_string()));
    
    rsx! {
        match content.get_filtered() {
            Some(Ok(items)) => rsx! {
                for item in items {
                    div { class: "content-item",
                        h2 { "{item.title}" }
                        if let Some(excerpt) = &item.excerpt {
                            p { "{excerpt}" }
                        }
                    }
                }
            },
            Some(Err(e)) => rsx! { div { "Error: {e}" } },
            None => rsx! { div { "Loading..." } },
        }
    }
}
```

### Tag Cloud

```rust
#[component]
fn TagCloud() -> Element {
    let config = use_signal(|| {
        client_config(
            "https://your-project.supabase.co".to_string(),
            "your-anon-key".to_string(),
        )
    });
    
    let tags = UseTags::new(config());
    
    rsx! {
        div { class: "tag-cloud",
            match tags.read() {
                Some(Ok(tags)) => rsx! {
                    for tag in tags {
                        span { 
                            class: "tag",
                            style: "background-color: {tag.color.unwrap_or_else(|| '#ccc'.to_string())}",
                            "{tag.name}"
                        }
                    }
                },
                _ => rsx! { "Loading tags..." },
            }
        }
    }
}
```

### Search Functionality

```rust
#[component]
fn ContentSearch() -> Element {
    let config = use_signal(|| {
        client_config(
            "https://your-project.supabase.co".to_string(),
            "your-anon-key".to_string(),
        )
    });
    
    let mut content = UseContent::new(config());
    let mut search_query = use_signal(|| String::new());
    
    // Update search query when input changes
    use_effect(move || {
        let query = search_query();
        content.set_search_query(if query.is_empty() { None } else { Some(query) });
    });
    
    rsx! {
        input {
            value: "{search_query}",
            oninput: move |e| *search_query.write() = e.value(),
            placeholder: "Search content...",
        }
        
        div { class: "search-results",
            match content.get_filtered() {
                Some(Ok(items)) => rsx! {
                    for item in items {
                        div { "{item.title}" }
                    }
                },
                _ => rsx! { div { "No results" } },
            }
        }
    }
}
```

### Content Detail Page

```rust
#[component]
fn ContentDetail(id: i32) -> Element {
    let config = use_signal(|| {
        client_config(
            "https://your-project.supabase.co".to_string(),
            "your-anon-key".to_string(),
        )
    });
    
    let content = UseContent::new(config());
    let mut item = use_resource(move || async move {
        content.get_by_id(id).await
    });
    
    rsx! {
        match &*item.read() {
            Some(Ok(Some(content))) => rsx! {
                article {
                    h1 { "{content.title}" }
                    if let Some(author) = &content.author {
                        p { class: "author", "By {author}" }
                    }
                    div { 
                        class: "content-body",
                        dangerous_inner_html: "{content.body}"
                    }
                }
            },
            Some(Ok(None)) => rsx! { div { "Content not found" } },
            Some(Err(e)) => rsx! { div { "Error: {e}" } },
            None => rsx! { div { "Loading..." } },
        }
    }
}
```

## Best Practices

### Signal Management

```rust
// ✅ GOOD - Config in a signal for reactivity
let config = use_signal(|| {
    client_config(
        "https://your-project.supabase.co".to_string(),
        "your-anon-key".to_string(),
    )
});
let content = UseContent::new(config());

// ❌ BAD - Config not in a signal
let config = client_config(
    "https://your-project.supabase.co".to_string(),
    "your-anon-key".to_string(),
);
let content = UseContent::new(config);
```

### Error Handling

```rust
// ✅ GOOD - Proper error handling
match content.read() {
    Some(Ok(items)) => rsx! { /* Display items */ },
    Some(Err(e)) => rsx! { 
        div { class: "error", 
            p { "Failed to load content" }
            p { "{e}" }
        }
    },
    None => rsx! { div { "Loading..." } },
}

// ❌ BAD - No error handling
match content.read() {
    Some(Ok(items)) => rsx! { /* Display items */ },
    _ => rsx! { div { "Something went wrong" } },
}
```

### Filtering Strategy

```rust
// ✅ GOOD - Apply filters on the client after fetching
let mut content = UseContent::new(config);
content.set_status_filter(Some(ContentStatus::Published));
content.set_content_type_filter(Some("blog".to_string()));

// ❌ BAD - Make separate requests for each filter
// This causes multiple API calls
let published = UseContent::with_table(config, "content".to_string());
let blog = UseContent::with_table(config, "content".to_string());
```

### Tag Association

```rust
// ✅ GOOD - Store tag IDs in content
let content = Content::new(
    "Title".to_string(),
    "slug".to_string(),
    "Body".to_string(),
);

// Associate with tags
content.tags = Some(vec![1, 2, 3]);

// ❌ BAD - Embed full tag objects
let content = Content::new(
    "Title".to_string(),
    "slug".to_string(),
    "Body".to_string(),
);

// Don't do this - use IDs instead
// content.tags = Some(vec![tag1, tag2, tag3]);
```

### Loading States

```rust
// ✅ GOOD - Provide loading feedback
if content.is_loading() {
    return rsx! {
        div { class: "loading",
            "Loading content..."
        }
    };
}

// ❌ BAD - No loading state
// Users won't know what's happening
match content.read() {
    Some(Ok(items)) => rsx! { /* ... */ },
    Some(Err(e)) => rsx! { /* ... */ },
    None => rsx! { /* Empty - bad UX */ },
}
```

## API Reference

### Pagination Types

| Type | Description |
|------|-------------|
| `PaginationParams` | Request parameters for pagination (page, page_size) |
| `PaginatedResponse<T>` | Response with paginated data and metadata |

### ContentContext Methods

| Method | Description | Returns |
|--------|-------------|---------|
| `get_all_content()` | Fetch all content (uses default pagination) | `Result<Vec<Content>>` |
| `get_paginated_content(filters, page, page_size)` | Fetch paginated content | `Result<PaginatedResponse<Content>>` |
| `count_content(filters)` | Count total content items | `Result<u32>` |
| `get_content_by_id(id)` | Fetch single content by ID | `Result<Option<Content>>` |
| `get_content_by_slug(slug)` | Fetch single content by slug | `Result<Option<Content>>` |
| `get_content_by_status(status)` | Fetch content by status | `Result<Vec<Content>>` |
| `get_content_by_ids(ids)` | Batch fetch content by IDs | `Result<Vec<Content>>` |
| `create_content(request)` | Create new content | `Result<Content>` |
| `update_content(id, request)` | Update existing content | `Result<Content>` |
| `delete_content(id)` | Delete content by ID | `Result<()>` |

### UseContent Methods

| Method | Description | Returns |
|--------|-------------|---------|
| `new(config)` | Create with default table | `UseContent` |
| `with_table(config, table)` | Create with custom table | `UseContent` |
| `read()` | Get current content value | `Option<Result<Vec<Content>>>` |
| `get_filtered()` | Get filtered content | `Option<Result<Vec<Content>>>` |
| `refresh()` | Reload from Supabase | `()` |
| `set_status_filter()` | Filter by status | `()` |
| `set_tag_filter()` | Filter by tag IDs | `()` |
| `set_content_type_filter()` | Filter by type | `()` |
| `set_search_query()` | Set search query | `()` |
| `get_status_filter()` | Get current status filter | `Option<ContentStatus>` |
| `get_tag_filter()` | Get current tag filter | `Option<Vec<i32>>` |
| `get_content_type_filter()` | Get current type filter | `Option<String>` |
| `get_search_query()` | Get current search query | `Option<String>` |
| `is_loading()` | Check if loading | `bool` |
| `get_all()` | Get unfiltered content | `Option<Result<Vec<Content>>>` |
| `get_by_id(id)` | Fetch single item by ID | `Result<Option<Content>>` |

### UseTags Methods

| Method | Description | Returns |
|--------|-------------|---------|
| `new(config)` | Create with default table | `UseTags` |
| `with_table(config, table)` | Create with custom table | `UseTags` |
| `read()` | Get current tags value | `Option<Result<Vec<Tag>>>` |
| `get_filtered()` | Get filtered tags | `Option<Result<Vec<Tag>>>` |
| `refresh()` | Reload from Supabase | `()` |
| `set_id_filter()` | Filter by tag IDs | `()` |
| `set_search_query()` | Set search query | `()` |
| `find_by_id(id)` | Find in loaded tags | `Option<Tag>` |
| `find_by_slug(slug)` | Find by slug | `Option<Tag>` |
| `get_by_id(id)` | Fetch from Supabase | `Result<Option<Tag>>` |

## Troubleshooting

### Content Not Loading

```rust
// Check if config is valid
if let Err(e) = config.validate() {
    eprintln!("Config error: {}", e);
}

// Check network errors
match content.read() {
    Some(Err(e)) => {
        eprintln!("Fetch error: {}", e);
        // Handle network issues
    },
    _ => {},
}
```

### Filters Not Working

```rust
// Ensure filters are set before reading
content.set_status_filter(Some(ContentStatus::Published));

// Then get filtered results
if let Some(Ok(items)) = content.get_filtered() {
    // Process filtered items
}
```

### Tags Not Associating

```rust
// Ensure tag IDs exist
let tags = UseTags::new(config.clone());
if let Some(Ok(all_tags)) = tags.read() {
    let tag_ids: Vec<i32> = all_tags.iter().filter_map(|t| t.id).collect();
    let content = Content::new("Title".to_string(), "slug".to_string(), "Body".to_string());
    // content.tags = Some(tag_ids); // Use existing IDs
}
```
