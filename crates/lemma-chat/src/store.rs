use lemma_db::entity::{Message, TokenUsage};
use sqlx::types::Json;
use uuid::Uuid;

// 发送事务先锁会话行：串行化同会话的 seq 取号，并发双发也得到确定顺序
pub async fn lock_conversation<'e, E>(executor: E, conversation_id: Uuid) -> sqlx::Result<()>
where
    E: sqlx::Executor<'e, Database = sqlx::Postgres>,
{
    sqlx::query("SELECT id FROM conversations WHERE id = $1 FOR UPDATE")
        .bind(conversation_id)
        .execute(executor)
        .await?;
    Ok(())
}

pub async fn insert_user_message<'e, E>(
    executor: E,
    conversation_id: Uuid,
    content: &str,
) -> sqlx::Result<Message>
where
    E: sqlx::Executor<'e, Database = sqlx::Postgres>,
{
    sqlx::query_as::<_, Message>(
        r#"
        INSERT INTO messages (conversation_id, role, content, seq)
        VALUES ($1, 'user', $2, (SELECT COALESCE(MAX(seq), 0) + 1 FROM messages WHERE conversation_id = $1))
        RETURNING *
        "#,
    )
    .bind(conversation_id)
    .bind(content)
    .fetch_one(executor)
    .await
}

// client_msg_id 只挂 assistant 消息：幂等重放时客户端要的是它
pub async fn insert_assistant_placeholder<'e, E>(
    executor: E,
    conversation_id: Uuid,
    provider_id: Uuid,
    model: &str,
    client_msg_id: Option<&str>,
) -> sqlx::Result<Message>
where
    E: sqlx::Executor<'e, Database = sqlx::Postgres>,
{
    sqlx::query_as::<_, Message>(
        r#"
        INSERT INTO messages (conversation_id, role, content, provider_id, model, status, client_msg_id, seq)
        VALUES ($1, 'assistant', '', $2, $3, 'streaming', $4, (SELECT COALESCE(MAX(seq), 0) + 1 FROM messages WHERE conversation_id = $1))
        RETURNING *
        "#,
    )
    .bind(conversation_id)
    .bind(provider_id)
    .bind(model)
    .bind(client_msg_id)
    .fetch_one(executor)
    .await
}

pub async fn find_assistant_by_client_msg_id<'e, E>(
    executor: E,
    conversation_id: Uuid,
    client_msg_id: &str,
) -> sqlx::Result<Option<Message>>
where
    E: sqlx::Executor<'e, Database = sqlx::Postgres>,
{
    sqlx::query_as::<_, Message>(
        "SELECT * FROM messages WHERE conversation_id = $1 AND client_msg_id = $2",
    )
    .bind(conversation_id)
    .bind(client_msg_id)
    .fetch_optional(executor)
    .await
}

// 归属校验内置：JOIN conversations 限定 user
pub async fn find_by_id_and_user<'e, E>(
    executor: E,
    id: Uuid,
    user_id: Uuid,
) -> sqlx::Result<Option<Message>>
where
    E: sqlx::Executor<'e, Database = sqlx::Postgres>,
{
    sqlx::query_as::<_, Message>(
        r#"
        SELECT m.* FROM messages m
        JOIN conversations c ON c.id = m.conversation_id AND c.user_id = $2
        WHERE m.id = $1
        "#,
    )
    .bind(id)
    .bind(user_id)
    .fetch_optional(executor)
    .await
}

// 模型上下文：只看完整轮次；新建的 user 消息是最后一条，streaming 占位天然排除
pub async fn list_context<'e, E>(executor: E, conversation_id: Uuid) -> sqlx::Result<Vec<Message>>
where
    E: sqlx::Executor<'e, Database = sqlx::Postgres>,
{
    sqlx::query_as::<_, Message>(
        "SELECT * FROM messages WHERE conversation_id = $1 AND status = 'done' ORDER BY seq, id",
    )
    .bind(conversation_id)
    .fetch_all(executor)
    .await
}

// 流式期间的节流落库，节奏由 service 控制
pub async fn flush_content<'e, E>(
    executor: E,
    id: Uuid,
    content: &str,
) -> sqlx::Result<Option<Message>>
where
    E: sqlx::Executor<'e, Database = sqlx::Postgres>,
{
    sqlx::query_as::<_, Message>(
        r#"
        UPDATE messages
        SET content = $2, sync_seq = nextval('sync_seq'), updated_at = now()
        WHERE id = $1
        RETURNING *
        "#,
    )
    .bind(id)
    .bind(content)
    .fetch_optional(executor)
    .await
}

pub async fn finalize<'e, E>(
    executor: E,
    id: Uuid,
    content: &str,
    usage: Option<TokenUsage>,
) -> sqlx::Result<Option<Message>>
where
    E: sqlx::Executor<'e, Database = sqlx::Postgres>,
{
    sqlx::query_as::<_, Message>(
        r#"
        UPDATE messages
        SET status = 'done', content = $2, token_usage = $3,
            sync_seq = nextval('sync_seq'), updated_at = now()
        WHERE id = $1
        RETURNING *
        "#,
    )
    .bind(id)
    .bind(content)
    .bind(usage.map(Json))
    .fetch_optional(executor)
    .await
}

// aborted / error 都保留已生成的部分内容
pub async fn mark_aborted<'e, E>(
    executor: E,
    id: Uuid,
    content: &str,
) -> sqlx::Result<Option<Message>>
where
    E: sqlx::Executor<'e, Database = sqlx::Postgres>,
{
    sqlx::query_as::<_, Message>(
        r#"
        UPDATE messages
        SET status = 'aborted', content = $2, sync_seq = nextval('sync_seq'), updated_at = now()
        WHERE id = $1
        RETURNING *
        "#,
    )
    .bind(id)
    .bind(content)
    .fetch_optional(executor)
    .await
}

pub async fn mark_error<'e, E>(
    executor: E,
    id: Uuid,
    content: &str,
) -> sqlx::Result<Option<Message>>
where
    E: sqlx::Executor<'e, Database = sqlx::Postgres>,
{
    sqlx::query_as::<_, Message>(
        r#"
        UPDATE messages
        SET status = 'error', content = $2, sync_seq = nextval('sync_seq'), updated_at = now()
        WHERE id = $1
        RETURNING *
        "#,
    )
    .bind(id)
    .bind(content)
    .fetch_optional(executor)
    .await
}
