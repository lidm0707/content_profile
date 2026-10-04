use async_trait::async_trait;
use serde_json::Value;
use supabase_client::{ClientConfig, count, create, delete, get, update};

use crate::port::http::{Http, HttpContext};

#[derive(Clone, Copy)]
pub struct SupabaseHttp;

fn client(ctx: &HttpContext) -> ClientConfig {
    let mut client = ClientConfig::new(ctx.base_url.clone(), ctx.anon_key.clone());
    if let Some(token) = ctx.jwt_token.clone() {
        client = client.with_jwt_token(token);
    }
    if let Some(key) = ctx.service_role_key.clone() {
        client = client.with_service_role_key(key);
    }
    client
}

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
impl Http for SupabaseHttp {
    async fn get(
        &self,
        ctx: &HttpContext,
        table: &str,
        query: &[(&str, &str)],
    ) -> Result<Vec<Value>, String> {
        get(&client(ctx), table, query).await
    }

    async fn create(
        &self,
        ctx: &HttpContext,
        table: &str,
        body: &Value,
    ) -> Result<Vec<Value>, String> {
        create(&client(ctx), table, body).await
    }

    async fn update(
        &self,
        ctx: &HttpContext,
        table: &str,
        id: i32,
        body: &Value,
    ) -> Result<Vec<Value>, String> {
        update(&client(ctx), table, id, body).await
    }

    async fn delete(&self, ctx: &HttpContext, table: &str, id: i32) -> Result<(), String> {
        delete(&client(ctx), table, id).await
    }

    async fn count(
        &self,
        ctx: &HttpContext,
        table: &str,
        filters: &[(&str, &str)],
    ) -> Result<u32, String> {
        count(&client(ctx), table, filters).await
    }
}
