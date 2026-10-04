use async_trait::async_trait;

use crate::app::dto::{TagDto, UpsertTag};
use crate::domain::Tag;

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
pub trait TagRepo: crate::port::NativeSend {
    async fn list(&self) -> Result<Vec<TagDto>, String>;
    async fn get(&self, id: i32) -> Result<Option<TagDto>, String>;
    async fn get_by_slug(&self, slug: &str) -> Result<Option<TagDto>, String>;
    async fn create(&self, data: UpsertTag) -> Result<TagDto, String>;
    async fn update(&self, id: i32, data: UpsertTag) -> Result<TagDto, String>;
    async fn delete(&self, id: i32) -> Result<(), String>;
    async fn list_by_ids(&self, ids: &[i32]) -> Result<Vec<TagDto>, String> {
        let _ = ids;
        Err("list_by_ids not supported".into())
    }
}

impl UpsertTag {
    pub fn into_domain(self) -> Tag {
        Tag::new(self.name, self.slug, self.parent_id)
    }
}
