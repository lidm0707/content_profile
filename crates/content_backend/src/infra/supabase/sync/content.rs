use sqlx::PgPool;

use super::queue::{self, ENTITY_CONTENT, OP_DELETE, OP_UPSERT};
use crate::app::dto::{ContentDto, UpsertContent};
use crate::domain::STATUS_DRAFT;
use crate::infra::pg::content::PgContentRepo;
use crate::infra::pg::pool_from_env;
use crate::infra::supabase::content::SupabaseContentRepo;
use crate::port::ContentRepo;

#[derive(Clone)]
pub struct PgSyncContentRepo {
    local: PgContentRepo,
    remote: SupabaseContentRepo,
    pool: PgPool,
}

impl PgSyncContentRepo {
    pub fn new(supabase: Option<crate::infra::supabase::SupabaseConfig>) -> Self {
        Self {
            local: PgContentRepo::from_env(),
            remote: SupabaseContentRepo::new(supabase),
            pool: pool_from_env(),
        }
    }

    async fn push_upsert(&self, slug: &str, data: &UpsertContent) -> Result<(), String> {
        match self.remote.get_by_slug(slug).await? {
            Some(dto) => {
                let id = dto.id.ok_or_else(|| queue::ERR_REMOTE_ROW.to_string())?;
                self.remote.update(id, data.clone()).await?;
            }
            None => {
                self.remote.create(data.clone()).await?;
            }
        }
        Ok(())
    }

    async fn push_delete(&self, slug: &str) -> Result<(), String> {
        if let Some(dto) = self.remote.get_by_slug(slug).await? {
            let id = dto.id.ok_or_else(|| queue::ERR_REMOTE_ROW.to_string())?;
            self.remote.delete(id).await?;
        }
        Ok(())
    }

    /// Post-save remote handling. Drafts stay local: a slug never pushed
    /// is ignored (no push), one that already exists on the remote is
    /// queued so the later flush pushes the edit (wait to push). Any
    /// other status pushes immediately.
    async fn after_local_save(&self, slug: &str, data: &UpsertContent) -> Result<(), String> {
        if data.status == STATUS_DRAFT {
            match self.remote.get_by_slug(slug).await {
                Ok(Some(_)) => {
                    let payload = serde_json::to_value(data).unwrap_or_default();
                    queue::enqueue(&self.pool, ENTITY_CONTENT, OP_UPSERT, slug, payload).await;
                }
                Ok(None) => {}
                Err(e) => {
                    // Unknown remote state — queue so the flush reconciles it.
                    let payload = serde_json::to_value(data).unwrap_or_default();
                    queue::enqueue(&self.pool, ENTITY_CONTENT, OP_UPSERT, slug, payload).await;
                    tracing::warn!("draft push-state check deferred: {e}");
                }
            }
            return Ok(());
        }
        if let Err(e) = self.push_upsert(slug, data).await {
            let payload = serde_json::to_value(data).unwrap_or_default();
            queue::enqueue(&self.pool, ENTITY_CONTENT, OP_UPSERT, slug, payload).await;
            tracing::warn!("content sync deferred: {e}");
        }
        self.local.mark_synced(slug).await?;
        Ok(())
    }

    pub async fn flush_pending(&self) -> Result<usize, String> {
        let rows = queue::pending(&self.pool, ENTITY_CONTENT).await?;

        let mut pushed = 0;
        for row in rows {
            let result = match row.op.as_str() {
                OP_UPSERT => match serde_json::from_value::<UpsertContent>(row.payload.clone()) {
                    Ok(data) => {
                        self.push_upsert(&row.slug, &data).await?;
                        self.local.mark_synced(&row.slug).await?;
                        Ok(())
                    }
                    Err(e) => Err(e.to_string()),
                },
                OP_DELETE => self.push_delete(&row.slug).await,
                _ => Ok(()),
            };
            if let Err(e) = result {
                tracing::warn!("content sync retry deferred: {e}");
                continue;
            }
            queue::dequeue(&self.pool, row.id).await?;
            pushed += 1;
        }
        Ok(pushed)
    }

    /// Pulls every remote content row into local pg, upserting by slug.
    /// Writes go straight to pg — no re-push to the remote.
    /// Remote created_at/updated_at are preserved.
    pub async fn pull_remote(&self) -> Result<usize, String> {
        let remote = self.remote.list().await?;
        for dto in &remote {
            self.local
                .upsert_sync(
                    &dto.title,
                    &dto.slug,
                    &dto.body,
                    &dto.status,
                    dto.created_at,
                    dto.updated_at,
                )
                .await?;
        }
        Ok(remote.len())
    }
}

#[async_trait::async_trait]
impl ContentRepo for PgSyncContentRepo {
    async fn list(&self) -> Result<Vec<ContentDto>, String> {
        self.local.list().await
    }

    async fn get(&self, id: i32) -> Result<Option<ContentDto>, String> {
        self.local.get(id).await
    }

    async fn get_by_slug(&self, slug: &str) -> Result<Option<ContentDto>, String> {
        self.local.get_by_slug(slug).await
    }

    async fn create(&self, data: UpsertContent) -> Result<ContentDto, String> {
        let dto = self.local.create(data.clone()).await?;
        self.after_local_save(&dto.slug, &data).await?;
        Ok(dto)
    }

    async fn update(&self, id: i32, data: UpsertContent) -> Result<ContentDto, String> {
        let dto = self.local.update(id, data.clone()).await?;
        self.after_local_save(&dto.slug, &data).await?;
        Ok(dto)
    }

    async fn delete(&self, id: i32) -> Result<(), String> {
        let Some(dto) = self.local.get(id).await? else {
            return Ok(());
        };
        self.local.delete(id).await?;
        if let Err(e) = self.push_delete(&dto.slug).await {
            let payload = serde_json::json!({ "slug": dto.slug });
            queue::enqueue(&self.pool, ENTITY_CONTENT, OP_DELETE, &dto.slug, payload).await;
            tracing::warn!("content delete sync deferred: {e}");
        }
        Ok(())
    }

    async fn list_page(&self, offset: u32, limit: u32) -> Result<Vec<ContentDto>, String> {
        self.local.list_page(offset, limit).await
    }

    async fn count(&self) -> Result<u32, String> {
        self.local.count().await
    }

    async fn list_by_ids(&self, ids: &[i32]) -> Result<Vec<ContentDto>, String> {
        self.local.list_by_ids(ids).await
    }
}
