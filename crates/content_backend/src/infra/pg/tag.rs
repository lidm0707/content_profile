use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::PgPool;

use super::content::PgContentRepo;
use crate::app::dto::{TagDto, UpsertTag};
use crate::port::TagRepo;

const SQL_LIST_TAGS: &str = "SELECT id, name, slug, parent_id, created_at, updated_at, synced_at FROM tags ORDER BY created_at DESC";
const SQL_GET_TAG: &str =
    "SELECT id, name, slug, parent_id, created_at, updated_at, synced_at FROM tags WHERE id = $1";
const SQL_GET_TAG_BY_SLUG: &str =
    "SELECT id, name, slug, parent_id, created_at, updated_at, synced_at FROM tags WHERE slug = $1";
const SQL_INSERT_TAG: &str = "INSERT INTO tags (name, slug, parent_id) VALUES ($1, $2, $3) RETURNING id, name, slug, parent_id, created_at, updated_at, synced_at";
const SQL_UPDATE_TAG: &str = "UPDATE tags SET name = $2, slug = $3, parent_id = $4, updated_at = now() WHERE id = $1 RETURNING id, name, slug, parent_id, created_at, updated_at, synced_at";
const SQL_DELETE_TAG: &str = "DELETE FROM tags WHERE id = $1";
const SQL_LIST_TAGS_BY_IDS: &str = "SELECT id, name, slug, parent_id, created_at, updated_at, synced_at FROM tags WHERE id = ANY($1) ORDER BY created_at DESC";
const SQL_UPSERT_TAG_SYNC: &str = "INSERT INTO tags (name, slug, created_at, updated_at, synced_at) VALUES ($1, $2, COALESCE($3, now()), $4, now()) ON CONFLICT (slug) DO UPDATE SET name = EXCLUDED.name, created_at = EXCLUDED.created_at, updated_at = COALESCE(EXCLUDED.updated_at, now()), synced_at = now() RETURNING id, name, slug, parent_id, created_at, updated_at, synced_at";
const SQL_SET_TAG_PARENT: &str = "UPDATE tags SET parent_id = $2 WHERE id = $1";

const ERR_NOT_FOUND: &str = "not found";

#[derive(Clone)]
pub struct PgTagRepo {
    pool: PgPool,
}

#[derive(sqlx::FromRow)]
struct TagRow {
    id: i32,
    name: String,
    slug: String,
    parent_id: Option<i32>,
    created_at: Option<DateTime<Utc>>,
    updated_at: Option<DateTime<Utc>>,
    synced_at: Option<DateTime<Utc>>,
}

impl From<TagRow> for TagDto {
    fn from(r: TagRow) -> Self {
        Self {
            id: Some(r.id),
            name: r.name,
            slug: r.slug,
            parent_id: r.parent_id,
            created_at: r.created_at,
            updated_at: r.updated_at,
            synced_at: r.synced_at,
        }
    }
}

impl PgTagRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub fn from_env() -> Self {
        Self::new(PgContentRepo::from_env().pool)
    }

    /// Sync-pull upsert: preserves remote timestamps instead of now(),
    /// keyed by slug. Parent is linked separately by [`Self::set_parent`].
    pub async fn upsert_sync(
        &self,
        name: &str,
        slug: &str,
        created_at: Option<DateTime<Utc>>,
        updated_at: Option<DateTime<Utc>>,
    ) -> Result<TagDto, String> {
        sqlx::query_as::<_, TagRow>(SQL_UPSERT_TAG_SYNC)
            .bind(name)
            .bind(slug)
            .bind(created_at)
            .bind(updated_at)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| e.to_string())
            .map(TagDto::from)
    }

    /// Links a parent without touching updated_at (sync-pull pass 2).
    pub async fn set_parent(&self, id: i32, parent_id: Option<i32>) -> Result<(), String> {
        sqlx::query(SQL_SET_TAG_PARENT)
            .bind(id)
            .bind(parent_id)
            .execute(&self.pool)
            .await
            .map_err(|e| e.to_string())?;
        Ok(())
    }
}

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
impl TagRepo for PgTagRepo {
    async fn list(&self) -> Result<Vec<TagDto>, String> {
        let rows: Vec<TagRow> = sqlx::query_as(SQL_LIST_TAGS)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| e.to_string())?;
        Ok(rows.into_iter().map(TagDto::from).collect())
    }

    async fn get(&self, id: i32) -> Result<Option<TagDto>, String> {
        sqlx::query_as::<_, TagRow>(SQL_GET_TAG)
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| e.to_string())
            .map(|opt| opt.map(TagDto::from))
    }

    async fn get_by_slug(&self, slug: &str) -> Result<Option<TagDto>, String> {
        sqlx::query_as::<_, TagRow>(SQL_GET_TAG_BY_SLUG)
            .bind(slug)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| e.to_string())
            .map(|opt| opt.map(TagDto::from))
    }

    async fn create(&self, data: UpsertTag) -> Result<TagDto, String> {
        sqlx::query_as::<_, TagRow>(SQL_INSERT_TAG)
            .bind(data.name)
            .bind(data.slug)
            .bind(data.parent_id)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| e.to_string())
            .map(TagDto::from)
    }

    async fn update(&self, id: i32, data: UpsertTag) -> Result<TagDto, String> {
        sqlx::query_as::<_, TagRow>(SQL_UPDATE_TAG)
            .bind(id)
            .bind(data.name)
            .bind(data.slug)
            .bind(data.parent_id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| e.to_string())?
            .map(TagDto::from)
            .ok_or_else(|| ERR_NOT_FOUND.to_string())
    }

    async fn delete(&self, id: i32) -> Result<(), String> {
        let result = sqlx::query(SQL_DELETE_TAG)
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| e.to_string())?;
        if result.rows_affected() == 0 {
            return Err(ERR_NOT_FOUND.to_string());
        }
        Ok(())
    }

    async fn list_by_ids(&self, ids: &[i32]) -> Result<Vec<TagDto>, String> {
        if ids.is_empty() {
            return Ok(Vec::new());
        }
        let rows: Vec<TagRow> = sqlx::query_as(SQL_LIST_TAGS_BY_IDS)
            .bind(ids)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| e.to_string())?;
        Ok(rows.into_iter().map(TagDto::from).collect())
    }
}
