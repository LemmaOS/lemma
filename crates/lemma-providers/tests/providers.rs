#![allow(clippy::unwrap_used)]

use lemma_auth::users;
use lemma_providers::providers::{self, NewProvider, ProviderPatch};
use sqlx::PgPool;
use uuid::Uuid;

async fn new_user(pool: &PgPool, name: &str) -> Uuid {
    users::insert(pool, name, &format!("{name}@example.com"), "hash")
        .await
        .unwrap()
        .id
}

fn new_provider(uid: Uuid, name: &str) -> NewProvider<'_> {
    NewProvider {
        id: Uuid::new_v4(),
        user_id: uid,
        kind: "openai",
        name,
        base_url: "https://api.example.com/v1",
        api_key: "sealed",
        api_path: "",
        models_path: "",
        models: &[],
    }
}

#[sqlx::test(migrations = "../lemma-db/migrations")]
async fn list_scoped_to_owner(pool: PgPool) {
    let u1 = new_user(&pool, "alice").await;
    let u2 = new_user(&pool, "erin").await;
    providers::insert(&pool, &new_provider(u1, "p1"))
        .await
        .unwrap();
    providers::insert(&pool, &new_provider(u1, "p2"))
        .await
        .unwrap();
    providers::insert(&pool, &new_provider(u2, "p3"))
        .await
        .unwrap();
    assert_eq!(providers::list_by_user(&pool, u1).await.unwrap().len(), 2);
    assert_eq!(providers::list_by_user(&pool, u2).await.unwrap().len(), 1);
}

#[sqlx::test(migrations = "../lemma-db/migrations")]
async fn find_by_id_and_user_isolation(pool: PgPool) {
    let u1 = new_user(&pool, "alice").await;
    let u2 = new_user(&pool, "erin").await;
    let p = providers::insert(&pool, &new_provider(u1, "p"))
        .await
        .unwrap();
    assert!(
        providers::find_by_id_and_user(&pool, p.id, u1)
            .await
            .unwrap()
            .is_some()
    );
    assert!(
        providers::find_by_id_and_user(&pool, p.id, u2)
            .await
            .unwrap()
            .is_none()
    );
}

// 回归：动态 update 只动指定字段（曾因 push_bind 分隔符产生非法 SQL）
#[sqlx::test(migrations = "../lemma-db/migrations")]
async fn update_partial_keeps_other_fields(pool: PgPool) {
    let uid = new_user(&pool, "alice").await;
    let models = vec!["m1".to_string(), "m2".to_string()];
    let p = providers::insert(
        &pool,
        &NewProvider {
            models: &models,
            ..new_provider(uid, "p")
        },
    )
    .await
    .unwrap();
    let updated = providers::update(
        &pool,
        p.id,
        uid,
        ProviderPatch {
            name: Some("renamed".into()),
            enabled: Some(false),
            ..Default::default()
        },
    )
    .await
    .unwrap()
    .unwrap();
    assert_eq!(updated.name, "renamed");
    assert!(!updated.enabled);
    // 未涉及字段保持原值
    assert_eq!(updated.base_url, p.base_url);
    assert_eq!(updated.models.0, p.models.0);
    assert_eq!(updated.api_key, p.api_key);
}

#[sqlx::test(migrations = "../lemma-db/migrations")]
async fn update_wrong_user_returns_none(pool: PgPool) {
    let u1 = new_user(&pool, "alice").await;
    let u2 = new_user(&pool, "erin").await;
    let p = providers::insert(&pool, &new_provider(u1, "p"))
        .await
        .unwrap();
    let patched = providers::update(
        &pool,
        p.id,
        u2,
        ProviderPatch {
            name: Some("hack".into()),
            ..Default::default()
        },
    )
    .await
    .unwrap();
    assert!(patched.is_none());
}

#[sqlx::test(migrations = "../lemma-db/migrations")]
async fn delete_once_then_misses(pool: PgPool) {
    let uid = new_user(&pool, "alice").await;
    let p = providers::insert(&pool, &new_provider(uid, "p"))
        .await
        .unwrap();
    assert!(providers::delete(&pool, p.id, uid).await.unwrap());
    assert!(!providers::delete(&pool, p.id, uid).await.unwrap());
}
