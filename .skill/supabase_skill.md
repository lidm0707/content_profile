# Supabase Skill Reference

## Quick Start

### Cargo.toml Setup

```toml
[dependencies]
supabase_client = { path = "../supabase_client" }
gloo-net = { version = "0.6", features = ["http"] }
web-sys = { version = "0.3", features = ["console"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
chrono = { version = "0.4", features = ["serde"] }
```

### Basic Configuration

```rust
use supabase_client::{ClientConfig, client_config};

let config = client_config(
    "https://your-project.supabase.co".to_string(),
    "your-anon-key".to_string(),
);
```

### Basic Request

```rust
use supabase_client::get;

let results: Vec<Content> = get::<Content>(&config, "content", &[]).await?;
```

## Client Configuration

### ClientConfig Structure

```rust
pub struct ClientConfig {
    pub base_url: String,
    pub anon_key: String,
    pub service_role_key: Option<String>,
    pub jwt_token: Option<String>,
}
```

### Creating ClientConfig

```rust
use supabase_client::ClientConfig;

// Basic configuration
let config = ClientConfig::new(
    "https://your-project.supabase.co".to_string(),
    "your-anon-key".to_string(),
);

// With service role key
let config = ClientConfig::new(base_url, anon_key)
    .with_service_role_key(service_role_key);

// With JWT token
let config = ClientConfig::new(base_url, anon_key)
    .with_jwt_token(access_token);
```

### Credential Priority

The `get_credential` function prioritizes credentials in this order:
1. `service_role_key` - if present
2. `jwt_token` - if present
3. `anon_key` - fallback

```rust
fn get_credential(config: &ClientConfig) -> &str {
    if let Some(ref service_role_key) = config.service_role_key {
        return service_role_key;
    }
    if let Some(ref jwt_token) = config.jwt_token {
        return jwt_token;
    }
    &config.anon_key
}
```

## Authentication

### JWT Token Management

```rust
use supabase_client::ClientConfig;

fn config_with_jwt(config: ClientConfig, access_token: String) -> ClientConfig {
    config.with_jwt_token(access_token)
}
```

### Session-based Authentication

```rust
use crate::services::session::SessionStorage;

fn config_with_session(config: ClientConfig) -> ClientConfig {
    match SessionStorage::load_session() {
        Ok(Some(session)) => config.with_jwt_token(session.access_token),
        _ => config,
    }
}
```

### Authorization Header

```rust
const AUTHORIZATION_HEADER: &str = "Authorization";
const BEARER_PREFIX: &str = "Bearer ";

headers.set(
    AUTHORIZATION_HEADER,
    &format!("{}{}", BEARER_PREFIX, credential),
);
```

## REST API Operations

### GET - Read Data

```rust
use supabase_client::get;

// Get all records
let items: Vec<Content> = get::<Content>(&config, "content", &[]).await?;

// Get with parameters
let items: Vec<Content> = get::<Content>(
    &config,
    "content",
    &[("order", "created_at.desc")],
).await?;
```

### GET by ID

```rust
use supabase_client::get_by_id;

let content: Option<Content> = get_by_id::<Content>(&config, "content", 5).await?;
```

### GET by Column Value

```rust
use supabase_client::get_by;

let items: Vec<Content> = get_by::<Content>(
    &config,
    "content",
    "slug",
    "my-post",
).await?;
```

### POST - Create Data

```rust
use supabase_client::create;

let content_request = ContentRequest {
    id: None,
    title: "My Title".to_string(),
    slug: "my-title".to_string(),
    body: "Content body".to_string(),
    status: "draft".to_string(),
    tags: None,
};

let results: Vec<Content> = create::<ContentRequest, Content>(
    &config,
    "content",
    &content_request,
).await?;
```

### PATCH - Update Data

```rust
use supabase_client::update;

let content_request = ContentRequest {
    id: Some(5),
    title: "Updated Title".to_string(),
    slug: "my-title".to_string(),
    body: "Updated body".to_string(),
    status: "published".to_string(),
    tags: None,
};

let results: Vec<Content> = update::<ContentRequest, Content>(
    &config,
    "content",
    5,
    &content_request,
).await?;
```

### DELETE - Remove Data

```rust
use supabase_client::delete;

delete(&config, "content", 5).await?;
```

## Query Filtering

### Filter Syntax

Supabase uses operator prefixes for filtering:

| Operator | Syntax | Description |
|----------|--------|-------------|
| Equals | `column=eq.value` | Exact match |
| Not equals | `column=neq.value` | Not equal |
| Greater than | `column=gt.value` | Greater than |
| Greater than or equal | `column=gte.value` | Greater than or equal |
| Less than | `column=lt.value` | Less than |
| Less than or equal | `column=lte.value` | Less than or equal |
| Like | `column=like.value%` | Pattern match |
| ILike | `column=ilike.value%` | Case-insensitive like |
| Is | `column=is.value` | Exact match |
| In | `column=in.(value1,value2)` | In list |

### Filter Implementation

The `build_url` function automatically adds `eq.` prefix to filter parameters:

```rust
// Filters get eq. prefix
let items: Vec<Content> = get::<Content>(
    &config,
    "content",
    &[("id", "5"), ("status", "published")],
).await?;
// URL: /content?id=eq.5&status=eq.published
```

### Non-Filter Parameters

Ordering, limit, offset, and select parameters do NOT get the `eq.` prefix:

```rust
let items: Vec<Content> = get::<Content>(
    &config,
    "content",
    &[("order", "created_at.desc"), ("limit", "10")],
).await?;
// URL: /content?order=created_at.desc&limit=10
```

### Combined Filtering

```rust
let items: Vec<Content> = get::<Content>(
    &config,
    "content",
    &[
        ("status", "published"),
        ("order", "created_at.desc"),
        ("limit", "20"),
    ],
).await?;
// URL: /content?status=eq.published&order=created_at.desc&limit=20
```

## Request Headers

### Standard Headers

```rust
const API_KEY_HEADER: &str = "apikey";
const AUTHORIZATION_HEADER: &str = "Authorization";
const CONTENT_TYPE_HEADER: &str = "Content-Type";
const PREFER_HEADER: &str = "Prefer";
```

### Build Headers

```rust
fn build_headers(config: &ClientConfig, prefer_return: bool) -> Result<Headers, String> {
    let headers = Headers::new();
    let credential = get_credential(config);

    headers.set(API_KEY_HEADER, &config.anon_key);
    headers.set(
        AUTHORIZATION_HEADER,
        &format!("{}{}", BEARER_PREFIX, credential),
    );
    headers.set(CONTENT_TYPE_HEADER, APPLICATION_JSON);

    if prefer_return {
        headers.set(PREFER_HEADER, RETURN_REPRESENTATION);
    }

    Ok(headers)
}
```

### Return Representation

For create/update operations, you can request the created/updated record:

```rust
const RETURN_REPRESENTATION: &str = "return=representation";

// With return representation
headers.set(PREFER_HEADER, RETURN_REPRESENTATION);

// Response includes the created/updated record
```

## Data Serialization

### Custom Serializer for ID Field

Handle `None` values by serializing to `0`:

```rust
use serde::Serializer;

fn serialize_id<S>(id: &Option<i32>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    match id {
        Some(value) => serializer.serialize_i32(*value),
        None => serializer.serialize_i32(0),
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Props)]
pub struct ContentRequest {
    #[serde(serialize_with = "serialize_id")]
    pub id: Option<i32>,
    pub title: String,
    pub slug: String,
    pub body: String,
    pub status: String,
    pub tags: Option<Vec<i32>>,
}
```

### Serialization Behavior

| Operation | ID Value | JSON Output |
|-----------|----------|-------------|
| Create | `id: None` | `{"id": 0, ...}` |
| Update | `id: Some(5)` | `{"id": 5, ...}` |

### Skip Serializing

Alternative approach to skip `None` values entirely:

```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Props)]
pub struct ContentRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<i32>,
    // ... other fields
}
```

## Error Handling

### Common Supabase Errors

| Code | Message | Solution |
|------|---------|----------|
| `23502` | null value violates not-null constraint | Ensure all NOT NULL fields are provided |
| `23503` | duplicate key value violates unique constraint | Use unique values for unique columns |
| `23505` | foreign key constraint violated | Ensure referenced record exists |
| `PGRST116` | relation does not exist | Check table name spelling |
| `PGRST204` | column does not exist | Check column name spelling |
| `JWT expired` | Token expired | Refresh token |

### Error Response Structure

```rust
#[derive(Debug, Deserialize)]
pub struct SupabaseError {
    pub code: String,
    pub message: String,
    pub details: Option<String>,
    pub hint: Option<String>,
}
```

### Handling Errors

```rust
use supabase_client::get;

let items: Vec<Content> = match get::<Content>(&config, "content", &[]).await {
    Ok(items) => items,
    Err(e) => {
        // Log error and return empty vector or propagate error
        eprintln!("Failed to fetch content: {}", e);
        vec![]
    }
};
```

## Best Practices

### URL Construction

Manually construct URLs to avoid trailing `&` and control format:

```rust
fn build_url(
    config: &ClientConfig,
    table: &str,
    params: &[(&str, &str)],
) -> Result<String, String> {
    let base_url = config.rest_url();
    let mut url_string = format!("{}/{}", base_url, table);

    if !params.is_empty() {
        let query_string: String = params
            .iter()
            .filter(|(k, v)| !k.is_empty() && !v.is_empty())
            .map(|(k, v)| {
                let no_eq_prefix = matches!(k.as_ref(), "order" | "limit" | "offset" | "select");
                if no_eq_prefix {
                    format!("{}={}", encode(k), encode(v))
                } else {
                    format!("{}=eq.{}", encode(k), encode(v))
                }
            })
            .collect::<Vec<_>>()
            .join("&");

        if !query_string.is_empty() {
            url_string = format!("{}?{}", url_string, query_string);
        }
    }

    Ok(url_string)
}
```

### Authentication

Always use JWT tokens for authenticated requests:

```rust
fn config_with_jwt(config: ClientConfig) -> ClientConfig {
    match SessionStorage::load_session() {
        Ok(Some(session)) => config.with_jwt_token(session.access_token),
        _ => config,
    }
}
```

### ID Handling

Use custom serializers to handle optional ID fields:

```rust
#[serde(serialize_with = "serialize_id")]
pub id: Option<i32>,
```

### Filter Parameters

Apply `eq.` prefix to filter parameters, not to ordering/limit/offset:

```rust
let no_eq_prefix = matches!(k.as_ref(), "order" | "limit" | "offset" | "select");
```

### Error Handling

Propagate errors appropriately and provide meaningful error messages:

```rust
match get_by_id::<Content>(&config, "content", id).await {
    Ok(Some(content)) => Ok(content),
    Ok(None) => Err(format!("Content with id {} not found", id)),
    Err(e) => Err(format!("Failed to fetch content: {}", e)),
}
```

## Service Pattern

### SupabaseService Structure

```rust
use supabase_client::{ClientConfig, client_config, get, get_by_id, create, update, delete};

#[derive(Clone)]
pub struct SupabaseService {
    config: ClientConfig,
}

impl SupabaseService {
    pub fn new() -> Self {
        let app_config = get_config();
        let supabase_url = app_config.supabase_url
            .expect("SUPABASE_URL must be set in Supabase mode");
        let supabase_anon_key = app_config.supabase_anon_key
            .expect("SUPABASE_ANON_KEY must be set in Supabase mode");

        let config = client_config(supabase_url, supabase_anon_key);

        Self { config }
    }

    fn config_with_jwt(&self) -> ClientConfig {
        match SessionStorage::load_session() {
            Ok(Some(session)) => self.config.clone().with_jwt_token(session.access_token),
            _ => self.config.clone(),
        }
    }

    pub async fn get_all_content(&self) -> Result<Vec<Content>, String> {
        let config = self.config_with_jwt();
        get::<Content>(&config, "content", &[("order", "created_at.desc")]).await
    }

    pub async fn get_content_by_id(&self, id: i32) -> Result<Option<Content>, String> {
        let config = self.config_with_jwt();
        get_by_id::<Content>(&config, "content", id).await
    }

    pub async fn create_content(&self, content_request: ContentRequest) -> Result<Content, String> {
        let config = self.config_with_jwt();
        let results: Vec<Content> = create::<ContentRequest, Content>(
            &config,
            "content",
            &content_request,
        ).await?;
        results.into_iter().next().ok_or_else(|| "No content returned".to_string())
    }

    pub async fn update_content(
        &self,
        id: i32,
        content_request: ContentRequest,
    ) -> Result<Content, String> {
        let config = self.config_with_jwt();
        let results: Vec<Content> = update::<ContentRequest, Content>(
            &config,
            "content",
            id,
            &content_request,
        ).await?;
        results.into_iter().next().ok_or_else(|| "No content returned".to_string())
    }

    pub async fn delete_content(&self, id: i32) -> Result<(), String> {
        let config = self.config_with_jwt();
        delete(&config, "content", id).await
    }
}
```

## Resources

- [Supabase REST API Documentation](https://supabase.com/docs/guides/database/api)
- [Supabase Authentication](https://supabase.com/docs/guides/auth)
- [PostgREST API](https://postgrest.org/en/stable/v7.0.0/api.html)
- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- [Serde Documentation](https://serde.rs/)