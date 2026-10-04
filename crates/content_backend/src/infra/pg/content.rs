use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::PgPool;

use crate::app::dto::{ContentDto, UpsertContent};
use crate::port::ContentRepo;

const SQL_LIST_CONTENT: &str = "SELECT id, title, slug, body, status, created_at, updated_at, synced_at FROM content ORDER BY created_at DESC";
const SQL_LIST_CONTENT_PAGE: &str = "SELECT id, title, slug, body, status, created_at, updated_at, synced_at FROM content ORDER BY created_at DESC LIMIT $1 OFFSET $2";
const SQL_COUNT_CONTENT: &str = "SELECT count(*)::BIGINT FROM content";
const SQL_LIST_CONTENT_BY_IDS: &str = "SELECT id, title, slug, body, status, created_at, updated_at, synced_at FROM content WHERE id = ANY($1) ORDER BY created_at DESC";
const SQL_GET_CONTENT: &str = "SELECT id, title, slug, body, status, created_at, updated_at, synced_at FROM content WHERE id = $1";
const SQL_GET_CONTENT_BY_SLUG: &str = "SELECT id, title, slug, body, status, created_at, updated_at, synced_at FROM content WHERE slug = $1";
const SQL_INSERT_CONTENT: &str = "INSERT INTO content (title, slug, body, status) VALUES ($1, $2, $3, $4) RETURNING id, title, slug, body, status, created_at, updated_at, synced_at";
const SQL_UPDATE_CONTENT: &str = "UPDATE content SET title = $2, slug = $3, body = $4, status = $5, updated_at = now(), synced_at = NULL WHERE id = $1 RETURNING id, title, slug, body, status, created_at, updated_at, synced_at";
const SQL_DELETE_CONTENT: &str = "DELETE FROM content WHERE id = $1";
const SQL_UPSERT_CONTENT_SYNC: &str = "INSERT INTO content (title, slug, body, status, created_at, updated_at, synced_at) VALUES ($1, $2, $3, $4, COALESCE($5, now()), $6, now()) ON CONFLICT (slug) DO UPDATE SET title = EXCLUDED.title, body = EXCLUDED.body, status = EXCLUDED.status, created_at = EXCLUDED.created_at, updated_at = COALESCE(EXCLUDED.updated_at, now()), synced_at = now() RETURNING id, title, slug, body, status, created_at, updated_at, synced_at";
const SQL_MARK_SYNCED: &str = "UPDATE content SET synced_at = now() WHERE slug = $1";

const ERR_NOT_FOUND: &str = "not found";

#[derive(Clone)]
pub struct PgContentRepo {
    pub(super) pool: PgPool,
}

#[derive(sqlx::FromRow)]
struct ContentRow {
    id: i32,
    title: String,
    slug: String,
    body: String,
    status: String,
    created_at: Option<DateTime<Utc>>,
    updated_at: Option<DateTime<Utc>>,
    synced_at: Option<DateTime<Utc>>,
}

impl From<ContentRow> for ContentDto {
    fn from(r: ContentRow) -> Self {
        Self {
            id: Some(r.id),
            title: r.title,
            slug: r.slug,
            body: r.body,
            status: r.status,
            created_at: r.created_at,
            updated_at: r.updated_at,
            synced_at: r.synced_at,
        }
    }
}

impl PgContentRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub fn from_env() -> Self {
        Self::new(super::pool_from_env())
    }

    /// Marks a row as pushed to the remote (after a successful push).
    pub async fn mark_synced(&self, slug: &str) -> Result<(), String> {
        sqlx::query(SQL_MARK_SYNCED)
            .bind(slug)
            .execute(&self.pool)
            .await
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    /// Sync-pull upsert: preserves remote timestamps instead of now(),
    /// keyed by slug.
    pub async fn upsert_sync(
        &self,
        title: &str,
        slug: &str,
        body: &str,
        status: &str,
        created_at: Option<DateTime<Utc>>,
        updated_at: Option<DateTime<Utc>>,
    ) -> Result<ContentDto, String> {
        sqlx::query_as::<_, ContentRow>(SQL_UPSERT_CONTENT_SYNC)
            .bind(title)
            .bind(slug)
            .bind(body)
            .bind(status)
            .bind(created_at)
            .bind(updated_at)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| e.to_string())
            .map(ContentDto::from)
    }
}

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
impl ContentRepo for PgContentRepo {
    async fn list(&self) -> Result<Vec<ContentDto>, String> {
        let rows: Vec<ContentRow> = sqlx::query_as(SQL_LIST_CONTENT)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| e.to_string())?;
        Ok(rows.into_iter().map(ContentDto::from).collect())
    }

    async fn get(&self, id: i32) -> Result<Option<ContentDto>, String> {
        sqlx::query_as::<_, ContentRow>(SQL_GET_CONTENT)
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| e.to_string())
            .map(|opt| opt.map(ContentDto::from))
    }

    async fn get_by_slug(&self, slug: &str) -> Result<Option<ContentDto>, String> {
        sqlx::query_as::<_, ContentRow>(SQL_GET_CONTENT_BY_SLUG)
            .bind(slug)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| e.to_string())
            .map(|opt| opt.map(ContentDto::from))
    }

    async fn create(&self, data: UpsertContent) -> Result<ContentDto, String> {
        sqlx::query_as::<_, ContentRow>(SQL_INSERT_CONTENT)
            .bind(data.title)
            .bind(data.slug)
            .bind(data.body)
            .bind(data.status)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| e.to_string())
            .map(ContentDto::from)
    }

    async fn update(&self, id: i32, data: UpsertContent) -> Result<ContentDto, String> {
        sqlx::query_as::<_, ContentRow>(SQL_UPDATE_CONTENT)
            .bind(id)
            .bind(data.title)
            .bind(data.slug)
            .bind(data.body)
            .bind(data.status)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| e.to_string())?
            .map(ContentDto::from)
            .ok_or_else(|| ERR_NOT_FOUND.to_string())
    }

    async fn delete(&self, id: i32) -> Result<(), String> {
        let result = sqlx::query(SQL_DELETE_CONTENT)
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| e.to_string())?;
        if result.rows_affected() == 0 {
            return Err(ERR_NOT_FOUND.to_string());
        }
        Ok(())
    }

    async fn list_page(&self, offset: u32, limit: u32) -> Result<Vec<ContentDto>, String> {
        let rows: Vec<ContentRow> = sqlx::query_as(SQL_LIST_CONTENT_PAGE)
            .bind(i64::from(limit))
            .bind(i64::from(offset))
            .fetch_all(&self.pool)
            .await
            .map_err(|e| e.to_string())?;
        Ok(rows.into_iter().map(ContentDto::from).collect())
    }

    async fn count(&self) -> Result<u32, String> {
        let (n,): (i64,) = sqlx::query_as(SQL_COUNT_CONTENT)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| e.to_string())?;
        Ok(u32::try_from(n).unwrap_or(u32::MAX))
    }

    async fn list_by_ids(&self, ids: &[i32]) -> Result<Vec<ContentDto>, String> {
        if ids.is_empty() {
            return Ok(Vec::new());
        }
        let rows: Vec<ContentRow> = sqlx::query_as(SQL_LIST_CONTENT_BY_IDS)
            .bind(ids)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| e.to_string())?;
        Ok(rows.into_iter().map(ContentDto::from).collect())
    }
}
