use serde_json::Value;
use sqlx::PgPool;

pub(super) const ENTITY_CONTENT: &str = "content";
pub(super) const ENTITY_TAG: &str = "tag";
pub(super) const OP_UPSERT: &str = "upsert";
pub(super) const OP_DELETE: &str = "delete";
pub(super) const ERR_REMOTE_ROW: &str = "remote row missing id";

const QUEUE_BATCH: i64 = 50;

const SQL_ENQUEUE: &str =
    "INSERT INTO sync_queue (entity, op, slug, payload) VALUES ($1, $2, $3, $4)";
const SQL_PENDING: &str =
    "SELECT id, op, slug, payload FROM sync_queue WHERE entity = $1 ORDER BY id ASC LIMIT $2";
const SQL_DEQUEUE: &str = "DELETE FROM sync_queue WHERE id = $1";

#[derive(sqlx::FromRow)]
pub(super) struct QueueRow {
    pub id: i32,
    pub op: String,
    pub slug: String,
    pub payload: Value,
}

pub(super) async fn enqueue(pool: &PgPool, entity: &str, op: &str, slug: &str, payload: Value) {
    if let Err(e) = sqlx::query(SQL_ENQUEUE)
        .bind(entity)
        .bind(op)
        .bind(slug)
        .bind(payload)
        .execute(pool)
        .await
    {
        tracing::warn!("{entity} sync enqueue failed: {e}");
    }
}

pub(super) async fn pending(pool: &PgPool, entity: &str) -> Result<Vec<QueueRow>, String> {
    sqlx::query_as(SQL_PENDING)
        .bind(entity)
        .bind(QUEUE_BATCH)
        .fetch_all(pool)
        .await
        .map_err(|e| e.to_string())
}

pub(super) async fn dequeue(pool: &PgPool, id: i32) -> Result<(), String> {
    sqlx::query(SQL_DEQUEUE)
        .bind(id)
        .execute(pool)
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}
