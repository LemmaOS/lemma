#![allow(clippy::unwrap_used, missing_docs)]

use lemma_auth::users;
use lemma_sync::store;
use sqlx::PgPool;
use uuid::Uuid;

async fn new_user(pool: &PgPool, name: &str) -> Uuid {
    users::insert(pool, name, &format!("{name}@example.com"), "hash")
        .await
        .unwrap()
        .id
}

#[sqlx::test(migrations = "../lemma-db/migrations")]
async fn pull_respects_cursor_and_order(pool: PgPool) {
    let uid = new_user(&pool, "alice").await;
    let c1 = lemma_conversations::store::insert(&pool, uid)
        .await
        .unwrap();
    let c2 = lemma_conversations::store::insert(&pool, uid)
        .await
        .unwrap();
    let c1 = lemma_conversations::store::rename(&pool, c1.id, uid, "新标题")
        .await
        .unwrap()
        .unwrap();

    let all = store::pull_conversations(&pool, uid, 0, 10).await.unwrap();
    assert_eq!(all.len(), 2);
    assert_eq!(all[0].id, c2.id);
    assert_eq!(all[1].id, c1.id);
    assert_eq!(all[1].title, "新标题");
}

#[sqlx::test(migrations = "../lemma-db/migrations")]
async fn pull_messages_scoped_to_owner(pool: PgPool) {
    let u1 = new_user(&pool, "alice").await;
    let u2 = new_user(&pool, "erin").await;
    let c1 = lemma_conversations::store::insert(&pool, u1).await.unwrap();
    let c2 = lemma_conversations::store::insert(&pool, u2).await.unwrap();
    lemma_chat::store::insert_user_message(&pool, c1.id, "alice 的消息")
        .await
        .unwrap();
    lemma_chat::store::insert_user_message(&pool, c2.id, "erin 的消息")
        .await
        .unwrap();

    let msgs = store::pull_messages(&pool, u1, 0, 10).await.unwrap();
    assert_eq!(msgs.len(), 1);
    assert_eq!(msgs[0].content, "alice 的消息");
}

#[sqlx::test(migrations = "../lemma-db/migrations")]
async fn head_sync_seq_tracks_latest_change(pool: PgPool) {
    let uid = new_user(&pool, "alice").await;
    let c = lemma_conversations::store::insert(&pool, uid)
        .await
        .unwrap();
    let head = store::head_sync_seq(&pool).await.unwrap();
    assert!(head >= c.sync_seq);
}
