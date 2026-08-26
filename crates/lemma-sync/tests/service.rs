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

// 经 wire 编解码还原具体消息：rustc 走 M: Encodable<M> 自反实现，rust-analyzer 走
// 不透明类型的 Encodable<M> 参数化，两侧推导一致，绕开 RA 对 RPITIT 精化的误报
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
    assert_eq!(r.conversations.len(), 2); // active 创建 + 归档变更各产生一行
    assert_eq!(r.messages.len(), 1);
    assert_eq!(r.archived.len(), 1); // 归档元数据全量
    assert!(!r.has_more);
    assert!(r.next_after > 0);

    // 游标推进后再拉：空页，has_more=false
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
    // 一次插入 501 行，超过单页 500
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
    // 501 行全部拉到，无丢失无重复
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

    // 首个轮询周期（3s）内应收到 hint；留 8s 余量
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

// 双表都截断：边界取两表较小者，边界外的行丢弃等下轮（跨表不丢变更）
#[sqlx::test(migrations = "../lemma-db/migrations")]
async fn pull_truncation_boundary_uses_min_across_tables(pool: PgPool) {
    let (uid, token) = new_user_token(&pool, "alice").await;
    let svc = SyncService::new(pool.clone(), JWT_SECRET);
    // 锚点会话先建，随后 501 条消息（sync_seq 靠前）、501 条会话（sync_seq 靠后）：
    // 首页双表截断，边界被消息侧压低，靠后的会话全部越界
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

    // 首页：会话只剩锚点（其余越界丢弃），消息截到 500
    let first = pull(&svc, &token, 0).await.unwrap();
    assert_eq!(first.conversations.len(), 1);
    assert_eq!(first.messages.len(), 500);
    assert!(first.has_more);

    // 拉完整轮：502 会话 + 501 消息一条不少、一条不重
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

// 只有消息侧超页限：边界取消息末行，会话原样带回
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
    assert_eq!(first.conversations.len(), 1); // 会话不截断
    assert_eq!(first.messages.len(), 500); // 消息截断 + 丢弃探测行
    assert!(first.has_more);

    let rest = pull(&svc, &token, first.next_after).await.unwrap();
    assert_eq!(rest.conversations.len(), 0);
    assert_eq!(rest.messages.len(), 2);
    assert!(!rest.has_more);
}
