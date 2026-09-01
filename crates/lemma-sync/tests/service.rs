#![allow(clippy::unwrap_used)]

use buffa::Message;
use connectrpc::{
    CodecFormat, Encodable, HasMessageView, JsonSerialize, RequestContext, ServiceRequest,
};
use futures::StreamExt;
use http::HeaderMap;
use lemma_auth::{sign_access_token, users};
use lemma_proto::lemma::v1::__buffa::oneof::watch_response::Kind;
use lemma_proto::lemma::v1::SyncService as SyncServiceRpc;
use lemma_proto::lemma::v1::{PullRequest, PullResponse, WatchRequest};
use lemma_sync::SyncService;
use sqlx::PgPool;
use uuid::Uuid;

const JWT_SECRET: &str = "jwt-test";

async fn new_user_token(pool: &PgPool, name: &str) -> (Uuid, String) {
    let uid = users::insert(pool, name, &format!("{name}@example.com"), "hash")
        .await
        .unwrap()
        .id;
    let token = sign_access_token(JWT_SECRET, uid).unwrap();
    (uid, token)
}

fn bearer_ctx(token: &str) -> RequestContext {
    let mut headers = HeaderMap::new();
    headers.insert(
        http::header::AUTHORIZATION,
        format!("Bearer {token}").parse().unwrap(),
    );
    RequestContext::new(headers)
}

fn owned_body<M>(body: &impl Encodable<M>) -> M
where
    M: Message + JsonSerialize,
{
    let bytes = body.encode(CodecFormat::Proto).unwrap();
    M::decode(&mut &bytes[..]).unwrap()
}

async fn pull(
    svc: &SyncService,
    token: &str,
    after: i64,
) -> Result<PullResponse, connectrpc::ConnectError> {
    let msg = PullRequest {
        after,
        ..Default::default()
    };
    let bytes = msg.encode_to_bytes();
    let view = PullRequest::decode_view(&bytes).unwrap();
    match svc
        .pull(bearer_ctx(token), ServiceRequest::from_parts(&view, &bytes))
        .await
    {
        Ok(resp) => Ok(owned_body(&resp.body)),
        Err(e) => Err(e),
    }
}

#[sqlx::test(migrations = "../lemma-db/migrations")]
async fn pull_assembles_changes_and_archived(pool: PgPool) {
    let (uid, token) = new_user_token(&pool, "alice").await;
    let svc = SyncService::new(pool.clone(), JWT_SECRET);

    let c = lemma_conversations::store::insert(&pool, uid)
        .await
        .unwrap();
    lemma_chat::store::insert_user_message(&pool, c.id, "hi")
        .await
        .unwrap();
    let archived = lemma_conversations::store::insert(&pool, uid)
        .await
        .unwrap();
    lemma_conversations::store::archive(&pool, archived.id, uid)
        .await
        .unwrap();

    let r = pull(&svc, &token, 0).await.unwrap();
    assert_eq!(r.conversations.len(), 2);
    assert_eq!(r.messages.len(), 1);
    assert_eq!(r.archived.len(), 1);
    assert!(!r.has_more);
    assert!(r.next_after > 0);

    let r2 = pull(&svc, &token, r.next_after).await.unwrap();
    assert_eq!(r2.conversations.len(), 0);
    assert_eq!(r2.messages.len(), 0);
    assert!(!r2.has_more);
    assert_eq!(r2.next_after, r.next_after);
}

#[sqlx::test(migrations = "../lemma-db/migrations")]
async fn pull_paginates_without_loss(pool: PgPool) {
    let (uid, token) = new_user_token(&pool, "alice").await;
    let svc = SyncService::new(pool.clone(), JWT_SECRET);
    sqlx::query("INSERT INTO conversations (user_id) SELECT $1 FROM generate_series(1, 501)")
        .bind(uid)
        .execute(&pool)
        .await
        .unwrap();

    let mut seen: Vec<String> = Vec::new();
    let mut after = 0;
    loop {
        let r = pull(&svc, &token, after).await.unwrap();
        seen.extend(
            r.conversations
                .iter()
                .map(|c| c.conversation.as_option().unwrap().id.clone()),
        );
        after = r.next_after;
        if !r.has_more {
            break;
        }
    }
    seen.sort();
    seen.dedup();
    assert_eq!(seen.len(), 501);
}

#[sqlx::test(migrations = "../lemma-db/migrations")]
async fn watch_emits_initial_hint(pool: PgPool) {
    let (_uid, token) = new_user_token(&pool, "alice").await;
    let svc = SyncService::new(pool.clone(), JWT_SECRET);

    let msg = WatchRequest::default();
    let bytes = msg.encode_to_bytes();
    let view = WatchRequest::decode_view(&bytes).unwrap();
    let mut stream = svc
        .watch(
            bearer_ctx(&token),
            ServiceRequest::from_parts(&view, &bytes),
        )
        .await
        .unwrap()
        .body;

    let first = tokio::time::timeout(std::time::Duration::from_secs(8), stream.next())
        .await
        .unwrap()
        .unwrap()
        .unwrap();
    assert!(matches!(
        first.kind.as_ref(),
        Some(Kind::Hint(h)) if h.sync_seq > 0
    ));
}

#[sqlx::test(migrations = "../lemma-db/migrations")]
async fn pull_truncation_boundary_uses_min_across_tables(pool: PgPool) {
    let (uid, token) = new_user_token(&pool, "alice").await;
    let svc = SyncService::new(pool.clone(), JWT_SECRET);
    let anchor = lemma_conversations::store::insert(&pool, uid)
        .await
        .unwrap();
    sqlx::query(
        "INSERT INTO messages (id, conversation_id, role, content, seq)
         SELECT gen_random_uuid(), $1, 'user', 'bulk', g FROM generate_series(1, 501) AS g",
    )
    .bind(anchor.id)
    .execute(&pool)
    .await
    .unwrap();
    sqlx::query("INSERT INTO conversations (user_id) SELECT $1 FROM generate_series(1, 501)")
        .bind(uid)
        .execute(&pool)
        .await
        .unwrap();

    let first = pull(&svc, &token, 0).await.unwrap();
    assert_eq!(first.conversations.len(), 1);
    assert_eq!(first.messages.len(), 500);
    assert!(first.has_more);

    let mut conv_ids: Vec<String> = first
        .conversations
        .iter()
        .map(|c| c.conversation.as_option().unwrap().id.clone())
        .collect();
    let mut msg_ids: Vec<String> = first
        .messages
        .iter()
        .map(|m| m.message.as_option().unwrap().id.clone())
        .collect();
    let mut after = first.next_after;
    loop {
        let r = pull(&svc, &token, after).await.unwrap();
        conv_ids.extend(
            r.conversations
                .iter()
                .map(|c| c.conversation.as_option().unwrap().id.clone()),
        );
        msg_ids.extend(
            r.messages
                .iter()
                .map(|m| m.message.as_option().unwrap().id.clone()),
        );
        after = r.next_after;
        if !r.has_more {
            break;
        }
    }
    conv_ids.sort();
    conv_ids.dedup();
    msg_ids.sort();
    msg_ids.dedup();
    assert_eq!(conv_ids.len(), 502);
    assert_eq!(msg_ids.len(), 501);
}

#[sqlx::test(migrations = "../lemma-db/migrations")]
async fn pull_msg_only_truncation_paginates(pool: PgPool) {
    let (uid, token) = new_user_token(&pool, "alice").await;
    let svc = SyncService::new(pool.clone(), JWT_SECRET);
    let c = lemma_conversations::store::insert(&pool, uid)
        .await
        .unwrap();
    sqlx::query(
        "INSERT INTO messages (id, conversation_id, role, content, seq)
         SELECT gen_random_uuid(), $1, 'user', 'bulk', g FROM generate_series(1, 502) AS g",
    )
    .bind(c.id)
    .execute(&pool)
    .await
    .unwrap();

    let first = pull(&svc, &token, 0).await.unwrap();
    assert_eq!(first.conversations.len(), 1);
    assert_eq!(first.messages.len(), 500);
    assert!(first.has_more);

    let rest = pull(&svc, &token, first.next_after).await.unwrap();
    assert_eq!(rest.conversations.len(), 0);
    assert_eq!(rest.messages.len(), 2);
    assert!(!rest.has_more);
}
