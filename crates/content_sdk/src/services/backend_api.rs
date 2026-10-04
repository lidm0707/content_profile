use crate::models::{
    Content, ContentRequest, ContentTag, ContentTagRequest, LoginRequest, Session, Tag, TagRequest,
    User,
};
use crate::pagination::{PaginatedResponse, PaginationParams};
use crate::utils::config::Config;
use chrono::Utc;
use supabase_client::transport::{self, Response};

const API_PREFIX: &str = "/api";
const BEARER: &str = "Bearer";
const SESSION_TTL_DAYS: i64 = 30;
const CONTENT_TYPE_JSON: &str = "Content-Type";
const MIME_JSON: &str = "application/json";
const ERR_NOT_CONFIGURED: &str = "Backend API not configured";
const ERR_SETTINGS_OBJECT: &str = "settings must be a JSON object";

#[derive(Clone, PartialEq)]
pub struct BackendApiService {
    base_url: Option<String>,
    jwt_token: Option<String>,
}

pub struct ApiPage {
    pub page: u32,
    pub page_size: u32,
    pub items: Vec<Content>,
    pub total_items: u32,
    pub total_pages: u32,
}

impl BackendApiService {
    pub fn new(config: Option<Config>) -> Self {
        let (base_url, jwt_token) = match config {
            Some(c) => (Some(c.api_base()), c.jwt_token.clone()),
            None => (None, None),
        };
        Self {
            base_url,
            jwt_token,
        }
    }

    fn base(&self) -> Result<&str, String> {
        self.base_url
            .as_deref()
            .ok_or(ERR_NOT_CONFIGURED.to_string())
    }

    fn url(&self, path: &str) -> Result<String, String> {
        Ok(format!("{}{}{}", self.base()?, API_PREFIX, path))
    }

    async fn request(
        &self,
        method: &'static str,
        path: &str,
        body: Option<String>,
    ) -> Result<Response, String> {
        let mut headers = vec![(CONTENT_TYPE_JSON.to_string(), MIME_JSON.to_string())];
        if let Some(token) = &self.jwt_token {
            headers.push(("Authorization".to_string(), format!("{BEARER} {token}")));
        }
        transport::send(method, &self.url(path)?, headers, body).await
    }

    async fn send_json<T: serde::de::DeserializeOwned>(
        &self,
        method: &'static str,
        path: &str,
        body: Option<String>,
    ) -> Result<T, String> {
        let resp = self.request(method, path, body).await?;
        if !is_ok(resp.status) {
            return Err(error_message(&resp));
        }
        serde_json::from_str(&resp.text).map_err(|e| format!("failed to parse response: {e}"))
    }

    /// Decodes JSON, mapping a 404 response to `None`.
    async fn send_json_opt<T: serde::de::DeserializeOwned>(
        &self,
        method: &'static str,
        path: &str,
    ) -> Result<Option<T>, String> {
        let resp = self.request(method, path, None).await?;
        if resp.status == 404 {
            return Ok(None);
        }
        if !is_ok(resp.status) {
            return Err(error_message(&resp));
        }
        serde_json::from_str(&resp.text)
            .map(Some)
            .map_err(|e| format!("failed to parse response: {e}"))
    }

    async fn send_empty(&self, method: &'static str, path: &str) -> Result<(), String> {
        let resp = self.request(method, path, None).await?;
        if is_ok(resp.status) {
            Ok(())
        } else {
            Err(error_message(&resp))
        }
    }

    async fn send_empty_json(
        &self,
        method: &'static str,
        path: &str,
        body: serde_json::Value,
    ) -> Result<(), String> {
        let resp = self.request(method, path, Some(body.to_string())).await?;
        if is_ok(resp.status) {
            Ok(())
        } else {
            Err(error_message(&resp))
        }
    }

    fn json_body(value: serde_json::Value) -> Option<String> {
        Some(value.to_string())
    }

    // --- auth ---

    pub async fn login(&self, request: LoginRequest) -> Result<Session, String> {
        let body = Self::json_body(serde_json::json!({
            "username": request.email,
            "password": request.password,
        }));
        let token: TokenResponse = self.send_json("POST", "/auth/login", body).await?;
        session_from_token(token)
    }

    pub async fn signup(&self, request: LoginRequest) -> Result<Session, String> {
        let body = Self::json_body(serde_json::json!({
            "username": request.email,
            "password": request.password,
        }));
        let token: TokenResponse = self.send_json("POST", "/auth/signup", body).await?;
        session_from_token(token)
    }

    pub async fn logout(&self) -> Result<(), String> {
        self.send_empty("POST", "/auth/logout").await
    }

    /// Asks the backend who this browser is via the HttpOnly session
    /// cookie. `None` when not logged in.
    pub async fn get_session(&self) -> Result<Option<String>, String> {
        let resp = self.request("GET", "/auth/session", None).await?;
        if resp.status == 401 {
            return Ok(None);
        }
        if !is_ok(resp.status) {
            return Err(error_message(&resp));
        }
        let value: serde_json::Value = serde_json::from_str(&resp.text)
            .map_err(|e| format!("failed to parse response: {e}"))?;
        Ok(value
            .get("username")
            .and_then(|u| u.as_str())
            .map(str::to_owned))
    }

    /// Exchanges a Supabase session for a backend session. The Supabase
    /// token pair travels along so the backend can store it per user.
    pub async fn exchange_supabase_session(&self, session: &Session) -> Result<Session, String> {
        let body = Self::json_body(serde_json::json!({
            "access_token": session.access_token,
            "refresh_token": if session.refresh_token.is_empty() { None } else { Some(session.refresh_token.clone()) },
            "expires_in": session.expires_at.saturating_sub(chrono::Utc::now().timestamp()),
        }));
        let token: TokenResponse = self.send_json("POST", "/auth/supabase", body).await?;
        session_from_token(token)
    }

    // --- per-user settings ---

    pub async fn get_user_settings(&self) -> Result<serde_json::Value, String> {
        self.send_json("GET", "/settings", None).await
    }

    pub async fn put_user_settings(&self, settings: serde_json::Value) -> Result<(), String> {
        if !settings.is_object() {
            return Err(ERR_SETTINGS_OBJECT.to_string());
        }
        self.send_empty_json("PUT", "/settings", settings).await
    }

    // --- google oauth (token lives on the backend, per user) ---

    pub async fn get_google_access_token(&self) -> Result<String, String> {
        #[derive(serde::Deserialize)]
        struct TokenDto {
            access_token: String,
        }
        let dto: TokenDto = self.send_json("GET", "/oauth/google/token", None).await?;
        Ok(dto.access_token)
    }

    pub async fn get_google_auth_url(&self) -> Result<String, String> {
        #[derive(serde::Deserialize)]
        struct UrlDto {
            auth_url: String,
        }
        let dto: UrlDto = self.send_json("GET", "/oauth/google/url", None).await?;
        Ok(dto.auth_url)
    }

    pub async fn get_google_status(&self) -> Result<bool, String> {
        #[derive(serde::Deserialize)]
        struct StatusDto {
            connected: bool,
        }
        let dto: StatusDto = self.send_json("GET", "/oauth/google/status", None).await?;
        Ok(dto.connected)
    }

    // --- sync ---

    pub async fn sync_pull(&self) -> Result<u32, String> {
        #[derive(serde::Deserialize)]
        struct PullDto {
            pulled: u32,
        }
        let dto: PullDto = self.send_json("POST", "/sync/pull", None).await?;
        Ok(dto.pulled)
    }

    // --- content ---

    pub async fn get_all_content(&self) -> Result<Vec<Content>, String> {
        self.send_json("GET", "/content", None).await
    }

    pub async fn get_content_by_id(&self, id: i32) -> Result<Option<Content>, String> {
        self.send_json_opt("GET", &format!("/content/{id}")).await
    }

    pub async fn get_content_by_slug(&self, slug: &str) -> Result<Option<Content>, String> {
        self.send_json_opt("GET", &format!("/content/slug/{slug}"))
            .await
    }

    pub async fn create_content(&self, request: ContentRequest) -> Result<Content, String> {
        let body = Self::json_body(upsert_body(&request));
        self.send_json("POST", "/content", body).await
    }

    pub async fn update_content(
        &self,
        id: i32,
        request: ContentRequest,
    ) -> Result<Content, String> {
        let body = Self::json_body(upsert_body(&request));
        self.send_json("PUT", &format!("/content/{id}"), body).await
    }

    pub async fn delete_content(&self, id: i32) -> Result<(), String> {
        self.send_empty("DELETE", &format!("/content/{id}")).await
    }

    pub async fn get_content_by_status(&self, status: &str) -> Result<Vec<Content>, String> {
        Ok(self
            .get_all_content()
            .await?
            .into_iter()
            .filter(|c| c.status == status)
            .collect())
    }

    pub async fn get_content_by_ids(&self, ids: &[i32]) -> Result<Vec<Content>, String> {
        if ids.is_empty() {
            return Ok(Vec::new());
        }
        let joined = ids
            .iter()
            .map(|id| id.to_string())
            .collect::<Vec<_>>()
            .join(",");
        self.send_json("GET", &format!("/content/by-ids?ids={joined}"), None)
            .await
    }

    pub async fn content_page(&self, page: u32, page_size: u32) -> Result<ApiPage, String> {
        #[derive(serde::Deserialize)]
        struct PageResponse {
            page: u32,
            page_size: u32,
            items: Vec<Content>,
            total_items: u32,
            total_pages: u32,
        }

        let path = format!("/content/page?page={page}&page_size={page_size}");
        let resp: PageResponse = self.send_json("GET", &path, None).await?;
        Ok(ApiPage {
            page: resp.page,
            page_size: resp.page_size,
            items: resp.items,
            total_items: resp.total_items,
            total_pages: resp.total_pages,
        })
    }

    pub async fn get_paginated_content(
        &self,
        page: u32,
        page_size: u32,
    ) -> Result<PaginatedResponse<Content>, String> {
        let resp = self.content_page(page, page_size).await?;
        Ok(PaginatedResponse::new(
            resp.items,
            &PaginationParams::new(resp.page, resp.page_size),
            resp.total_items,
        ))
    }

    pub async fn count_content(&self) -> Result<u32, String> {
        Ok(self.content_page(1, 1).await?.total_items)
    }

    // --- tags ---

    pub async fn get_all_tags(&self) -> Result<Vec<Tag>, String> {
        self.send_json("GET", "/tags", None).await
    }

    pub async fn get_tag_by_id(&self, id: i32) -> Result<Option<Tag>, String> {
        self.send_json_opt("GET", &format!("/tags/{id}")).await
    }

    pub async fn create_tag(&self, request: TagRequest) -> Result<Tag, String> {
        let body = Self::json_body(tag_body(&request));
        self.send_json("POST", "/tags", body).await
    }

    pub async fn update_tag(&self, id: i32, request: TagRequest) -> Result<Tag, String> {
        let body = Self::json_body(tag_body(&request));
        self.send_json("PUT", &format!("/tags/{id}"), body).await
    }

    pub async fn delete_tag(&self, id: i32) -> Result<(), String> {
        self.send_empty("DELETE", &format!("/tags/{id}")).await
    }

    // --- content <-> tag links ---

    pub async fn get_content_tags_for_content(
        &self,
        content_id: i32,
    ) -> Result<Vec<ContentTag>, String> {
        self.send_json("GET", &format!("/content/{content_id}/content-tags"), None)
            .await
    }

    pub async fn get_content_tags_for_tag(&self, tag_id: i32) -> Result<Vec<ContentTag>, String> {
        self.send_json("GET", &format!("/tags/{tag_id}/content-tags"), None)
            .await
    }

    pub async fn get_content_ids_for_tag(&self, tag_id: i32) -> Result<Vec<i32>, String> {
        Ok(self
            .get_content_tags_for_tag(tag_id)
            .await?
            .into_iter()
            .map(|ct| ct.content_id)
            .collect())
    }

    pub async fn add_tag_to_content(
        &self,
        request: ContentTagRequest,
    ) -> Result<ContentTag, String> {
        let body = Self::json_body(serde_json::json!({
            "content_id": request.content_id,
            "tag_id": request.tag_id,
        }));
        self.send_json("POST", "/content-tags", body).await
    }

    pub async fn remove_tag_from_content(
        &self,
        content_id: i32,
        tag_id: i32,
    ) -> Result<(), String> {
        self.send_empty(
            "DELETE",
            &format!("/content-tags/content/{content_id}/tag/{tag_id}"),
        )
        .await
    }

    pub async fn delete_content_tag(&self, id: i32) -> Result<(), String> {
        self.send_empty("DELETE", &format!("/content-tags/{id}"))
            .await
    }

    pub async fn get_tags_for_content(&self, content_id: i32) -> Result<Vec<Tag>, String> {
        self.send_json("GET", &format!("/content/{content_id}/tags"), None)
            .await
    }

    pub async fn get_content_for_tag(&self, tag_id: i32) -> Result<Vec<Content>, String> {
        let ids = self.get_content_ids_for_tag(tag_id).await?;
        self.get_content_by_ids(&ids).await
    }
}

fn is_ok(status: u16) -> bool {
    (200..300).contains(&status)
}

fn upsert_body(request: &ContentRequest) -> serde_json::Value {
    serde_json::json!({
        "title": request.title,
        "slug": request.slug,
        "body": request.body,
        "status": request.status,
    })
}

fn tag_body(request: &TagRequest) -> serde_json::Value {
    serde_json::json!({
        "name": request.name,
        "slug": request.slug,
        "parent_id": request.parent_id,
    })
}

/// Backend auth response: `{token, username}`.
#[derive(serde::Deserialize)]
struct TokenResponse {
    token: String,
    username: String,
}

/// Maps the backend `{token, username}` session to the SDK `Session` shape.
fn session_from_token(res: TokenResponse) -> Result<Session, String> {
    let now = Utc::now();
    Ok(Session {
        access_token: res.token,
        refresh_token: String::new(),
        expires_at: (now + chrono::Duration::days(SESSION_TTL_DAYS)).timestamp(),
        token_type: BEARER.to_string(),
        user: User {
            id: res.username.clone(),
            email: res.username,
            email_confirmed_at: None,
            created_at: now.to_rfc3339(),
            updated_at: now.to_rfc3339(),
            last_sign_in_at: Some(now.to_rfc3339()),
        },
    })
}

fn error_message(resp: &Response) -> String {
    let text = resp.text.trim();
    if text.is_empty() {
        format!("request failed with status {}", resp.status)
    } else {
        text.to_string()
    }
}
