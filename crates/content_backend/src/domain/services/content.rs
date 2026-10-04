use crate::app::dto::{ContentDto, UpsertContent};
use crate::domain::{Content, STATUS_PUBLISHED};
use crate::port::ContentRepo;

#[derive(Debug, Clone)]
pub struct ContentPage {
    pub page: u32,
    pub page_size: u32,
    pub items: Vec<ContentDto>,
    pub total_items: u32,
    pub total_pages: u32,
}

#[derive(Clone)]
pub struct ContentService<R: ContentRepo> {
    repo: R,
}

impl<R: ContentRepo> ContentService<R> {
    pub fn new(repo: R) -> Self {
        Self { repo }
    }

    pub async fn list(&self) -> Result<Vec<Content>, String> {
        let items = self.repo.list().await?;
        Ok(items.into_iter().map(Content::from).collect())
    }

    pub async fn get(&self, id: i32) -> Result<Option<Content>, String> {
        Ok(self.repo.get(id).await?.map(Content::from))
    }

    pub async fn get_by_slug(&self, slug: &str) -> Result<Option<Content>, String> {
        Ok(self.repo.get_by_slug(slug).await?.map(Content::from))
    }

    pub async fn create(&self, data: UpsertContent) -> Result<Content, String> {
        let mut content = data.into_domain();
        content.touch();
        let saved = self.repo.create(to_upsert(&content)).await?;
        Ok(saved.into())
    }

    pub async fn update(&self, id: i32, data: UpsertContent) -> Result<Content, String> {
        let existing = self.repo.get(id).await?.ok_or_else(not_found)?;
        let mut content: Content = existing.into();
        content.title = data.title;
        content.slug = if data.slug.is_empty() {
            crate::domain::value_objects::generate_slug(&content.title)
        } else {
            data.slug
        };
        content.body = data.body;
        content.status = data.status;
        content.touch();
        let saved = self.repo.update(id, to_upsert(&content)).await?;
        Ok(saved.into())
    }

    pub async fn publish(&self, id: i32) -> Result<Content, String> {
        let existing = self.repo.get(id).await?.ok_or_else(not_found)?;
        let mut content: Content = existing.into();
        content.status = STATUS_PUBLISHED.to_string();
        content.touch();
        let saved = self.repo.update(id, to_upsert(&content)).await?;
        Ok(saved.into())
    }

    pub async fn delete(&self, id: i32) -> Result<(), String> {
        self.repo.delete(id).await
    }

    pub async fn list_published(&self) -> Result<Vec<Content>, String> {
        let published: Vec<Content> = self
            .list()
            .await?
            .into_iter()
            .filter(|c| c.status == STATUS_PUBLISHED)
            .collect();
        Ok(published)
    }
}

fn not_found() -> String {
    "content not found".to_string()
}

fn to_upsert(content: &Content) -> UpsertContent {
    UpsertContent {
        title: content.title.clone(),
        slug: content.slug.clone(),
        body: content.body.clone(),
        status: content.status.clone(),
    }
}
