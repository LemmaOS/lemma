//! Queries for the conversations and messages tables.

use lemma_db::entity::{Conversation, Message};
use uuid::Uuid;

/// Creates an empty conversation and returns it.
pub async fn insert<'e, E>(executor: E, user_id: Uuid) -> sqlx::Result<Conversation>
where
    E: sqlx::Executor<'e, Database = sqlx::Postgres>,
{
    sqlx::query_as::<_, Conversation>("INSERT INTO conversations (user_id) VALUES ($1) RETURNING *")
        .bind(user_id)
        .fetch_one(executor)
        .await
}

/// Lists a user's active conversations, most recently updated first.
pub async fn list_active_by_user<'e, E>(
    executor: E,
    user_id: Uuid,
) -> sqlx::Result<Vec<Conversation>>
where
    E: sqlx::Executor<'e, Database = sqlx::Postgres>,
{
    sqlx::query_as::<_, Conversation>(
        "SELECT * FROM conversations WHERE user_id = $1 AND status = 'active' ORDER BY updated_at DESC",
    )
    .bind(user_id)
    .fetch_all(executor)
    .await
}

/// Lists a user's archived conversations, most recently archived first.
pub async fn list_archived_by_user<'e, E>(
    executor: E,
    user_id: Uuid,
) -> sqlx::Result<Vec<Conversation>>
where
    E: sqlx::Executor<'e, Database = sqlx::Postgres>,
{
    sqlx::query_as::<_, Conversation>(
        "SELECT * FROM conversations WHERE user_id = $1 AND status = 'archived' ORDER BY archived_at DESC",
    )
    .bind(user_id)
    .fetch_all(executor)
    .await
}

/// Finds a conversation owned by the given user, any status.
pub async fn find_by_id_and_user<'e, E>(
    executor: E,
    id: Uuid,
    user_id: Uuid,
) -> sqlx::Result<Option<Conversation>>
where
    E: sqlx::Executor<'e, Database = sqlx::Postgres>,
{
    sqlx::query_as::<_, Conversation>("SELECT * FROM conversations WHERE id = $1 AND user_id = $2")
        .bind(id)
        .bind(user_id)
        .fetch_optional(executor)
        .await
}

/// Renames a conversation owned by the given user.
pub async fn rename<'e, E>(
    executor: E,
    id: Uuid,
    user_id: Uuid,
    title: &str,
) -> sqlx::Result<Option<Conversation>>
where
    E: sqlx::Executor<'e, Database = sqlx::Postgres>,
{
    sqlx::query_as::<_, Conversation>(
        r#"
        UPDATE conversations
        SET title = $3, sync_seq = nextval('sync_seq'), updated_at = now()
        WHERE id = $1 AND user_id = $2
        RETURNING *
        "#,
    )
    .bind(id)
    .bind(user_id)
    .bind(title)
    .fetch_optional(executor)
    .await
}

/// Archives an active conversation without touching its messages,
/// snapshotting the message count. Returns `None` unless the conversation
/// is currently active.
pub async fn archive<'e, E>(
    executor: E,
    id: Uuid,
    user_id: Uuid,
) -> sqlx::Result<Option<Conversation>>
where
    E: sqlx::Executor<'e, Database = sqlx::Postgres>,
{
    sqlx::query_as::<_, Conversation>(
        r#"
        UPDATE conversations
        SET status = 'archived',
            archived_at = now(),
            message_count = (SELECT count(*) FROM messages WHERE conversation_id = $1),
            sync_seq = nextval('sync_seq'),
            updated_at = now()
        WHERE id = $1 AND user_id = $2 AND status = 'active'
        RETURNING *
        "#,
    )
    .bind(id)
    .bind(user_id)
    .fetch_optional(executor)
    .await
}

/// Restores an archived conversation to active, clearing the archive
/// metadata. Returns `None` unless the conversation is currently
/// archived.
pub async fn restore<'e, E>(
    executor: E,
    id: Uuid,
    user_id: Uuid,
) -> sqlx::Result<Option<Conversation>>
where
    E: sqlx::Executor<'e, Database = sqlx::Postgres>,
{
    sqlx::query_as::<_, Conversation>(
        r#"
        UPDATE conversations
        SET status = 'active', archived_at = NULL, message_count = NULL,
            sync_seq = nextval('sync_seq'), updated_at = now()
        WHERE id = $1 AND user_id = $2 AND status = 'archived'
        RETURNING *
        "#,
    )
    .bind(id)
    .bind(user_id)
    .fetch_optional(executor)
    .await
}

/// Deletes an archived conversation. Active conversations are not
/// deletable through this path.
pub async fn delete_archived<'e, E>(executor: E, id: Uuid, user_id: Uuid) -> sqlx::Result<bool>
where
    E: sqlx::Executor<'e, Database = sqlx::Postgres>,
{
    sqlx::query("DELETE FROM conversations WHERE id = $1 AND user_id = $2 AND status = 'archived'")
        .bind(id)
        .bind(user_id)
        .execute(executor)
        .await
        .map(|r| r.rows_affected() > 0)
}

/// Keyset-paginates a conversation's messages newest-first. `before_id`
/// selects messages older than that message; one extra row is fetched to
/// compute `has_more`.
pub async fn list_messages<'e, E>(
    executor: E,
    conversation_id: Uuid,
    before_id: Option<Uuid>,
    limit: i64,
) -> sqlx::Result<(Vec<Message>, bool)>
where
    E: sqlx::Executor<'e, Database = sqlx::Postgres>,
{
    let rows: Vec<Message> = if let Some(before) = before_id {
        // The self-join resolves before_id to its seq in the same
        // conversation, so the cursor cannot page across conversations.
        sqlx::query_as::<_, Message>(
            r#"
            SELECT m.* FROM messages m
            JOIN messages b ON b.id = $2 AND b.conversation_id = $1
            WHERE m.conversation_id = $1
              AND m.seq < b.seq
            ORDER BY m.seq DESC
            LIMIT $3
            "#,
        )
        .bind(conversation_id)
        .bind(before)
        .bind(limit + 1)
        .fetch_all(executor)
        .await?
    } else {
        sqlx::query_as::<_, Message>(
            "SELECT * FROM messages WHERE conversation_id = $1 ORDER BY seq DESC LIMIT $2",
        )
        .bind(conversation_id)
        .bind(limit + 1)
        .fetch_all(executor)
        .await?
    };
    let has_more = rows.len() as i64 > limit;
    let messages = if has_more {
        rows[..rows.len() - 1].to_vec()
    } else {
        rows
    };
    Ok((messages, has_more))
}

/// Locks an active conversation row for update inside a transaction.
pub async fn lock_active(
    conn: &mut sqlx::PgConnection,
    id: Uuid,
    user_id: Uuid,
) -> sqlx::Result<Option<Conversation>> {
    sqlx::query_as(
        "SELECT * FROM conversations WHERE id = $1 AND user_id = $2 AND status = 'active' FOR UPDATE",
    )
    .bind(id)
    .bind(user_id)
    .fetch_optional(conn)
    .await
}

/// Locks an archived conversation row for update inside a transaction.
pub async fn lock_archived(
    conn: &mut sqlx::PgConnection,
    id: Uuid,
    user_id: Uuid,
) -> sqlx::Result<Option<Conversation>> {
    sqlx::query_as(
        "SELECT * FROM conversations WHERE id = $1 AND user_id = $2 AND status = 'archived' FOR UPDATE",
    )
    .bind(id)
    .bind(user_id)
    .fetch_optional(conn)
    .await
}

/// Lists all of a conversation's messages in seq order, for archiving.
pub async fn list_all_messages(
    conn: &mut sqlx::PgConnection,
    conversation_id: Uuid,
) -> sqlx::Result<Vec<Message>> {
    sqlx::query_as("SELECT * FROM messages WHERE conversation_id = $1 ORDER BY seq")
        .bind(conversation_id)
        .fetch_all(conn)
        .await
}

/// Marks a locked conversation archived with its S3 object key,
/// snapshotting the message count.
pub async fn mark_archived_with_key(
    conn: &mut sqlx::PgConnection,
    id: Uuid,
    key: &str,
) -> sqlx::Result<Conversation> {
    sqlx::query_as(
        r#"
        UPDATE conversations
        SET status = 'archived', archived_at = now(),
            message_count = (SELECT count(*) FROM messages WHERE conversation_id = $1),
            archive_key = $2, sync_seq = nextval('sync_seq'), updated_at = now()
        WHERE id = $1
        RETURNING *
        "#,
    )
    .bind(id)
    .bind(key)
    .fetch_one(conn)
    .await
}

/// Deletes all of a conversation's messages, after they are safely in
/// the archive.
pub async fn delete_all_messages(
    conn: &mut sqlx::PgConnection,
    conversation_id: Uuid,
) -> sqlx::Result<u64> {
    sqlx::query("DELETE FROM messages WHERE conversation_id = $1")
        .bind(conversation_id)
        .execute(conn)
        .await
        .map(|r| r.rows_affected())
}

/// Reinserts messages recovered from an archive, preserving their ids and
/// seq. `sync_seq` is intentionally not bound: the INSERT default draws a
/// fresh sequence value.
pub async fn insert_restored(
    conn: &mut sqlx::PgConnection,
    messages: &[Message],
) -> sqlx::Result<()> {
    for m in messages {
        sqlx::query(
            r#"
            INSERT INTO messages (id, conversation_id, role, content, provider_id, model,
                                  client_msg_id, status, token_usage, seq, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
            "#,
        )
        .bind(m.id)
        .bind(m.conversation_id)
        .bind(&m.role)
        .bind(&m.content)
        .bind(m.provider_id)
        .bind(&m.model)
        .bind(&m.client_msg_id)
        .bind(&m.status)
        .bind(&m.token_usage)
        .bind(m.seq)
        .bind(m.created_at)
        .bind(m.updated_at)
        .execute(&mut *conn)
        .await?;
    }
    Ok(())
}

/// Returns the archive key of an archived conversation. The outer `None`
/// means the conversation is missing or not archived; the inner `None`
/// means it was archived in place without an S3 object.
pub async fn find_archive_key<'e, E>(
    executor: E,
    id: Uuid,
    user_id: Uuid,
) -> sqlx::Result<Option<Option<String>>>
where
    E: sqlx::Executor<'e, Database = sqlx::Postgres>,
{
    sqlx::query_scalar::<_, Option<String>>(
        "SELECT archive_key FROM conversations WHERE id = $1 AND user_id = $2 AND status = 'archived'",
    )
    .bind(id)
    .bind(user_id)
    .fetch_optional(executor)
    .await
}
