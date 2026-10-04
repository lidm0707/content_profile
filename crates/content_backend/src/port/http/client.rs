use async_trait::async_trait;
use serde_json::Value;

use crate::port::http::context::HttpContext;

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
pub trait Http: crate::port::NativeSend {
    async fn get(
        &self,
        ctx: &HttpContext,
        table: &str,
        query: &[(&str, &str)],
    ) -> Result<Vec<Value>, String>;
    async fn create(
        &self,
        ctx: &HttpContext,
        table: &str,
        body: &Value,
    ) -> Result<Vec<Value>, String>;
    async fn update(
        &self,
        ctx: &HttpContext,
        table: &str,
        id: i32,
        body: &Value,
    ) -> Result<Vec<Value>, String>;
    async fn delete(&self, ctx: &HttpContext, table: &str, id: i32) -> Result<(), String>;
    async fn count(
        &self,
        ctx: &HttpContext,
        table: &str,
        filters: &[(&str, &str)],
    ) -> Result<u32, String>;
}
