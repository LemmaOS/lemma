#![allow(clippy::unwrap_used)]

use chrono::{DateTime, Duration, Utc};
use lemma_auth::{tokens, users};
use sqlx::PgPool;
use uuid::Uuid;

// 每个测试独立建库，用户名可任意
async fn new_user(pool: &PgPool) -> Uuid {
    let name = format!("u-{}", Uuid::new_v4());
    users::insert(pool, &name, &format!("{name}@example.com"), "hash")
        .await
        .unwrap()
        .id
}

fn expires() -> DateTime<Utc> {
    Utc::now() + Duration::days(30)
}

#[sqlx::test(migrations = "../lemma-db/migrations")]
async fn insert_and_find_by_hash(pool: PgPool) {
    let uid = new_user(&pool).await;
    let id = Uuid::new_v4();
    tokens::insert(&pool, id, uid, "hash-a", None, expires())
        .await
        .unwrap();
    let row = tokens::find_by_hash(&pool, "hash-a")
        .await
        .unwrap()
        .unwrap();
    assert_eq!(row.id, id);
    assert_eq!(row.user_id, uid);
    assert!(row.revoked_at.is_none());
    assert!(row.replaced_by.is_none());
}

#[sqlx::test(migrations = "../lemma-db/migrations")]
async fn mark_replaced_links_old_to_new(pool: PgPool) {
    let uid = new_user(&pool).await;
    let (a, b) = (Uuid::new_v4(), Uuid::new_v4());
    tokens::insert(&pool, a, uid, "hash-a", None, expires())
        .await
        .unwrap();
    tokens::insert(&pool, b, uid, "hash-b", None, expires())
        .await
        .unwrap();
    tokens::mark_replaced(&pool, a, b).await.unwrap();
    let row = tokens::find_by_hash(&pool, "hash-a")
        .await
        .unwrap()
        .unwrap();
    assert_eq!(row.replaced_by, Some(b));
}

#[sqlx::test(migrations = "../lemma-db/migrations")]
async fn revoke_is_guarded_by_revoked_at(pool: PgPool) {
    let uid = new_user(&pool).await;
    let id = Uuid::new_v4();
    tokens::insert(&pool, id, uid, "hash-a", None, expires())
        .await
        .unwrap();
    assert_eq!(tokens::revoke(&pool, id).await.unwrap(), 1);
    // 已吊销的再吊销是幂等 no-op
    assert_eq!(tokens::revoke(&pool, id).await.unwrap(), 0);
}

// 回归：链式吊销必须覆盖全部后代（曾只吊销链首）
#[sqlx::test(migrations = "../lemma-db/migrations")]
async fn revoke_chain_revokes_all_descendants(pool: PgPool) {
    let uid = new_user(&pool).await;
    let (a, b, c) = (Uuid::new_v4(), Uuid::new_v4(), Uuid::new_v4());
    for (id, hash) in [(a, "hash-a"), (b, "hash-b"), (c, "hash-c")] {
        tokens::insert(&pool, id, uid, hash, None, expires())
            .await
            .unwrap();
    }
    // a→b→c 轮换链
    tokens::mark_replaced(&pool, a, b).await.unwrap();
    tokens::mark_replaced(&pool, b, c).await.unwrap();

    assert_eq!(tokens::revoke_chain(&pool, a).await.unwrap(), 3);
    for hash in ["hash-a", "hash-b", "hash-c"] {
        let row = tokens::find_by_hash(&pool, hash).await.unwrap().unwrap();
        assert!(row.revoked_at.is_some(), "{hash} 应被吊销");
    }
}
