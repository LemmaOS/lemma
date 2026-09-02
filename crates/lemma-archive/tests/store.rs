#![allow(clippy::unwrap_used, missing_docs)]

use lemma_archive::store::{self, UpsertS3Config};
use lemma_auth::users;
use sqlx::PgPool;
use uuid::Uuid;

async fn new_user(pool: &PgPool) -> Uuid {
    let name = format!("u-{}", Uuid::new_v4());
    users::insert(pool, &name, &format!("{name}@example.com"), "hash")
        .await
        .unwrap()
        .id
}

fn upsert_cfg<'a>(
    user_id: Uuid,
    endpoint: &'a str,
    bucket: &'a str,
    migration_from: Option<serde_json::Value>,
) -> UpsertS3Config<'a> {
    UpsertS3Config {
        user_id,
        endpoint,
        region: "us-east-1",
        bucket,
        access_key: "sealed-a",
        secret_key: "sealed-s",
        migration_from,
    }
}

#[sqlx::test(migrations = "../lemma-db/migrations")]
async fn upsert_replaces_same_user_row(pool: PgPool) {
    let user = new_user(&pool).await;
    let first = store::upsert(&pool, &upsert_cfg(user, "http://old:9000", "b1", None))
        .await
        .unwrap();
    let second = store::upsert(&pool, &upsert_cfg(user, "http://new:9000", "b2", None))
        .await
        .unwrap();

    assert_eq!(first.id, second.id);
    assert_eq!(second.endpoint, "http://new:9000");
    assert_eq!(second.bucket, "b2");
}

#[sqlx::test(migrations = "../lemma-db/migrations")]
async fn clear_migration_wipes_snapshot(pool: PgPool) {
    let user = new_user(&pool).await;
    let snapshot = serde_json::json!({
        "endpoint": "http://old:9000",
        "region": "us-east-1",
        "bucket": "b1",
        "access_key": "sealed-a",
        "secret_key": "sealed-s",
    });
    store::upsert(
        &pool,
        &upsert_cfg(user, "http://new:9000", "b2", Some(snapshot)),
    )
    .await
    .unwrap();

    assert!(store::clear_migration(&pool, user).await.unwrap());
    let cleared = store::find_by_user(&pool, user).await.unwrap().unwrap();
    assert!(cleared.migration_from.is_none());
    assert!(cleared.migrated_at.is_some());

    assert!(!store::clear_migration(&pool, user).await.unwrap());
}

#[sqlx::test(migrations = "../lemma-db/migrations")]
async fn delete_by_user_removes_config(pool: PgPool) {
    let user = new_user(&pool).await;
    store::upsert(&pool, &upsert_cfg(user, "http://x:9000", "b", None))
        .await
        .unwrap();

    assert!(store::delete_by_user(&pool, user).await.unwrap());
    assert!(store::find_by_user(&pool, user).await.unwrap().is_none());
    assert!(!store::delete_by_user(&pool, user).await.unwrap());
}

#[sqlx::test(migrations = "../lemma-db/migrations")]
async fn archive_keys_scoped_and_filtered(pool: PgPool) {
    let user = new_user(&pool).await;
    let other = new_user(&pool).await;

    for (status, key) in [
        ("active", None),
        ("archived", Some("archives/a.json")),
        ("archived", None),
    ] {
        sqlx::query(
            "INSERT INTO conversations (id, user_id, status, archive_key) VALUES ($1, $2, $3, $4)",
        )
        .bind(Uuid::new_v4())
        .bind(user)
        .bind(status)
        .bind(key)
        .execute(&pool)
        .await
        .unwrap();
    }
    sqlx::query("INSERT INTO conversations (id, user_id, status, archive_key) VALUES ($1, $2, 'archived', $3)")
        .bind(Uuid::new_v4())
        .bind(other)
        .bind("archives/other.json")
        .execute(&pool)
        .await
        .unwrap();

    let keys = store::list_archive_keys(&pool, user).await.unwrap();
    assert_eq!(keys, vec!["archives/a.json".to_string()]);
}
