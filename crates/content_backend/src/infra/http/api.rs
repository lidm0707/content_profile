use axum::Json;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::Deserialize;

use super::AppState;
use super::auth::{AuthUser, Credentials};
use crate::app::dto::{ContentDto, ContentTagDto, TagDto, UpsertContent, UpsertTag};
use crate::domain::entities::user::Session;
use crate::infra::pg::user_store::HexTokenIssuer;
use crate::port::{SettingsStore, TokenIssuer};

const ERR_NOT_FOUND: &str = "not found";
const IDS_SEPARATOR: char = ',';

#[derive(Deserialize)]
pub struct PageParams {
    pub page: Option<u32>,
    pub page_size: Option<u32>,
}

#[derive(Deserialize)]
pub struct IdsParams {
    pub ids: Option<String>,
}

fn parse_ids(raw: Option<String>) -> Vec<i32> {
    raw.unwrap_or_default()
        .split(IDS_SEPARATOR)
        .filter_map(|part| part.trim().parse::<i32>().ok())
        .collect()
}

fn err(e: String) -> Response {
    (StatusCode::INTERNAL_SERVER_ERROR, e).into_response()
}

fn not_found() -> Response {
    (StatusCode::NOT_FOUND, ERR_NOT_FOUND).into_response()
}

fn ok<T: serde::Serialize>(value: T) -> Response {
    Json(value).into_response()
}

pub(super) async fn signup(
    State(state): State<AppState>,
    Json(creds): Json<Credentials>,
) -> Result<Response, Response> {
    let res = super::auth::signup(&state.pool, &creds)
        .await
        .map_err(|(code, msg)| (code, msg).into_response())?;
    state
        .sessions
        .insert(&res.token, &res.username)
        .await
        .map_err(err)?;
    let cookie = super::auth::session_cookie(&res.token);
    Ok(super::auth::with_cookie(ok(res), cookie))
}

pub(super) async fn login(
    State(state): State<AppState>,
    Json(creds): Json<Credentials>,
) -> Result<Response, Response> {
    let res = super::auth::login(&state.pool, &creds)
        .await
        .map_err(|(code, msg)| (code, msg).into_response())?;
    state
        .sessions
        .insert(&res.token, &res.username)
        .await
        .map_err(err)?;
    let cookie = super::auth::session_cookie(&res.token);
    Ok(super::auth::with_cookie(ok(res), cookie))
}

pub(super) async fn logout(State(state): State<AppState>, user: AuthUser) -> Response {
    let _ = state.sessions.remove(&user.token).await;
    super::auth::with_cookie(
        StatusCode::NO_CONTENT.into_response(),
        super::auth::clear_session_cookie(),
    )
}

#[derive(Deserialize)]
pub struct SupabaseExchange {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_in: Option<i64>,
}

/// Exchanges a Supabase access token for a backend session so
/// Supabase-authenticated users can call the authenticated API.
/// The Supabase token pair is stored per user for later use.
pub(super) async fn supabase_exchange(
    State(state): State<AppState>,
    Json(body): Json<SupabaseExchange>,
) -> Result<Response, Response> {
    let username = state
        .supabase_auth
        .verify(&body.access_token)
        .await
        .map_err(|e| (StatusCode::UNAUTHORIZED, e).into_response())?;
    state
        .supabase_tokens
        .save(
            &username,
            &body.access_token,
            body.refresh_token.as_deref(),
            crate::infra::pg::supabase_tokens::expires_from_ttl(body.expires_in),
        )
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e).into_response())?;
    let token = HexTokenIssuer.issue();
    state
        .sessions
        .insert(&token, &username)
        .await
        .map_err(err)?;
    let cookie = super::auth::session_cookie(&token);
    Ok(super::auth::with_cookie(
        ok(Session { token, username }),
        cookie,
    ))
}

/// Tells the browser who it is, based on the session cookie. 401 when
/// no valid session — the UI's "am I still logged in?" probe.
pub(super) async fn get_session(user: AuthUser) -> Response {
    ok(serde_json::json!({ "username": user.username }))
}

// --- google oauth (per-user tokens stored in pg; per-user client id in settings) ---

#[derive(serde::Serialize)]
pub struct AuthUrlDto {
    pub auth_url: String,
}

#[derive(serde::Serialize)]
pub struct AccessTokenDto {
    pub access_token: String,
}

#[derive(serde::Serialize)]
pub struct ConnectedDto {
    pub connected: bool,
}

#[derive(Deserialize)]
pub struct CallbackQuery {
    pub code: Option<String>,
    pub error: Option<String>,
    pub state: Option<String>,
}

async fn google_service(
    state: &AppState,
    username: &str,
) -> Result<crate::app::GoogleOauthService, Response> {
    state
        .google_service(username)
        .await
        .map_err(|e| (StatusCode::NOT_IMPLEMENTED, e).into_response())
}

/// Returns the Google consent URL. `state` carries the caller's session token
/// so the unauthenticated redirect-back can be attributed to a user.
pub(super) async fn google_auth_url(
    State(state): State<AppState>,
    user: AuthUser,
) -> Result<Response, Response> {
    let service = google_service(&state, &user.username).await?;
    tracing::info!(
        "google oauth: using redirect_uri from {}",
        service.redirect_url()
    );
    Ok(ok(AuthUrlDto {
        auth_url: service.auth_url(&user.token),
    }))
}

/// Google redirects here. Exchanges the code and stores the tokens for the
/// user identified by the `state` session token.
pub(super) async fn google_callback(
    State(state): State<AppState>,
    Query(query): Query<CallbackQuery>,
) -> Response {
    if let Some(error) = query.error {
        return (StatusCode::BAD_REQUEST, error).into_response();
    }
    let Some(code) = query.code else {
        return (StatusCode::BAD_REQUEST, "missing code").into_response();
    };
    let Some(state_token) = query.state else {
        return (StatusCode::BAD_REQUEST, "missing state").into_response();
    };
    let username = match state.sessions.get(&state_token).await {
        Ok(Some(username)) => username,
        _ => return (StatusCode::UNAUTHORIZED, "unknown session").into_response(),
    };
    let service = match google_service(&state, &username).await {
        Ok(s) => s,
        Err(resp) => return resp,
    };
    match service.exchange(&code).await {
        Ok(tokens) => match service.save(&username, &tokens).await {
            Ok(()) => (
                StatusCode::OK,
                "Google Drive connected — you can close this tab",
            )
                .into_response(),
            Err(e) => err(e),
        },
        Err(e) => err(e),
    }
}

/// Returns a valid Drive access token for the current user, refreshing via
/// the stored refresh token when needed.
pub(super) async fn google_access_token(
    State(state): State<AppState>,
    user: AuthUser,
) -> Result<Response, Response> {
    let service = google_service(&state, &user.username).await?;
    let token = service.access_token(&user.username).await.map_err(|e| {
        if e == "not connected" || e == "no refresh token stored" {
            (StatusCode::NOT_FOUND, e).into_response()
        } else {
            (StatusCode::INTERNAL_SERVER_ERROR, e).into_response()
        }
    })?;
    Ok(ok(AccessTokenDto {
        access_token: token,
    }))
}

/// Connection status for the current user.
pub(super) async fn google_status(
    State(state): State<AppState>,
    user: AuthUser,
) -> Result<Response, Response> {
    let service = google_service(&state, &user.username).await?;
    let connected = service.is_connected(&user.username).await.map_err(err)?;
    Ok(ok(ConnectedDto { connected }))
}

const SECRET_SETTING_KEYS: &[&str] = &["supabase_password"];

/// Removes never-stored secret keys (e.g. passwords) from a settings object.
fn strip_secret_keys(settings: &mut serde_json::Value) {
    if let Some(obj) = settings.as_object_mut() {
        for key in SECRET_SETTING_KEYS {
            obj.remove(*key);
        }
    }
}

pub(super) async fn get_settings(
    State(state): State<AppState>,
    user: AuthUser,
) -> Result<Response, Response> {
    let mut settings = state.settings.load(&user.username).await.map_err(err)?;
    strip_secret_keys(&mut settings);
    Ok(ok(settings))
}

pub(super) async fn put_settings(
    State(state): State<AppState>,
    user: AuthUser,
    Json(mut update): Json<serde_json::Value>,
) -> Result<Response, Response> {
    if !update.is_object() {
        return Ok((StatusCode::BAD_REQUEST, "settings must be a JSON object").into_response());
    }
    // Passwords must never be persisted — auth uses refresh tokens instead.
    strip_secret_keys(&mut update);
    // Merge partial updates so separate tabs can save their own fields
    // without wiping fields saved from another tab.
    let mut merged = state.settings.load(&user.username).await.map_err(err)?;
    if let (Some(current), Some(update)) = (merged.as_object_mut(), update.as_object()) {
        for (key, value) in update {
            current.insert(key.clone(), value.clone());
        }
    }
    state
        .settings
        .save(&user.username, &merged)
        .await
        .map_err(err)?;
    Ok(ok(merged))
}

pub(super) async fn list_content(
    State(state): State<AppState>,
    _user: AuthUser,
) -> Result<Response, Response> {
    let items = state.backend.content.list().await.map_err(err)?;
    Ok(ok(items.iter().map(ContentDto::from).collect::<Vec<_>>()))
}

pub(super) async fn get_content(
    State(state): State<AppState>,
    _user: AuthUser,
    Path(id): Path<i32>,
) -> Response {
    match state.backend.content.get(id).await {
        Ok(Some(c)) => ok(ContentDto::from(&c)),
        Ok(None) => not_found(),
        Err(e) => err(e),
    }
}

pub(super) async fn get_content_by_slug(
    State(state): State<AppState>,
    _user: AuthUser,
    Path(slug): Path<String>,
) -> Response {
    match state.backend.content.get_by_slug(&slug).await {
        Ok(Some(c)) => ok(ContentDto::from(&c)),
        Ok(None) => not_found(),
        Err(e) => err(e),
    }
}

pub(super) async fn create_content(
    State(state): State<AppState>,
    _user: AuthUser,
    Json(data): Json<UpsertContent>,
) -> Result<Response, Response> {
    let saved = state.backend.content.create(data).await.map_err(err)?;
    Ok(ok(ContentDto::from(&saved)))
}

pub(super) async fn update_content(
    State(state): State<AppState>,
    _user: AuthUser,
    Path(id): Path<i32>,
    Json(data): Json<UpsertContent>,
) -> Result<Response, Response> {
    let saved = state.backend.content.update(id, data).await.map_err(err)?;
    Ok(ok(ContentDto::from(&saved)))
}

pub(super) async fn delete_content(
    State(state): State<AppState>,
    _user: AuthUser,
    Path(id): Path<i32>,
) -> Result<Response, Response> {
    state.backend.content.delete(id).await.map_err(err)?;
    Ok(StatusCode::NO_CONTENT.into_response())
}

pub(super) async fn content_page(
    State(state): State<AppState>,
    _user: AuthUser,
    Query(params): Query<PageParams>,
) -> Result<Response, Response> {
    let page = state
        .backend
        .content_page(params.page.unwrap_or(1), params.page_size.unwrap_or(20))
        .await
        .map_err(err)?;
    Ok(ok(super::server::PageResponse::from(page)))
}

pub(super) async fn content_by_ids(
    State(state): State<AppState>,
    _user: AuthUser,
    Query(params): Query<IdsParams>,
) -> Result<Response, Response> {
    let ids = parse_ids(params.ids);
    if ids.is_empty() {
        return Ok(ok(Vec::<ContentDto>::new()));
    }
    let items = state.backend.content_by_ids(&ids).await.map_err(err)?;
    Ok(ok(items))
}

pub(super) async fn tags_by_ids(
    State(state): State<AppState>,
    _user: AuthUser,
    Query(params): Query<IdsParams>,
) -> Result<Response, Response> {
    let ids = parse_ids(params.ids);
    if ids.is_empty() {
        return Ok(ok(Vec::<TagDto>::new()));
    }
    let items = state.backend.tags_by_ids(&ids).await.map_err(err)?;
    Ok(ok(items))
}

pub(super) async fn tags_for_content(
    State(state): State<AppState>,
    _user: AuthUser,
    Path(id): Path<i32>,
) -> Result<Response, Response> {
    let items = state.backend.tags_for_content(id).await.map_err(err)?;
    Ok(ok(items))
}

#[derive(serde::Serialize)]
pub struct SyncPullResult {
    pub pulled: usize,
}

/// Pulls content and tags from Supabase into local pg.
pub(super) async fn sync_pull(
    State(state): State<AppState>,
    _user: AuthUser,
) -> Result<Response, Response> {
    let pulled = state.backend.pull_remote().await.map_err(err)?;
    Ok(ok(SyncPullResult { pulled }))
}

pub(super) async fn content_tags_for_content(
    State(state): State<AppState>,
    _user: AuthUser,
    Path(id): Path<i32>,
) -> Result<Response, Response> {
    let items = state
        .backend
        .content_tags
        .list_for_content(id)
        .await
        .map_err(err)?;
    Ok(ok(items))
}

pub(super) async fn content_tags_for_tag(
    State(state): State<AppState>,
    _user: AuthUser,
    Path(id): Path<i32>,
) -> Result<Response, Response> {
    let items = state
        .backend
        .content_tags
        .list_for_tag(id)
        .await
        .map_err(err)?;
    Ok(ok(items))
}

pub(super) async fn add_content_tag(
    State(state): State<AppState>,
    _user: AuthUser,
    Json(link): Json<ContentTagDto>,
) -> Result<Response, Response> {
    let saved = state
        .backend
        .content_tags
        .add(link.content_id, link.tag_id)
        .await
        .map_err(err)?;
    Ok(ok(saved))
}

pub(super) async fn remove_content_tag_pair(
    State(state): State<AppState>,
    _user: AuthUser,
    Path((content_id, tag_id)): Path<(i32, i32)>,
) -> Result<Response, Response> {
    state
        .backend
        .content_tags
        .remove_pair(content_id, tag_id)
        .await
        .map_err(err)?;
    Ok(StatusCode::NO_CONTENT.into_response())
}

pub(super) async fn remove_content_tag(
    State(state): State<AppState>,
    _user: AuthUser,
    Path(id): Path<i32>,
) -> Result<Response, Response> {
    state.backend.content_tags.remove(id).await.map_err(err)?;
    Ok(StatusCode::NO_CONTENT.into_response())
}

pub(super) async fn list_tags(
    State(state): State<AppState>,
    _user: AuthUser,
) -> Result<Response, Response> {
    let items = state.backend.tag.list().await.map_err(err)?;
    Ok(ok(items.iter().map(TagDto::from).collect::<Vec<_>>()))
}

pub(super) async fn get_tag(
    State(state): State<AppState>,
    _user: AuthUser,
    Path(id): Path<i32>,
) -> Response {
    match state.backend.tag.get(id).await {
        Ok(Some(t)) => ok(TagDto::from(&t)),
        Ok(None) => not_found(),
        Err(e) => err(e),
    }
}

pub(super) async fn create_tag(
    State(state): State<AppState>,
    _user: AuthUser,
    Json(data): Json<UpsertTag>,
) -> Result<Response, Response> {
    let saved = state.backend.tag.create(data).await.map_err(err)?;
    Ok(ok(TagDto::from(&saved)))
}

pub(super) async fn update_tag(
    State(state): State<AppState>,
    _user: AuthUser,
    Path(id): Path<i32>,
    Json(data): Json<UpsertTag>,
) -> Result<Response, Response> {
    let saved = state.backend.tag.update(id, data).await.map_err(err)?;
    Ok(ok(TagDto::from(&saved)))
}

pub(super) async fn delete_tag(
    State(state): State<AppState>,
    _user: AuthUser,
    Path(id): Path<i32>,
) -> Result<Response, Response> {
    state.backend.tag.delete(id).await.map_err(err)?;
    Ok(StatusCode::NO_CONTENT.into_response())
}

pub(super) async fn flush_sync(
    State(state): State<AppState>,
    _user: AuthUser,
) -> Result<Response, Response> {
    let pushed = state.backend.flush_pending().await.map_err(err)?;
    Ok(ok(serde_json::json!({ "pushed": pushed })))
}
