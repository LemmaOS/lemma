use lemma_db::entity::{Conversation, Message};
use uuid::Uuid;

// 多拉一条供调用方探测截断
pub async fn pull_conversations<'e, E>(
    executor: E,
    user_id: Uuid,
    after: i64,
    limit: i64,
) -> sqlx::Result<Vec<Conversation>>
where
    E: sqlx::Executor<'e, Database = sqlx::Postgres>,
{
    sqlx::query_as::<_, Conversation>(
        "SELECT * FROM conversations WHERE user_id = $1 AND sync_seq > $2 ORDER BY sync_seq LIMIT $3",
    )
    .bind(user_id)
    .bind(after)
    .bind(limit)
    .fetch_all(executor)
    .await
}

// 归属校验内置：JOIN conversations 限定 user
pub async fn pull_messages<'e, E>(
    executor: E,
    user_id: Uuid,
    after: i64,
    limit: i64,
) -> sqlx::Result<Vec<Message>>
where
    E: sqlx::Executor<'e, Database = sqlx::Postgres>,
{
    sqlx::query_as::<_, Message>(
        r#"
        SELECT m.* FROM messages m
        JOIN conversations c ON c.id = m.conversation_id AND c.user_id = $1
        WHERE m.sync_seq > $2
        ORDER BY m.sync_seq
        LIMIT $3
        "#,
    )
    .bind(user_id)
    .bind(after)
    .bind(limit)
    .fetch_all(executor)
    .await
}

// 序列头部 = 全局最新变更序号。hint 语义允许跨用户（客户端自行比对游标，空了无害）
pub async fn head_sync_seq<'e, E>(executor: E) -> sqlx::Result<i64>
where
    E: sqlx::Executor<'e, Database = sqlx::Postgres>,
{
    sqlx::query_scalar("SELECT last_value FROM sync_seq")
        .fetch_one(executor)
        .await
}
