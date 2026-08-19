#![allow(clippy::unwrap_used)]

use lemma_auth::users;
use sqlx::PgPool;

#[sqlx::test(migrations = "../lemma-db/migrations")]
async fn first_user_is_owner(pool: PgPool) {
    let u = users::insert(&pool, "alice", "alice@example.com", "hash")
        .await
        .unwrap();
    assert_eq!(u.role, "owner");
}

#[sqlx::test(migrations = "../lemma-db/migrations")]
async fn second_user_is_normal(pool: PgPool) {
    users::insert(&pool, "alice", "alice@example.com", "hash")
        .await
        .unwrap();
    let u = users::insert(&pool, "bob", "bob@example.com", "hash")
        .await
        .unwrap();
    assert_eq!(u.role, "normal");
}

#[sqlx::test(migrations = "../lemma-db/migrations")]
async fn find_by_login_matches_username_or_email(pool: PgPool) {
    users::insert(&pool, "alice", "alice@example.com", "hash")
        .await
        .unwrap();
    assert!(users::find_by_login(&pool, "alice").await.unwrap().is_some());
    assert!(
        users::find_by_login(&pool, "alice@example.com")
            .await
            .unwrap()
            .is_some()
    );
    assert!(users::find_by_login(&pool, "carol").await.unwrap().is_none());
}

#[sqlx::test(migrations = "../lemma-db/migrations")]
async fn find_by_id_miss_returns_none(pool: PgPool) {
    let id = uuid::Uuid::new_v4();
    assert!(users::find_by_id(&pool, id).await.unwrap().is_none());
}
