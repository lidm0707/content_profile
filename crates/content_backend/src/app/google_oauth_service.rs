use chrono::{DateTime, Duration, Utc};
use serde::Deserialize;

use crate::infra::pg::google_oauth::{GoogleTokenRow, PgGoogleTokenStore};

const TOKEN_URL: &str = "https://oauth2.googleapis.com/token";
const AUTH_URL: &str = "https://accounts.google.com/o/oauth2/v2/auth";
pub(crate) const DRIVE_SCOPE: &str = "https://www.googleapis.com/auth/drive.file";
const GRANT_CODE: &str = "authorization_code";
const GRANT_REFRESH: &str = "refresh_token";
/// Refresh this many seconds before the stored token actually expires.
const EXPIRY_SKEW_SECS: i64 = 60;
const ERR_NOT_CONNECTED: &str = "not connected";
const ERR_NO_REFRESH: &str = "no refresh token stored";

#[derive(Clone)]
pub struct GoogleOauthService {
    client_id: String,
    client_secret: String,
    redirect_url: String,
    store: PgGoogleTokenStore,
}

#[derive(Debug, Clone)]
pub struct TokenSet {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_at: DateTime<Utc>,
    pub scope: String,
}

#[derive(Deserialize)]
struct GoogleTokenResponse {
    access_token: String,
    #[serde(default)]
    refresh_token: Option<String>,
    #[serde(default)]
    expires_in: Option<i64>,
    #[serde(default)]
    scope: Option<String>,
}

impl GoogleOauthService {
    pub fn new(
        client_id: String,
        client_secret: String,
        redirect_url: String,
        store: PgGoogleTokenStore,
    ) -> Self {
        Self {
            client_id,
            client_secret,
            redirect_url,
            store,
        }
    }

    pub(crate) fn redirect_url(&self) -> &str {
        &self.redirect_url
    }

    /// Consent screen URL. `state` carries the app session token so the
    /// unauthenticated Google redirect can be attributed to a user.
    pub fn auth_url(&self, state: &str) -> String {
        let query = [
            ("client_id", self.client_id.as_str()),
            ("redirect_uri", self.redirect_url.as_str()),
            ("response_type", "code"),
            ("scope", DRIVE_SCOPE),
            ("access_type", "offline"),
            ("prompt", "consent"),
            ("state", state),
        ];
        let encoded: Vec<String> = query
            .iter()
            .map(|(k, v)| format!("{k}={}", urlencode(v)))
            .collect();
        format!("{AUTH_URL}?{}", encoded.join("&"))
    }

    /// Exchanges an authorization code for the first token set.
    pub async fn exchange(&self, code: &str) -> Result<TokenSet, String> {
        let form = [
            ("code", code),
            ("client_id", self.client_id.as_str()),
            ("client_secret", self.client_secret.as_str()),
            ("redirect_uri", self.redirect_url.as_str()),
            ("grant_type", GRANT_CODE),
        ];
        self.request_token(&form).await
    }

    /// Returns a valid access token for the user, refreshing via the stored
    /// refresh token when the current one is (nearly) expired.
    pub async fn access_token(&self, username: &str) -> Result<String, String> {
        let row = self
            .store
            .find(username)
            .await?
            .ok_or_else(|| ERR_NOT_CONNECTED.to_string())?;
        if row.expires_at > Utc::now() + Duration::seconds(EXPIRY_SKEW_SECS) {
            return Ok(row.access_token);
        }
        self.refresh(row).await
    }

    async fn refresh(&self, row: GoogleTokenRow) -> Result<String, String> {
        let refresh_token = row
            .refresh_token
            .as_deref()
            .ok_or_else(|| ERR_NO_REFRESH.to_string())?;
        let form = [
            ("refresh_token", refresh_token),
            ("client_id", self.client_id.as_str()),
            ("client_secret", self.client_secret.as_str()),
            ("grant_type", GRANT_REFRESH),
        ];
        let tokens = self.request_token(&form).await?;
        Ok(tokens.access_token)
    }

    async fn request_token(&self, form: &[(&str, &str)]) -> Result<TokenSet, String> {
        let resp = reqwest::Client::new()
            .post(TOKEN_URL)
            .form(form)
            .send()
            .await
            .map_err(|e| format!("google token request failed: {e}"))?;
        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(format!("google token endpoint returned {status}: {body}"));
        }
        let parsed: GoogleTokenResponse = resp
            .json()
            .await
            .map_err(|e| format!("failed to parse google token response: {e}"))?;
        let expires_in = parsed.expires_in.unwrap_or(3600);
        let expires_at = Utc::now() + Duration::seconds(expires_in);
        Ok(TokenSet {
            access_token: parsed.access_token,
            refresh_token: parsed.refresh_token,
            expires_at,
            scope: parsed.scope.unwrap_or_else(|| DRIVE_SCOPE.to_string()),
        })
    }

    pub async fn is_connected(&self, username: &str) -> Result<bool, String> {
        Ok(self.store.find(username).await?.is_some())
    }

    pub async fn save(&self, username: &str, tokens: &TokenSet) -> Result<(), String> {
        self.store
            .save(
                username,
                &tokens.access_token,
                tokens.refresh_token.as_deref(),
                tokens.expires_at,
                &tokens.scope,
            )
            .await
    }
}

fn urlencode(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    for byte in value.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(byte as char)
            }
            other => out.push_str(&format!("%{other:02X}")),
        }
    }
    out
}
