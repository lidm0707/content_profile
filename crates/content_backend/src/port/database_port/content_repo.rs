use async_trait::async_trait;

use crate::app::dto::{ContentDto, UpsertContent};
use crate::domain::Content;

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
pub trait ContentRepo: crate::port::NativeSend {
    async fn list(&self) -> Result<Vec<ContentDto>, String>;
    async fn get(&self, id: i32) -> Result<Option<ContentDto>, String>;
    async fn get_by_slug(&self, slug: &str) -> Result<Option<ContentDto>, String>;
    async fn create(&self, data: UpsertContent) -> Result<ContentDto, String>;
    async fn update(&self, id: i32, data: UpsertContent) -> Result<ContentDto, String>;
    async fn delete(&self, id: i32) -> Result<(), String>;
    async fn list_page(&self, offset: u32, limit: u32) -> Result<Vec<ContentDto>, String> {
        let _ = (offset, limit);
        Err("pagination not supported".into())
    }
    async fn count(&self) -> Result<u32, String> {
        Err("count not supported".into())
    }
    async fn list_by_ids(&self, ids: &[i32]) -> Result<Vec<ContentDto>, String> {
        let _ = ids;
        Err("list_by_ids not supported".into())
    }
}

impl UpsertContent {
    pub fn into_domain(self) -> Content {
        let mut content = Content::new(self.title, self.slug, self.body);
        content.status = self.status;
        content
    }
}
