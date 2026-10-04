use async_trait::async_trait;
use serde::Deserialize;
use serde_json::json;
use supabase_client::transport;

use crate::domain::entities::user::{Credentials, Session};
use crate::infra::supabase::SupabaseConfig;
use crate::port::RemoteAuth;

const PATH_SIGNUP: &str = "auth/v1/signup";
const PATH_TOKEN: &str = "auth/v1/token";
const PATH_USER: &str = "auth/v1/user";
const QUERY_PASSWORD_GRANT: &str = "grant_type=password";
const HEADER_API_KEY: &str = "apikey";
const HEADER_AUTHORIZATION: &str = "Authorization";
const HEADER_CONTENT_TYPE: &str = "Content-Type";
const MIME_JSON: &str = "application/json";
const METHOD_POST: &str = "POST";
const METHOD_GET: &str = "GET";
const BEARER_PREFIX: &str = "Bearer ";
const ERR_NOT_CONFIGURED: &str = "supabase not configured";
const ERR_REQUEST: &str = "auth request failed";
const ERR_NO_TOKEN: &str = "no access token returned";
const ERR_TOKEN_REJECTED: &str = "supabase token rejected";

#[derive(Deserialize, Default)]
struct GotrueUser {
    #[serde(default)]
    email: Option<String>,
}

#[derive(Deserialize, Default)]
struct GotrueResponse {
    #[serde(default)]
    access_token: Option<String>,
    #[serde(default)]
    user: Option<GotrueUser>,
}

#[derive(Clone)]
pub struct SupabaseRemoteAuth {
    ctx: Option<SupabaseConfig>,
}

impl SupabaseRemoteAuth {
    pub fn new(ctx: Option<SupabaseConfig>) -> Self {
        Self { ctx }
    }

    fn ctx(&self) -> Result<&SupabaseConfig, String> {
        self.ctx
            .as_ref()
            .ok_or_else(|| ERR_NOT_CONFIGURED.to_string())
    }

    fn headers(ctx: &SupabaseConfig) -> Vec<(String, String)> {
        vec![
            (HEADER_API_KEY.to_string(), ctx.anon_key.clone()),
            (HEADER_CONTENT_TYPE.to_string(), MIME_JSON.to_string()),
        ]
    }

    async fn post(&self, url: &str, body: &serde_json::Value) -> Result<GotrueResponse, String> {
        let ctx = self.ctx()?;
        let payload = serde_json::to_string(body).map_err(|e| e.to_string())?;
        let response = transport::send(METHOD_POST, url, Self::headers(ctx), Some(payload))
            .await
            .map_err(|_| ERR_REQUEST.to_string())?;
        serde_json::from_str(response.text()).map_err(|e| e.to_string())
    }

    fn session(creds: &Credentials, response: GotrueResponse) -> Result<Session, String> {
        let token = response
            .access_token
            .ok_or_else(|| ERR_NO_TOKEN.to_string())?;
        let username = response
            .user
            .and_then(|u| u.email)
            .unwrap_or_else(|| creds.username.clone());
        Ok(Session { token, username })
    }

    /// Validates a Supabase access token and returns the owning email.
    pub async fn verify(&self, access_token: &str) -> Result<String, String> {
        let ctx = self.ctx()?;
        let url = format!("{}/{PATH_USER}", ctx.base_url);
        let mut headers = Self::headers(ctx);
        headers.push((
            HEADER_AUTHORIZATION.to_string(),
            format!("{BEARER_PREFIX}{access_token}"),
        ));
        let response = transport::send(METHOD_GET, &url, headers, None)
            .await
            .map_err(|_| ERR_TOKEN_REJECTED.to_string())?;
        if !(200..300).contains(&response.status) {
            return Err(ERR_TOKEN_REJECTED.to_string());
        }
        let user: GotrueUser = serde_json::from_str(response.text()).map_err(|e| e.to_string())?;
        user.email.ok_or_else(|| ERR_TOKEN_REJECTED.to_string())
    }
}

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
impl RemoteAuth for SupabaseRemoteAuth {
    async fn signup(&self, creds: &Credentials) -> Result<Session, String> {
        let url = format!("{}/{PATH_SIGNUP}", self.ctx()?.base_url);
        let body = json!({ "email": creds.username, "password": creds.password });
        let response = self.post(&url, &body).await?;
        Self::session(creds, response)
    }

    async fn login(&self, creds: &Credentials) -> Result<Session, String> {
        let url = format!(
            "{}/{PATH_TOKEN}?{QUERY_PASSWORD_GRANT}",
            self.ctx()?.base_url
        );
        let body = json!({ "email": creds.username, "password": creds.password });
        let response = self.post(&url, &body).await?;
        Self::session(creds, response)
    }
}
