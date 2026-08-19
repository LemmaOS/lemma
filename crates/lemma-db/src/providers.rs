use crate::entity::Provider;
use sqlx::QueryBuilder;
use sqlx::types::Json;
use uuid::Uuid;

pub struct NewProvider<'a> {
    pub id: Uuid,
    pub user_id: Uuid,
    pub kind: &'a str,
    pub name: &'a str,
    pub base_url: &'a str,
    pub api_key: &'a str, // 已加密
    pub api_path: &'a str,
    pub models_path: &'a str,
    pub models: &'a [String],
}

pub struct ProviderPatch {
    pub name: Option<String>,
    pub base_url: Option<String>,
    pub api_key: Option<String>, // 已加密
    pub api_path: Option<String>,
    pub models_path: Option<String>,
    pub enabled: Option<bool>,
    pub models: Option<Vec<String>>,
}

pub async fn insert<'e, E>(executor: E, p: &NewProvider<'_>) -> sqlx::Result<Provider>
where
    E: sqlx::Executor<'e, Database = sqlx::Postgres>,
{
    sqlx::query_as::<_, Provider>(
        r#"
        INSERT INTO providers (id, user_id, kind, name, base_url, api_key, api_path, models_path, models)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
        RETURNING *
        "#,
    )
    .bind(p.id)
    .bind(p.user_id)
    .bind(p.kind)
    .bind(p.name)
    .bind(p.base_url)
    .bind(p.api_key)
    .bind(p.api_path)
    .bind(p.models_path)
    .bind(Json(p.models.to_vec()))
    .fetch_one(executor)
    .await
}

pub async fn list_by_user<'e, E>(executor: E, user_id: Uuid) -> sqlx::Result<Vec<Provider>>
where
    E: sqlx::Executor<'e, Database = sqlx::Postgres>,
{
    sqlx::query_as::<_, Provider>("SELECT * FROM providers WHERE user_id = $1 ORDER BY created_at")
        .bind(user_id)
        .fetch_all(executor)
        .await
}

// 归属校验内置：id 存在但不属于该用户时同样返回 None
pub async fn find_by_id_and_user<'e, E>(
    executor: E,
    id: Uuid,
    user_id: Uuid,
) -> sqlx::Result<Option<Provider>>
where
    E: sqlx::Executor<'e, Database = sqlx::Postgres>,
{
    sqlx::query_as::<_, Provider>("SELECT * FROM providers WHERE id = $1 AND user_id = $2")
        .bind(id)
        .bind(user_id)
        .fetch_optional(executor)
        .await
}

// 动态 SET；updated_at 恒更新，空 patch 也是合法 SQL
pub async fn update<'e, E>(
    executor: E,
    id: Uuid,
    user_id: Uuid,
    patch: ProviderPatch,
) -> sqlx::Result<Option<Provider>>
where
    E: sqlx::Executor<'e, Database = sqlx::Postgres>,
{
    let mut qb: QueryBuilder<sqlx::Postgres> = QueryBuilder::new("UPDATE providers SET ");
    {
        let mut sep = qb.separated(", ");
        sep.push("updated_at = now()");
        if let Some(v) = patch.name {
            sep.push(" name = ").push_bind_unseparated(v);
        }
        if let Some(v) = patch.base_url {
            sep.push(" base_url = ").push_bind_unseparated(v);
        }
        if let Some(v) = patch.api_key {
            sep.push(" api_key = ").push_bind_unseparated(v);
        }
        if let Some(v) = patch.api_path {
            sep.push(" api_path = ").push_bind_unseparated(v);
        }
        if let Some(v) = patch.models_path {
            sep.push(" models_path = ").push_bind_unseparated(v);
        }
        if let Some(v) = patch.enabled {
            sep.push(" enabled = ").push_bind_unseparated(v);
        }
        if let Some(v) = patch.models {
            sep.push(" models = ").push_bind_unseparated(Json(v));
        }
    }
    qb.push(" WHERE id = ").push_bind(id);
    qb.push(" AND user_id = ").push_bind(user_id);
    qb.push(" RETURNING *");
    qb.build_query_as::<Provider>()
        .fetch_optional(executor)
        .await
}

pub async fn delete<'e, E>(executor: E, id: Uuid, user_id: Uuid) -> sqlx::Result<bool>
where
    E: sqlx::Executor<'e, Database = sqlx::Postgres>,
{
    sqlx::query("DELETE FROM providers WHERE id = $1 AND user_id = $2")
        .bind(id)
        .bind(user_id)
        .execute(executor)
        .await
        .map(|r| r.rows_affected() > 0)
}
