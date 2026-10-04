use serde_json::json;
use std::collections::HashMap;

use sqlx::PgPool;

use super::queue::{self, ENTITY_TAG, OP_DELETE, OP_UPSERT};
use crate::app::dto::{TagDto, UpsertTag};
use crate::infra::pg::pool_from_env;
use crate::infra::pg::tag::PgTagRepo;
use crate::infra::supabase::tag::SupabaseTagRepo;
use crate::port::TagRepo;

#[derive(Clone)]
pub struct PgSyncTagRepo {
    local: PgTagRepo,
    remote: SupabaseTagRepo,
    pool: PgPool,
}

impl PgSyncTagRepo {
    pub fn new(supabase: Option<crate::infra::supabase::SupabaseConfig>) -> Self {
        Self {
            local: PgTagRepo::from_env(),
            remote: SupabaseTagRepo::new(supabase),
            pool: pool_from_env(),
        }
    }

    async fn push_upsert(&self, slug: &str, data: &UpsertTag) -> Result<(), String> {
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

    pub async fn flush_pending(&self) -> Result<usize, String> {
        let rows = queue::pending(&self.pool, ENTITY_TAG).await?;

        let mut pushed = 0;
        for row in rows {
            let result = match row.op.as_str() {
                OP_UPSERT => match serde_json::from_value::<UpsertTag>(row.payload.clone()) {
                    Ok(data) => self.push_upsert(&row.slug, &data).await,
                    Err(e) => Err(e.to_string()),
                },
                OP_DELETE => self.push_delete(&row.slug).await,
                _ => Ok(()),
            };
            if let Err(e) = result {
                tracing::warn!("tag sync retry deferred: {e}");
                continue;
            }
            queue::dequeue(&self.pool, row.id).await?;
            pushed += 1;
        }
        Ok(pushed)
    }

    /// Pulls every remote tag into local pg, upserting by slug.
    /// Writes go straight to pg — no re-push to the remote.
    ///
    /// Two passes: all rows are upserted with `parent_id = None` first
    /// (local ids differ from remote ids, and children may arrive before
    /// their parents), then parents are linked via a remote-id → local-id
    /// slug map. This satisfies `tags_parent_id_fkey` regardless of order.
    pub async fn pull_remote(&self) -> Result<usize, String> {
        const NO_ID: &str = "remote tag row has no id";

        let remote = self.remote.list().await?;
        let slug_of_remote: HashMap<i32, String> = remote
            .iter()
            .filter_map(|dto| Some((dto.id?, dto.slug.clone())))
            .collect();

        let mut pulled_rows: Vec<(i32, Option<i32>)> = Vec::new();
        for dto in &remote {
            let local_id = self
                .local
                .upsert_sync(&dto.name, &dto.slug, dto.created_at, dto.updated_at)
                .await?
                .id
                .ok_or_else(|| NO_ID.to_string())?;
            pulled_rows.push((local_id, dto.parent_id));
        }

        let local_by_slug: HashMap<&str, i32> = pulled_rows
            .iter()
            .zip(remote.iter())
            .map(|((id, _), dto)| (dto.slug.as_str(), *id))
            .collect();
        for (local_id, remote_parent) in &pulled_rows {
            let Some(remote_parent) = remote_parent else {
                continue;
            };
            let Some(parent_slug) = slug_of_remote.get(remote_parent) else {
                continue;
            };
            let Some(parent_local) = local_by_slug.get(parent_slug.as_str()) else {
                continue;
            };
            if parent_local == local_id {
                continue;
            }
            self.local
                .set_parent(*local_id, Some(*parent_local))
                .await?;
        }
        Ok(pulled_rows.len())
    }
}

#[async_trait::async_trait]
impl TagRepo for PgSyncTagRepo {
    async fn list(&self) -> Result<Vec<TagDto>, String> {
        self.local.list().await
    }

    async fn get(&self, id: i32) -> Result<Option<TagDto>, String> {
        self.local.get(id).await
    }

    async fn get_by_slug(&self, slug: &str) -> Result<Option<TagDto>, String> {
        self.local.get_by_slug(slug).await
    }

    async fn create(&self, data: UpsertTag) -> Result<TagDto, String> {
        let dto = self.local.create(data.clone()).await?;
        if let Err(e) = self.push_upsert(&dto.slug, &data).await {
            let payload = serde_json::to_value(&data).unwrap_or_default();
            queue::enqueue(&self.pool, ENTITY_TAG, OP_UPSERT, &dto.slug, payload).await;
            tracing::warn!("tag sync deferred: {e}");
        }
        Ok(dto)
    }

    async fn update(&self, id: i32, data: UpsertTag) -> Result<TagDto, String> {
        let dto = self.local.update(id, data.clone()).await?;
        if let Err(e) = self.push_upsert(&dto.slug, &data).await {
            let payload = serde_json::to_value(&data).unwrap_or_default();
            queue::enqueue(&self.pool, ENTITY_TAG, OP_UPSERT, &dto.slug, payload).await;
            tracing::warn!("tag sync deferred: {e}");
        }
        Ok(dto)
    }

    async fn delete(&self, id: i32) -> Result<(), String> {
        let Some(dto) = self.local.get(id).await? else {
            return Ok(());
        };
        self.local.delete(id).await?;
        if let Err(e) = self.push_delete(&dto.slug).await {
            let payload = json!({ "slug": dto.slug });
            queue::enqueue(&self.pool, ENTITY_TAG, OP_DELETE, &dto.slug, payload).await;
            tracing::warn!("tag delete sync deferred: {e}");
        }
        Ok(())
    }

    async fn list_by_ids(&self, ids: &[i32]) -> Result<Vec<TagDto>, String> {
        self.local.list_by_ids(ids).await
    }
}
