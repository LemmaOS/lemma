use lemma_db::entity::S3Config;
use sqlx::types::Json;
use uuid::Uuid;

pub struct UpsertS3Config<'a> {
    pub user_id: Uuid,
    pub endpoint: &'a str,
    pub region: &'a str,
    pub bucket: &'a str,
    pub access_key: &'a str,
    pub secret_key: &'a str,
    pub migration_from: Option<serde_json::Value>,
}

pub async fn upsert<'e, E>(executor: E, cfg: &UpsertS3Config<'_>) -> sqlx::Result<S3Config>
where
    E: sqlx::Executor<'e, Database = sqlx::Postgres>,
{
    sqlx::query_as::<_, S3Config>(
        r#"
        INSERT INTO s3_configs (user_id, endpoint, region, bucket, access_key, secret_key, migration_from)
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        ON CONFLICT (user_id) DO UPDATE SET
            endpoint = EXCLUDED.endpoint,
            region = EXCLUDED.region,
            bucket = EXCLUDED.bucket,
            access_key = EXCLUDED.access_key,
            secret_key = EXCLUDED.secret_key,
            migration_from = EXCLUDED.migration_from,
            updated_at = now()
        RETURNING *
        "#,
    )
    .bind(cfg.user_id)
    .bind(cfg.endpoint)
    .bind(cfg.region)
    .bind(cfg.bucket)
    .bind(cfg.access_key)
    .bind(cfg.secret_key)
    .bind(cfg.migration_from.as_ref().map(Json))
    .fetch_one(executor)
    .await
}

pub async fn find_by_user<'e, E>(executor: E, user_id: Uuid) -> sqlx::Result<Option<S3Config>>
where
    E: sqlx::Executor<'e, Database = sqlx::Postgres>,
{
    sqlx::query_as::<_, S3Config>("SELECT * FROM s3_configs WHERE user_id = $1")
        .bind(user_id)
        .fetch_optional(executor)
        .await
}

pub async fn delete_by_user<'e, E>(executor: E, user_id: Uuid) -> sqlx::Result<bool>
where
    E: sqlx::Executor<'e, Database = sqlx::Postgres>,
{
    sqlx::query("DELETE FROM s3_configs WHERE user_id = $1")
        .bind(user_id)
        .execute(executor)
        .await
        .map(|r| r.rows_affected() > 0)
}

pub async fn clear_migration<'e, E>(executor: E, user_id: Uuid) -> sqlx::Result<bool>
where
    E: sqlx::Executor<'e, Database = sqlx::Postgres>,
{
    sqlx::query(
        "UPDATE s3_configs
         SET migration_from = NULL, migrated_at = now(), updated_at = now()
         WHERE user_id = $1 AND migration_from IS NOT NULL",
    )
    .bind(user_id)
    .execute(executor)
    .await
    .map(|r| r.rows_affected() > 0)
}

pub async fn list_archive_keys<'e, E>(executor: E, user_id: Uuid) -> sqlx::Result<Vec<String>>
where
    E: sqlx::Executor<'e, Database = sqlx::Postgres>,
{
    sqlx::query_scalar(
        "SELECT archive_key FROM conversations
         WHERE user_id = $1 AND status = 'archived' AND archive_key IS NOT NULL",
    )
    .bind(user_id)
    .fetch_all(executor)
    .await
}
