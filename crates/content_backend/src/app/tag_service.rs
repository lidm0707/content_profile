use crate::app::dto::UpsertTag;
use crate::port::TagRepo;
use crate::domain::Tag;

#[derive(Clone)]
pub struct TagService<R: TagRepo> {
    repo: R,
}

impl<R: TagRepo> TagService<R> {
    pub fn new(repo: R) -> Self {
        Self { repo }
    }

    pub async fn list(&self) -> Result<Vec<Tag>, String> {
        let items = self.repo.list().await?;
        Ok(items.into_iter().map(Tag::from).collect())
    }

    pub async fn get(&self, id: i32) -> Result<Option<Tag>, String> {
        Ok(self.repo.get(id).await?.map(Tag::from))
    }

    pub async fn create(&self, data: UpsertTag) -> Result<Tag, String> {
        let saved = self.repo.create(data).await?;
        Ok(saved.into())
    }

    pub async fn update(&self, id: i32, data: UpsertTag) -> Result<Tag, String> {
        let saved = self.repo.update(id, data).await?;
        Ok(saved.into())
    }

    pub async fn delete(&self, id: i32) -> Result<(), String> {
        self.repo.delete(id).await
    }
}
