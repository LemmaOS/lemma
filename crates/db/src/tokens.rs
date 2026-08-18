use crate::entity::RefreshToken;
use chrono::{DateTime, Utc};
use uuid::Uuid;

pub async fn insert<'e, E>(
    executor: E,
    id: Uuid,
    user_id: Uuid,
    token_hash: &str,
    label: Option<&str>,
    expires_at: DateTime<Utc>,
) -> sqlx::Result<RefreshToken>
where
    E: sqlx::Executor<'e, Database = sqlx::Postgres>,
{
    sqlx::query_as::<_, RefreshToken>(
        r#"
        INSERT INTO refresh_tokens (id, user_id, token_hash, label, expires_at)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING *
        "#,
    )
    .bind(id)
    .bind(user_id)
    .bind(token_hash)
    .bind(label)
    .bind(expires_at)
    .fetch_one(executor)
    .await
}

// 不做有效性过滤，状态判定留给 auth 层（区分过期/吊销/重放）
pub async fn find_by_hash<'e, E>(
    executor: E,
    token_hash: &str,
) -> sqlx::Result<Option<RefreshToken>>
where
    E: sqlx::Executor<'e, Database = sqlx::Postgres>,
{
    sqlx::query_as::<_, RefreshToken>("SELECT * FROM refresh_tokens WHERE token_hash = $1")
        .bind(token_hash)
        .fetch_optional(executor)
        .await
}

pub async fn mark_replaced<'e, E>(executor: E, id: Uuid, replaced_by: Uuid) -> sqlx::Result<u64>
where
    E: sqlx::Executor<'e, Database = sqlx::Postgres>,
{
    sqlx::query("UPDATE refresh_tokens SET replaced_by = $2 WHERE id = $1")
        .bind(id)
        .bind(replaced_by)
        .execute(executor)
        .await
        .map(|r| r.rows_affected())
}

pub async fn revoke<'e, E>(executor: E, id: Uuid) -> sqlx::Result<u64>
where
    E: sqlx::Executor<'e, Database = sqlx::Postgres>,
{
    sqlx::query("UPDATE refresh_tokens SET revoked_at = now() WHERE id = $1 AND revoked_at IS NULL")
        .bind(id)
        .execute(executor)
        .await
        .map(|r| r.rows_affected())
}

// 重放检测：吊销整条轮换链
pub async fn revoke_chain<'e, E>(executor: E, id: Uuid) -> sqlx::Result<u64>
where
    E: sqlx::Executor<'e, Database = sqlx::Postgres>,
{
    sqlx::query(
        r#"
        WITH RECURSIVE chain AS (
            SELECT id FROM refresh_tokens WHERE id = $1
            UNION ALL
            SELECT r.id FROM refresh_tokens r JOIN chain c ON r.replaced_by = c.id
        )
        UPDATE refresh_tokens SET revoked_at = now()
        WHERE id IN (SELECT id FROM chain) AND revoked_at IS NULL
        "#,
    )
    .bind(id)
    .execute(executor)
    .await
    .map(|r| r.rows_affected())
}
