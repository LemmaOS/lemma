use crate::entity::User;

// 首个用户为 owner；并发竞态由 owner 唯一部分索引兜底（23505）
pub async fn insert<'e, E>(
    executor: E,
    username: &str,
    email: &str,
    password_hash: &str,
) -> sqlx::Result<User>
where
    E: sqlx::Executor<'e, Database = sqlx::Postgres>,
{
    sqlx::query_as::<_, User>(
        r#"
        INSERT INTO users (username, email, password_hash, role)
        VALUES ($1, $2, $3,
            CASE WHEN EXISTS (SELECT 1 FROM users WHERE role = 'owner')
                THEN 'normal' ELSE 'owner' END)
        RETURNING *
        "#,
    )
    .bind(username)
    .bind(email)
    .bind(password_hash)
    .fetch_one(executor)
    .await
}

pub async fn find_by_login<'e, E>(executor: E, login: &str) -> sqlx::Result<Option<User>>
where
    E: sqlx::Executor<'e, Database = sqlx::Postgres>,
{
    sqlx::query_as::<_, User>("SELECT * FROM users WHERE username = $1 OR email = $1")
        .bind(login)
        .fetch_optional(executor)
        .await
}

pub async fn find_by_id<'e, E>(executor: E, id: uuid::Uuid) -> sqlx::Result<Option<User>>
where
    E: sqlx::Executor<'e, Database = sqlx::Postgres>,
{
    sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1")
        .bind(id)
        .fetch_optional(executor)
        .await
}
