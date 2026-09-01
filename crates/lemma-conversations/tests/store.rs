#![allow(clippy::unwrap_used)]

use lemma_auth::users;
use lemma_conversations::store;
use sqlx::PgPool;
use uuid::Uuid;

async fn new_user(pool: &PgPool) -> Uuid {
    let name = format!("u-{}", Uuid::new_v4());
    users::insert(pool, &name, &format!("{name}@example.com"), "hash")
        .await
        .unwrap()
        .id
}

async fn seed_message(pool: &PgPool, conv: Uuid, offset_secs: f64) {
    sqlx::query(
        r#"
        INSERT INTO messages (id, conversation_id, role, content, created_at, seq)
        VALUES ($1, $2, 'user', 'm', now() - make_interval(secs => $3),
                (SELECT COALESCE(MAX(seq), 0) + 1 FROM messages WHERE conversation_id = $2))
        "#,
    )
    .bind(Uuid::new_v4())
    .bind(conv)
    .bind(offset_secs)
    .execute(pool)
    .await
    .unwrap();
}

#[sqlx::test(migrations = "../lemma-db/migrations")]
async fn create_defaults(pool: PgPool) {
    let uid = new_user(&pool).await;
    let c = store::insert(&pool, uid).await.unwrap();
    assert_eq!(c.title, "");
    assert_eq!(c.status, "active");
    assert!(c.archived_at.is_none());
    assert!(c.sync_seq > 0);
}

#[sqlx::test(migrations = "../lemma-db/migrations")]
async fn rename_bumps_sync_seq(pool: PgPool) {
    let uid = new_user(&pool).await;
    let c = store::insert(&pool, uid).await.unwrap();
    let renamed = store::rename(&pool, c.id, uid, "new title")
        .await
        .unwrap()
        .unwrap();
    assert_eq!(renamed.title, "new title");
    assert!(renamed.sync_seq > c.sync_seq);
}

#[sqlx::test(migrations = "../lemma-db/migrations")]
async fn archive_sets_metadata_and_guards(pool: PgPool) {
    let uid = new_user(&pool).await;
    let c = store::insert(&pool, uid).await.unwrap();
    seed_message(&pool, c.id, 60.0).await;
    seed_message(&pool, c.id, 30.0).await;

    let archived = store::archive(&pool, c.id, uid).await.unwrap().unwrap();
    assert_eq!(archived.status, "archived");
    assert!(archived.archived_at.is_some());
    assert_eq!(archived.message_count, Some(2));

    assert!(store::archive(&pool, c.id, uid).await.unwrap().is_none());
}

#[sqlx::test(migrations = "../lemma-db/migrations")]
async fn restore_reactivates_and_guards(pool: PgPool) {
    let uid = new_user(&pool).await;
    let c = store::insert(&pool, uid).await.unwrap();
    store::archive(&pool, c.id, uid).await.unwrap().unwrap();

    let restored = store::restore(&pool, c.id, uid).await.unwrap().unwrap();
    assert_eq!(restored.status, "active");
    assert!(restored.archived_at.is_none());
    assert!(restored.message_count.is_none());

    assert!(store::restore(&pool, c.id, uid).await.unwrap().is_none());
}

#[sqlx::test(migrations = "../lemma-db/migrations")]
async fn active_and_archived_lists_are_disjoint(pool: PgPool) {
    let uid = new_user(&pool).await;
    let a = store::insert(&pool, uid).await.unwrap();
    let b = store::insert(&pool, uid).await.unwrap();
    store::archive(&pool, a.id, uid).await.unwrap().unwrap();

    let active = store::list_active_by_user(&pool, uid).await.unwrap();
    let archived = store::list_archived_by_user(&pool, uid).await.unwrap();
    assert_eq!(active.len(), 1);
    assert_eq!(active[0].id, b.id);
    assert_eq!(archived.len(), 1);
    assert_eq!(archived[0].id, a.id);
}

#[sqlx::test(migrations = "../lemma-db/migrations")]
async fn delete_archived_cascades_messages(pool: PgPool) {
    let uid = new_user(&pool).await;
    let c = store::insert(&pool, uid).await.unwrap();
    seed_message(&pool, c.id, 60.0).await;
    store::archive(&pool, c.id, uid).await.unwrap().unwrap();

    assert!(store::delete_archived(&pool, c.id, uid).await.unwrap());
    let n: i64 = sqlx::query_scalar("SELECT count(*) FROM messages WHERE conversation_id = $1")
        .bind(c.id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(n, 0);

    let live = store::insert(&pool, uid).await.unwrap();
    assert!(!store::delete_archived(&pool, live.id, uid).await.unwrap());
}

#[sqlx::test(migrations = "../lemma-db/migrations")]
async fn message_keyset_pagination(pool: PgPool) {
    let uid = new_user(&pool).await;
    let c = store::insert(&pool, uid).await.unwrap();
    for i in 1..=5 {
        seed_message(&pool, c.id, (6 - i) as f64 * 60.0).await;
    }

    let (page1, more) = store::list_messages(&pool, c.id, None, 2).await.unwrap();
    assert!(more);
    assert_eq!(page1.len(), 2);
    assert!(page1[0].created_at > page1[1].created_at);

    let (page2, more) = store::list_messages(&pool, c.id, Some(page1[1].id), 2)
        .await
        .unwrap();
    assert!(more);
    assert_eq!(page2.len(), 2);

    let (page3, more) = store::list_messages(&pool, c.id, Some(page2[1].id), 2)
        .await
        .unwrap();
    assert!(!more);
    assert_eq!(page3.len(), 1);

    let other = store::insert(&pool, uid).await.unwrap();
    let (empty, more) = store::list_messages(&pool, c.id, Some(other.id), 2)
        .await
        .unwrap();
    assert!(empty.is_empty());
    assert!(!more);
}
