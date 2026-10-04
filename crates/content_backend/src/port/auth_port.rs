use async_trait::async_trait;

use crate::domain::entities::user::{Credentials, Session};

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
pub trait UserStore: crate::port::NativeSend {
    async fn exists(&self, username: &str) -> Result<bool, String>;
    async fn create(&self, username: &str, password_hash: &str) -> Result<String, String>;
    async fn password_hash(&self, username: &str) -> Result<Option<String>, String>;
}

pub trait PasswordHasher: crate::port::NativeSend {
    fn hash(&self, password: &str) -> Result<String, String>;
    fn verify(&self, password: &str, hash: &str) -> bool;
}

pub trait TokenIssuer: crate::port::NativeSend {
    fn issue(&self) -> String;
}

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
pub trait SettingsStore: crate::port::NativeSend {
    async fn load(&self, username: &str) -> Result<serde_json::Value, String>;
    async fn save(&self, username: &str, settings: &serde_json::Value) -> Result<(), String>;
}

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
pub trait RemoteAuth: crate::port::NativeSend {
    async fn signup(&self, creds: &Credentials) -> Result<Session, String>;
    async fn login(&self, creds: &Credentials) -> Result<Session, String>;
}
