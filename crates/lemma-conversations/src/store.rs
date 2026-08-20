use lemma_db::entity::{Conversation, Message};
use uuid::Uuid;

pub async fn insert<'e, E>(executor: E, user_id: Uuid) -> sqlx::Result<Conversation>
where
    E: sqlx::Executor<'e, Database = sqlx::Postgres>,
{
    sqlx::query_as::<_, Conversation>("INSERT INTO conversations (user_id) VALUES ($1) RETURNING *")
        .bind(user_id)
        .fetch_one(executor)
        .await
}

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

// 所有 UPDATE 显式取新 sync_seq：列默认值只作用于 INSERT
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

// 单向迁移：仅 active 可归档；message_count 就地统计
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

// 仅归档可彻底删除；messages 级联
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

// keyset 分页：(created_at, id) 递减取最新一页；before 也须属于同一会话
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
        sqlx::query_as::<_, Message>(
            r#"
            SELECT m.* FROM messages m
            JOIN messages b ON b.id = $2 AND b.conversation_id = $1
            WHERE m.conversation_id = $1
              AND (m.created_at, m.id) < (b.created_at, b.id)
            ORDER BY m.created_at DESC, m.id DESC
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
            "SELECT * FROM messages WHERE conversation_id = $1 ORDER BY created_at DESC, id DESC LIMIT $2",
        )
        .bind(conversation_id)
        .bind(limit + 1)
        .fetch_all(executor)
        .await?
    };
    // limit+1 探测下一页
    let has_more = rows.len() as i64 > limit;
    let messages = if has_more {
        rows[..rows.len() - 1].to_vec()
    } else {
        rows
    };
    Ok((messages, has_more))
}
