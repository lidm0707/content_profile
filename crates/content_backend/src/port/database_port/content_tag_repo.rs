use async_trait::async_trait;

use crate::app::dto::ContentTagDto;

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
pub trait ContentTagRepo: crate::port::NativeSend {
    async fn list_for_content(&self, content_id: i32) -> Result<Vec<ContentTagDto>, String>;
    async fn list_for_tag(&self, tag_id: i32) -> Result<Vec<ContentTagDto>, String>;
    async fn add(&self, content_id: i32, tag_id: i32) -> Result<ContentTagDto, String>;
    async fn remove(&self, id: i32) -> Result<(), String>;
}
