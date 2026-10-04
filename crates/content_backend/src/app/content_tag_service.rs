use crate::app::dto::ContentTagDto;
use crate::port::ContentTagRepo;

#[derive(Clone)]
pub struct ContentTagService<R: ContentTagRepo> {
    repo: R,
}

impl<R: ContentTagRepo> ContentTagService<R> {
    pub fn new(repo: R) -> Self {
        Self { repo }
    }

    pub async fn list_for_content(&self, content_id: i32) -> Result<Vec<ContentTagDto>, String> {
        self.repo.list_for_content(content_id).await
    }

    pub async fn list_for_tag(&self, tag_id: i32) -> Result<Vec<ContentTagDto>, String> {
        self.repo.list_for_tag(tag_id).await
    }

    pub async fn content_ids_for_tag(&self, tag_id: i32) -> Result<Vec<i32>, String> {
        Ok(self
            .list_for_tag(tag_id)
            .await?
            .into_iter()
            .map(|row| row.content_id)
            .collect())
    }

    pub async fn add(&self, content_id: i32, tag_id: i32) -> Result<ContentTagDto, String> {
        self.repo.add(content_id, tag_id).await
    }

    pub async fn remove(&self, id: i32) -> Result<(), String> {
        self.repo.remove(id).await
    }

    pub async fn remove_pair(&self, content_id: i32, tag_id: i32) -> Result<(), String> {
        let rows = self.list_for_content(content_id).await?;
        for row in rows.iter().filter(|row| row.tag_id == tag_id) {
            if let Some(id) = row.id {
                self.repo.remove(id).await?;
            }
        }
        Ok(())
    }

    pub async fn set_tags(&self, content_id: i32, tag_ids: Vec<i32>) -> Result<(), String> {
        let current = self.list_for_content(content_id).await?;
        let existing: Vec<i32> = current.iter().map(|row| row.tag_id).collect();

        for row in current {
            if !tag_ids.contains(&row.tag_id)
                && let Some(id) = row.id
            {
                self.repo.remove(id).await?;
            }
        }

        for tag_id in tag_ids {
            if !existing.contains(&tag_id) {
                self.repo.add(content_id, tag_id).await?;
            }
        }
        Ok(())
    }
}
