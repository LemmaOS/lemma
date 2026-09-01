//! Queries for chat-time reads and writes on the messages table.

use lemma_db::entity::{Message, TokenUsage};
use sqlx::types::Json;
use uuid::Uuid;

/// Takes the conversation row lock, serializing concurrent sends so the
/// seq assignment below stays gapless.
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

/// Appends a user message with the next seq in the conversation. Must be
/// called under [`lock_conversation`].
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

/// Appends the streaming placeholder for the assistant reply. Must be
/// called under [`lock_conversation`].
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

/// Finds the assistant message created for an idempotency key.
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

/// Finds a message in a conversation owned by the given user.
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

/// Lists the messages sent as model context. Only completed messages
/// count; streaming, aborted, and failed ones are excluded.
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

/// Persists a mid-stream content snapshot.
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

/// Finalizes a message as done with its content and token usage.
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

/// Finalizes a message as aborted, keeping the content generated so far.
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

/// Finalizes a message as failed, keeping the content generated so far.
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
