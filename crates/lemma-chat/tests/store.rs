#![allow(clippy::unwrap_used, missing_docs)]

use lemma_auth::users;
use lemma_chat::store;
use lemma_db::entity::TokenUsage;
use lemma_providers::providers::{self, NewProvider};
use sqlx::PgPool;
use uuid::Uuid;

async fn new_fixture(pool: &PgPool, name: &str) -> (Uuid, Uuid, Uuid) {
    let uid = users::insert(pool, name, &format!("{name}@example.com"), "hash")
        .await
        .unwrap()
        .id;
    let pid = providers::insert(
        pool,
        &NewProvider {
            id: Uuid::new_v4(),
            user_id: uid,
            kind: "openai",
            name: "p",
            base_url: "https://api.example.com/v1",
            api_key: "sealed",
            api_path: "",
            models_path: "",
            models: &[],
        },
    )
    .await
    .unwrap()
    .id;
    let cid = lemma_conversations::store::insert(pool, uid)
        .await
        .unwrap()
        .id;
    (uid, pid, cid)
}

#[sqlx::test(migrations = "../lemma-db/migrations")]
async fn placeholder_then_finalize_roundtrip(pool: PgPool) {
    let (_uid, pid, cid) = new_fixture(&pool, "alice").await;
    store::insert_user_message(&pool, cid, "你好")
        .await
        .unwrap();
    let a = store::insert_assistant_placeholder(&pool, cid, pid, "gpt-x", Some("cmsg-1"))
        .await
        .unwrap();
    assert_eq!(a.status, "streaming");
    assert_eq!(a.client_msg_id.as_deref(), Some("cmsg-1"));

    let usage = TokenUsage {
        prompt: 3,
        completion: 2,
        total: 5,
    };
    let done = store::finalize(&pool, a.id, "你好呀", Some(usage))
        .await
        .unwrap()
        .unwrap();
    assert_eq!(done.status, "done");
    assert_eq!(done.content, "你好呀");
    assert_eq!(done.token_usage.unwrap().0.total, 5);

    let found = store::find_assistant_by_client_msg_id(&pool, cid, "cmsg-1")
        .await
        .unwrap()
        .unwrap();
    assert_eq!(found.id, a.id);
}

#[sqlx::test(migrations = "../lemma-db/migrations")]
async fn client_msg_id_unique_per_conversation(pool: PgPool) {
    let (_uid, pid, cid) = new_fixture(&pool, "alice").await;
    store::insert_assistant_placeholder(&pool, cid, pid, "gpt-x", Some("dup"))
        .await
        .unwrap();
    assert!(
        store::insert_assistant_placeholder(&pool, cid, pid, "gpt-x", Some("dup"))
            .await
            .is_err()
    );
    let cid2 = lemma_conversations::store::insert(&pool, _uid)
        .await
        .unwrap()
        .id;
    store::insert_assistant_placeholder(&pool, cid2, pid, "gpt-x", Some("dup"))
        .await
        .unwrap();
}

#[sqlx::test(migrations = "../lemma-db/migrations")]
async fn list_context_excludes_streaming_and_orders(pool: PgPool) {
    let (_uid, pid, cid) = new_fixture(&pool, "alice").await;
    store::insert_user_message(&pool, cid, "q1").await.unwrap();
    let a1 = store::insert_assistant_placeholder(&pool, cid, pid, "gpt-x", None)
        .await
        .unwrap();
    store::finalize(&pool, a1.id, "a1", None).await.unwrap();
    store::insert_user_message(&pool, cid, "q2").await.unwrap();
    store::insert_assistant_placeholder(&pool, cid, pid, "gpt-x", None)
        .await
        .unwrap();

    let ctx = store::list_context(&pool, cid).await.unwrap();
    let pairs: Vec<(&str, &str)> = ctx
        .iter()
        .map(|m| (m.role.as_str(), m.content.as_str()))
        .collect();
    assert_eq!(pairs, [("user", "q1"), ("assistant", "a1"), ("user", "q2")]);
}

#[sqlx::test(migrations = "../lemma-db/migrations")]
async fn flush_and_abort_keep_partial(pool: PgPool) {
    let (_uid, pid, cid) = new_fixture(&pool, "alice").await;
    let a = store::insert_assistant_placeholder(&pool, cid, pid, "gpt-x", None)
        .await
        .unwrap();
    let flushed = store::flush_content(&pool, a.id, "半截")
        .await
        .unwrap()
        .unwrap();
    assert_eq!(flushed.status, "streaming");
    assert_eq!(flushed.content, "半截");
    let sync_after_flush = flushed.sync_seq;

    let aborted = store::mark_aborted(&pool, a.id, "半截")
        .await
        .unwrap()
        .unwrap();
    assert_eq!(aborted.status, "aborted");
    assert_eq!(aborted.content, "半截");
    assert!(aborted.sync_seq > sync_after_flush);
}

#[sqlx::test(migrations = "../lemma-db/migrations")]
async fn find_by_id_and_user_enforces_ownership(pool: PgPool) {
    let (_uid, pid, cid) = new_fixture(&pool, "alice").await;
    let a = store::insert_assistant_placeholder(&pool, cid, pid, "gpt-x", None)
        .await
        .unwrap();
    assert!(
        store::find_by_id_and_user(&pool, a.id, _uid)
            .await
            .unwrap()
            .is_some()
    );
    assert!(
        store::find_by_id_and_user(&pool, a.id, Uuid::new_v4())
            .await
            .unwrap()
            .is_none()
    );
}
