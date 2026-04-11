//! # Supabase Login System Example
//!
//! A simple authentication example using Supabase with Dioxus 0.7.

use dioxus::prelude::*;

use gloo_net::http::Request;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

const SUPABASE_URL: &str = env!("SUPABASE_URL");
const SUPABASE_ANON_KEY: &str = env!("SUPABASE_ANON_KEY");
const SESSION_STORAGE_KEY: &str = "supabase_session";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct User {
    pub id: String,
    pub email: String,
    pub email_confirmed_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub last_sign_in_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Session {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_at: i64,
    pub token_type: String,
    pub user: User,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_at: i64,
    pub token_type: String,
    pub user: User,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthError {
    pub error: String,
    pub error_description: String,
}

#[derive(Clone)]
pub struct AuthService {
    base_url: String,
    anon_key: String,
}

impl AuthService {
    pub fn new() -> Self {
        Self {
            base_url: SUPABASE_URL.to_string(),
            anon_key: SUPABASE_ANON_KEY.to_string(),
        }
    }

    pub async fn login(&self, request: LoginRequest) -> Result<Session, String> {
        let url = format!("{}/auth/v1/token?grant_type=password", self.base_url);
        let body = serde_json::to_string(&request)
            .map_err(|e| format!("Failed to serialize request: {}", e))?;

        let req = Request::post(&url)
            .header("apikey", &self.anon_key)
            .header("Content-Type", "application/json")
            .body(body)
            .map_err(|e| format!("Failed to build request: {}", e))?;

        let response = req
            .send()
            .await
            .map_err(|e| format!("Network error: {}", e))?;

        if response.ok() {
            let auth_response: AuthResponse = response
                .json()
                .await
                .map_err(|e| format!("Failed to parse response: {}", e))?;
            Ok(Session {
                access_token: auth_response.access_token,
                refresh_token: auth_response.refresh_token,
                expires_at: auth_response.expires_at,
                token_type: auth_response.token_type,
                user: auth_response.user,
            })
        } else {
            let error: AuthError = response.json().await.unwrap_or(AuthError {
                error: "unknown".to_string(),
                error_description: "Unknown error".to_string(),
            });
            Err(error.error_description)
        }
    }

    pub async fn signup(&self, request: LoginRequest) -> Result<Session, String> {
        let url = format!("{}/auth/v1/signup", self.base_url);
        let body = serde_json::to_string(&request)
            .map_err(|e| format!("Failed to serialize request: {}", e))?;

        let req = Request::post(&url)
            .header("apikey", &self.anon_key)
            .header("Content-Type", "application/json")
            .body(body)
            .map_err(|e| format!("Failed to build request: {}", e))?;

        let response = req
            .send()
            .await
            .map_err(|e| format!("Network error: {}", e))?;

        if response.ok() {
            let auth_response: AuthResponse = response
                .json()
                .await
                .map_err(|e| format!("Failed to parse response: {}", e))?;
            Ok(Session {
                access_token: auth_response.access_token,
                refresh_token: auth_response.refresh_token,
                expires_at: auth_response.expires_at,
                token_type: auth_response.token_type,
                user: auth_response.user,
            })
        } else {
            let error: AuthError = response.json().await.unwrap_or(AuthError {
                error: "unknown".to_string(),
                error_description: "Unknown error".to_string(),
            });
            Err(error.error_description)
        }
    }

    pub async fn logout(&self, _access_token: &str) -> Result<(), String> {
        Ok(())
    }

    pub async fn refresh_token(&self, _refresh_token: &str) -> Result<Session, String> {
        Err("Token refresh not implemented".to_string())
    }

    pub async fn get_user(&self, access_token: &str) -> Result<User, String> {
        let url = format!("{}/auth/v1/user", self.base_url);

        let req = Request::get(&url)
            .header("apikey", &self.anon_key)
            .header("Authorization", &format!("Bearer {}", access_token))
            .build()
            .map_err(|e| format!("Failed to build request: {}", e))?;

        let response = req
            .send()
            .await
            .map_err(|e| format!("Network error: {}", e))?;

        if response.ok() {
            response
                .json()
                .await
                .map_err(|e| format!("Failed to parse response: {}", e))
        } else {
            Err("Failed to get user".to_string())
        }
    }
}

impl Default for AuthService {
    fn default() -> Self {
        Self::new()
    }
}

pub struct SessionStorage;

impl SessionStorage {
    pub fn save_session(session: &Session) -> Result<(), String> {
        let json = serde_json::to_string(session)
            .map_err(|e| format!("Failed to serialize session: {}", e))?;
        let storage = web_sys::window()
            .ok_or("No window object")?
            .local_storage()
            .map_err(|e| format!("Failed to get localStorage: {:?}", e))?
            .ok_or("localStorage not available")?;

        storage
            .set_item(SESSION_STORAGE_KEY, &json)
            .map_err(|e| format!("Failed to save session: {:?}", e))?;
        Ok(())
    }

    pub fn load_session() -> Result<Option<Session>, String> {
        let storage = web_sys::window()
            .ok_or("No window object")?
            .local_storage()
            .map_err(|e| format!("Failed to get localStorage: {:?}", e))?
            .ok_or("localStorage not available")?;

        let js_value = storage
            .get_item(SESSION_STORAGE_KEY)
            .map_err(|e| format!("Failed to get item: {:?}", e))?;

        match js_value {
            Some(val) => {
                let session = serde_json::from_str(&val)
                    .map_err(|e| format!("Failed to deserialize session: {}", e))?;
                Ok(Some(session))
            }
            None => Ok(None),
        }
    }

    pub fn clear_session() -> Result<(), String> {
        let storage = web_sys::window()
            .ok_or("No window object")?
            .local_storage()
            .map_err(|e| format!("Failed to get localStorage: {:?}", e))?
            .ok_or("localStorage not available")?;

        storage
            .remove_item(SESSION_STORAGE_KEY)
            .map_err(|e| format!("Failed to clear session: {:?}", e))?;
        Ok(())
    }

    pub fn has_valid_session() -> bool {
        Self::load_session().ok().flatten().is_some()
    }
}

impl Clone for UserContext {
    fn clone(&self) -> Self {
        Self {
            auth_service: Arc::clone(&self.auth_service),
        }
    }
}

pub struct UserContext {
    auth_service: Arc<AuthService>,
}

impl UserContext {
    pub fn new() -> Self {
        Self {
            auth_service: Arc::new(AuthService::new()),
        }
    }

    pub async fn login(&self, email: String, password: String) -> Result<Session, String> {
        let request = LoginRequest { email, password };
        let session = self.auth_service.login(request).await?;
        SessionStorage::save_session(&session)?;
        Ok(session)
    }

    pub async fn signup(&self, email: String, password: String) -> Result<Session, String> {
        let request = LoginRequest { email, password };
        let session = self.auth_service.signup(request).await?;
        SessionStorage::save_session(&session)?;
        Ok(session)
    }

    pub async fn logout(&self, access_token: Option<String>) -> Result<(), String> {
        if let Some(token) = access_token {
            let _ = self.auth_service.logout(&token).await;
        }
        SessionStorage::clear_session()
    }

    pub async fn get_user(&self, access_token: &str) -> Result<User, String> {
        self.auth_service.get_user(access_token).await
    }
}

impl Default for UserContext {
    fn default() -> Self {
        Self::new()
    }
}

#[component]
fn LoginForm() -> Element {
    let user_context = use_context::<UserContext>();
    let mut email = use_signal(String::new);
    let mut password = use_signal(String::new);
    let mut error = use_signal(|| Option::<String>::None);
    let mut loading = use_signal(|| false);
    let mut is_login = use_signal(|| true);

    rsx! {
        div { class: "min-h-screen flex items-center justify-center bg-gray-100",
            div { class: "bg-white p-8 rounded-lg shadow-md w-full max-w-md",
                h1 { class: "text-2xl font-bold mb-6 text-center",
                    {if is_login() { "Login" } else { "Sign Up" }}
                }

                if let Some(err) = error() {
                    div { class: "bg-red-100 border border-red-400 text-red-700 px-4 py-3 rounded mb-4",
                        "{err}"
                    }
                }

                div { class: "mb-4",
                    label { class: "block text-gray-700 text-sm font-bold mb-2", "Email" }
                    input {
                        r#type: "email",
                        value: "{email}",
                        oninput: move |e| {*email.write() = e.value();},
                        class: "w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500",
                        placeholder: "you@example.com"
                    }
                }

                div { class: "mb-6",
                    label { class: "block text-gray-700 text-sm font-bold mb-2", "Password" }
                    input {
                        r#type: "password",
                        value: "{password}",
                        oninput: move |e| {*password.write() = e.value();},
                        class: "w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500",
                        placeholder: "••••••••"
                    }
                }

                button {
                    onclick: move |_| {
                        let email_val = email.read().clone();
                        let password_val = password.read().clone();
                        let user_context = user_context.clone();
                        let is_login_mode = is_login();

                        if email_val.is_empty() || password_val.is_empty() {
                            *error.write() = Some("Email and password are required".to_string());
                            return;
                        }

                        *loading.write() = true;

                        let _ = spawn(async move {
                            let result = if is_login_mode {
                                user_context.login(email_val, password_val).await
                            } else {
                                user_context.signup(email_val, password_val).await
                            };

                            match result {
                                Ok(_) => {
                                    *loading.write() = false;
                                }
                                Err(e) => {
                                    *error.write() = Some(e);
                                    *loading.write() = false;
                                }
                            }
                        });
                    },
                    disabled: loading(),
                    class: "w-full bg-blue-500 text-white font-bold py-2 px-4 rounded-md hover:bg-blue-600 disabled:opacity-50 disabled:cursor-not-allowed",
                    {
                        if loading() {
                            "Loading..."
                        } else if is_login() {
                            "Login"
                        } else {
                            "Sign Up"
                        }
                    }
                }

                div { class: "mt-4 text-center",
                    button {
                        onclick: move |_| {is_login.set(!is_login());},
                        class: "text-blue-500 hover:underline",
                        {
                            if is_login() {
                                "Don't have an account? Sign up"
                            } else {
                                "Already have an account? Login"
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn App() -> Element {
    let user_context = use_context_provider(|| UserContext::new());
    let mut session = use_signal(|| Option::<Session>::None);

    let _ = use_resource(move || async move {
        match SessionStorage::load_session() {
            Ok(Some(s)) => session.set(Some(s)),
            _ => session.set(None),
        }
    });

    rsx! {
        if session().is_some() {
            if let Some(ref s) = session() {
                div { class: "min-h-screen flex items-center justify-center bg-gray-100",
                    div { class: "bg-white p-8 rounded-lg shadow-md w-full max-w-md",
                        h1 { class: "text-2xl font-bold mb-4", "Welcome, {s.user.email}!" }
                        p { class: "text-gray-600 mb-6", "You are logged in." }
                        button {
                            onclick: move |_| {
                                let user_context = user_context.clone();
                                let access_token = session().as_ref().map(|s| s.access_token.clone());
                                spawn(async move {
                                    let _ = user_context.logout(access_token).await;
                                });
                                session.set(None);
                            },
                            class: "w-full bg-red-500 text-white font-bold py-2 px-4 rounded-md hover:bg-red-600",
                            "Logout"
                        }
                    }
                }
            }
        } else {
            LoginForm {}
        }
    }
}

fn main() {
    dioxus::launch(App);
}
