# Implement Pagination in content_sdk

## Overview
Add pagination support to the content_sdk crate to enable server-side pagination of content items. Currently, `get_all_content()` fetches all items at once, which is inefficient for large datasets.

## Requirements

### 1. Add Pagination Parameters
Create a new `PaginationParams` struct:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginationParams {
    pub page: u32,
    pub page_size: u32,
}

impl PaginationParams {
    pub fn new(page: u32, page_size: u32) -> Self {
        Self { page: page.max(1), page_size }
    }
    
    pub fn offset(&self) -> u32 {
        (self.page - 1) * self.page_size
    }
    
    pub fn default() -> Self {
        Self { page: 1, page_size: 10 }
    }
}
```

### 2. Add Paginated Response Model
Create a `PaginatedResponse<T>` generic struct:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginatedResponse<T> {
    pub data: Vec<T>,
    pub page: u32,
    pub page_size: u32,
    pub total_items: u32,
    pub total_pages: u32,
}

impl<T> PaginatedResponse<T> {
    pub fn has_next(&self) -> bool {
        self.page < self.total_pages
    }
    
    pub fn has_previous(&self) -> bool {
        self.page > 1
    }
}
```

### 3. Update ContentContext
Add new methods to `ContentContext`:

```rust
impl ContentContext {
    // Existing method (keep for backward compatibility)
    pub async fn get_all_content(&self) -> Result<Vec<Content>, Error> {
        self.get_paginated_content(PaginationParams::default())
            .await
            .map(|r| r.data)
    }
    
    // New paginated method
    pub async fn get_paginated_content(
        &self,
        params: PaginationParams,
    ) -> Result<PaginatedResponse<Content>, Error> {
        // Query with pagination using Supabase
        // Include pagination metadata in response
    }
    
    pub async fn count_all_content(&self) -> Result<u32, Error> {
        // Query total count of content items
    }
}
```

### 4. Update TagContext (if applicable)
Add similar pagination methods for tags:

```rust
impl TagContext {
    pub async fn get_paginated_tags(
        &self,
        params: PaginationParams,
    ) -> Result<PaginatedResponse<Tag>, Error> {
        // Query with pagination
    }
}
```

### 5. Update Supabase Queries
Modify API calls to include pagination:

```rust
pub async fn get_paginated_content(
    &self,
    params: PaginationParams,
) -> Result<PaginatedResponse<Content>, Error> {
    // Build query with range pagination
    let response = self.client
        .from("content")
        .select("*")
        .range(params.offset() as usize, (params.offset() + params.page_size) as usize - 1)
        .order("created_at", SupabaseOrder::Ascending)
        .execute()
        .await?;
    
    // Parse response
    let items: Vec<Content> = serde_json::from_str(&response.text()?)?;
    
    // Get total count
    let total_items = self.count_all_content().await?;
    let total_pages = (total_items as f64 / params.page_size as f64).ceil() as u32;
    
    Ok(PaginatedResponse {
        data: items,
        page: params.page,
        page_size: params.page_size,
        total_items,
        total_pages,
    })
}
```

### 6. Update Export Module
Add new types to the main module exports:

```rust
pub mod pagination {
    pub use crate::pagination::{PaginationParams, PaginatedResponse};
}
```

## Implementation Guidelines

### API Design
- Use 1-based pagination (page starts at 1, not 0)
- Default page_size should be 10
- Maximum page_size should be 100 (add validation)
- Return total_items and total_pages in every response

### Error Handling
- Return error if page < 1
- Return error if page_size < 1 or > 100
- Handle empty results gracefully (total_items = 0, total_pages = 0)

### Performance
- Use Supabase's range header for efficient pagination
- Fetch total count separately or use Content-Range header
- Consider caching total counts for frequently accessed tables

### Backward Compatibility
- Keep existing `get_all_content()` method (internally use default pagination)
- Maintain current method signatures where possible
- Add deprecation warnings for old methods if needed

### Testing
- Unit tests for PaginationParams (edge cases: page 0, negative, etc.)
- Integration tests for paginated queries
- Verify pagination metadata is accurate
- Test empty result sets
- Test maximum page_size limits

### Documentation
- Add documentation to all new methods
- Include examples in module-level docs
- Document pagination parameters and response structure
- Add usage examples for common pagination patterns

## Acceptance Criteria
- [x] `PaginationParams` struct with validation
- [x] `PaginatedResponse<T>` generic struct
- [x] `get_paginated_content()` method in ContentContext
- [x] `count_content()` helper method (note: named differently from plan)
- [x] Supabase queries use offset/limit pagination
- [x] Total count and pages calculated correctly
- [x] Backward compatibility maintained
- [ ] Unit tests added
- [ ] Documentation complete
- [x] Code passes cargo check and clippy

## Future Enhancements
- Cursor-based pagination for better performance on large datasets
- Filtering and sorting in paginated queries