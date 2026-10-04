use axum::extract::FromRequestParts;
use axum::http::header::AUTHORIZATION;
use axum::http::request::Parts;
use axum::http::{HeaderMap, HeaderValue, StatusCode};
use axum::response::Response;
use sqlx::PgPool;

pub use crate::domain::entities::user::{Credentials, Session};
use crate::domain::entities::user::{ERR_INVALID_CREDENTIALS, ERR_USERNAME_TAKEN};
use crate::domain::services::AuthService;
use crate::infra::pg::user_store::{Argon2Hasher, HexTokenIssuer, PgUserStore};

const ERR_UNAUTHORIZED: &str = "unauthorized";
const BEARER_PREFIX: &str = "Bearer ";
const COOKIE_HEADER: &str = "cookie";
const COOKIE_ATTRS: &str = "; HttpOnly; SameSite=Lax; Path=/";

/// Name of the browser cookie holding the backend session token.
pub const SESSION_COOKIE: &str = "cms_session";

/// Set-Cookie value establishing the session cookie.
pub fn session_cookie(token: &str) -> String {
    format!("{SESSION_COOKIE}={token}{COOKIE_ATTRS}")
}

/// Set-Cookie value expiring the session cookie.
pub fn clear_session_cookie() -> String {
    format!("{SESSION_COOKIE}=; Max-Age=0{COOKIE_ATTRS}")
}

/// Extracts the session token from the Cookie header, if present.
fn cookie_token(headers: &HeaderMap) -> Option<String> {
    let cookies = headers.get(COOKIE_HEADER)?.to_str().ok()?;
    cookies.split(';').find_map(|pair| {
        let (name, value) = pair.trim().split_once('=')?;
        (name == SESSION_COOKIE).then(|| value.to_owned())
    })
}

pub type LocalAuthService = AuthService<PgUserStore, Argon2Hasher, HexTokenIssuer>;

pub fn local_auth(pool: &PgPool) -> LocalAuthService {
    AuthService::new(PgUserStore::new(pool.clone()), Argon2Hasher, HexTokenIssuer)
}

fn map_auth_err(e: String) -> (StatusCode, String) {
    if e == ERR_USERNAME_TAKEN {
        (StatusCode::CONFLICT, e)
    } else if e == ERR_INVALID_CREDENTIALS {
        (StatusCode::UNAUTHORIZED, e)
    } else {
        (StatusCode::INTERNAL_SERVER_ERROR, e)
    }
}

pub async fn signup(pool: &PgPool, creds: &Credentials) -> Result<Session, (StatusCode, String)> {
    local_auth(pool).register(creds).await.map_err(map_auth_err)
}

pub async fn login(pool: &PgPool, creds: &Credentials) -> Result<Session, (StatusCode, String)> {
    local_auth(pool).login(creds).await.map_err(map_auth_err)
}

#[derive(Clone)]
pub struct AuthUser {
    pub token: String,
    pub username: String,
}

pub struct Unauthorized;

impl std::fmt::Display for Unauthorized {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(ERR_UNAUTHORIZED)
    }
}

impl axum::response::IntoResponse for Unauthorized {
    fn into_response(self) -> Response {
        (
            StatusCode::UNAUTHORIZED,
            [("www-authenticate", "Bearer realm=\"api\"")],
            ERR_UNAUTHORIZED.to_string(),
        )
            .into_response()
    }
}

pub fn bearer_token(headers: &HeaderMap) -> Option<String> {
    headers
        .get(AUTHORIZATION)?
        .to_str()
        .ok()?
        .strip_prefix(BEARER_PREFIX)
        .map(str::to_owned)
        .filter(|t| !t.is_empty())
}

/// Attaches a Set-Cookie header to an existing response.
pub fn with_cookie(mut response: Response, cookie: String) -> Response {
    if let Ok(value) = HeaderValue::from_str(&cookie) {
        response
            .headers_mut()
            .insert(axum::http::header::SET_COOKIE, value);
    }
    response
}

impl FromRequestParts<super::AppState> for AuthUser {
    type Rejection = Unauthorized;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &super::AppState,
    ) -> Result<Self, Self::Rejection> {
        // Bearer first (API clients), then the HttpOnly session cookie
        // (browser — the token never needs to be readable by JS).
        let token = bearer_token(&parts.headers)
            .or_else(|| cookie_token(&parts.headers))
            .ok_or(Unauthorized)?;
        let username = state
            .sessions
            .get(&token)
            .await
            .map_err(|_| Unauthorized)?
            .ok_or(Unauthorized)?;
        Ok(Self { token, username })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bearer_token_parses_header() {
        let mut headers = HeaderMap::new();
        assert_eq!(bearer_token(&headers), None);
        headers.insert(AUTHORIZATION, "Bearer abc".parse().expect("header value"));
        assert_eq!(bearer_token(&headers).as_deref(), Some("abc"));
    }
}
