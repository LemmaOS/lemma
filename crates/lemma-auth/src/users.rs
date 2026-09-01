//! Queries for the users table.

use lemma_db::entity::User;

/// Inserts a user and returns it. The very first user becomes the owner;
/// everyone after that is normal.
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

/// Finds a user by username or email; `login` matches either column.
pub async fn find_by_login<'e, E>(executor: E, login: &str) -> sqlx::Result<Option<User>>
where
    E: sqlx::Executor<'e, Database = sqlx::Postgres>,
{
    sqlx::query_as::<_, User>("SELECT * FROM users WHERE username = $1 OR email = $1")
        .bind(login)
        .fetch_optional(executor)
        .await
}

/// Finds a user by id.
pub async fn find_by_id<'e, E>(executor: E, id: uuid::Uuid) -> sqlx::Result<Option<User>>
where
    E: sqlx::Executor<'e, Database = sqlx::Postgres>,
{
    sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1")
        .bind(id)
        .fetch_optional(executor)
        .await
}
