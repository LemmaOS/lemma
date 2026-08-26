#![allow(clippy::unwrap_used)]

use buffa::Message;
use connectrpc::{CodecFormat, Encodable, JsonSerialize};
use connectrpc::{ErrorCode, HasMessageView, RequestContext, ServiceRequest};
use http::HeaderMap;
use lemma_archive::MemoryArchiveStore;
use lemma_archive::{ArchiveError, ArchiveStore, object_key};
use lemma_auth::{sign_access_token, users};
use lemma_conversations::ConversationService;
use lemma_proto::lemma::v1::ConversationService as ConversationServiceRpc;
use lemma_proto::lemma::v1::MessageStatus;
use sqlx::PgPool;
use std::sync::Arc;
use uuid::Uuid;

type Svc = ConversationService<MemoryArchiveStore>;

const SECRET: &str = "test-secret";

async fn new_user(pool: &PgPool) -> (Uuid, String) {
    let name = format!("u-{}", Uuid::new_v4());
    let id = users::insert(pool, &name, &format!("{name}@example.com"), "hash")
        .await
        .unwrap()
        .id;
    let token = sign_access_token(SECRET, id).unwrap();
    (id, token)
}

fn bearer_ctx(token: &str) -> RequestContext {
    let mut headers = HeaderMap::new();
    headers.insert(
        http::header::AUTHORIZATION,
        format!("Bearer {token}").parse().unwrap(),
    );
    RequestContext::new(headers)
}

// 经 wire 编解码还原具体消息。两侧推导一致：rustc 走 M: Encodable<M> 自反实现，
// rust-analyzer 走不透明类型的 Encodable<M> 参数化——绕开 RA 对 RPITIT 精化的误报
fn owned_body<M>(body: &impl Encodable<M>) -> M
where
    M: Message + JsonSerialize,
{
    let bytes = body.encode(CodecFormat::Proto).unwrap();
    M::decode(&mut &bytes[..]).unwrap()
}

// 返回完整响应；conversation 字段经 MessageField Deref 取值
async fn create(svc: &Svc, token: &str) -> lemma_proto::lemma::v1::CreateConversationResponse {
    let msg = lemma_proto::lemma::v1::CreateConversationRequest::default();
    let bytes = msg.encode_to_bytes();
    let view = lemma_proto::lemma::v1::CreateConversationRequest::decode_view(&bytes).unwrap();
    let resp = svc
        .create_conversation(bearer_ctx(token), ServiceRequest::from_parts(&view, &bytes))
        .await
        .unwrap();
    owned_body(&resp.body)
}

async fn list_active_count(svc: &Svc, token: &str) -> usize {
    let msg = lemma_proto::lemma::v1::ListConversationsRequest::default();
    let bytes = msg.encode_to_bytes();
    let view = lemma_proto::lemma::v1::ListConversationsRequest::decode_view(&bytes).unwrap();
    svc.list_conversations(bearer_ctx(token), ServiceRequest::from_parts(&view, &bytes))
        .await
        .unwrap()
        .body
        .conversations
        .len()
}

async fn list_archived_count(svc: &Svc, token: &str) -> usize {
    let msg = lemma_proto::lemma::v1::ListArchivedRequest::default();
    let bytes = msg.encode_to_bytes();
    let view = lemma_proto::lemma::v1::ListArchivedRequest::decode_view(&bytes).unwrap();
    svc.list_archived(bearer_ctx(token), ServiceRequest::from_parts(&view, &bytes))
        .await
        .unwrap()
        .body
        .conversations
        .len()
}

async fn archive(
    svc: &Svc,
    token: &str,
    id: &str,
) -> Result<lemma_proto::lemma::v1::ArchiveConversationResponse, connectrpc::ConnectError> {
    let msg = lemma_proto::lemma::v1::ArchiveConversationRequest {
        id: id.into(),
        ..Default::default()
    };
    let bytes = msg.encode_to_bytes();
    let view = lemma_proto::lemma::v1::ArchiveConversationRequest::decode_view(&bytes).unwrap();
    match svc
        .archive_conversation(bearer_ctx(token), ServiceRequest::from_parts(&view, &bytes))
        .await
    {
        Ok(resp) => Ok(owned_body(&resp.body)),
        Err(e) => Err(e),
    }
}

async fn restore(
    svc: &Svc,
    token: &str,
    id: &str,
) -> Result<lemma_proto::lemma::v1::RestoreConversationResponse, connectrpc::ConnectError> {
    let msg = lemma_proto::lemma::v1::RestoreConversationRequest {
        id: id.into(),
        ..Default::default()
    };
    let bytes = msg.encode_to_bytes();
    let view = lemma_proto::lemma::v1::RestoreConversationRequest::decode_view(&bytes).unwrap();
    match svc
        .restore_conversation(bearer_ctx(token), ServiceRequest::from_parts(&view, &bytes))
        .await
    {
        Ok(resp) => Ok(owned_body(&resp.body)),
        Err(e) => Err(e),
    }
}

async fn delete_archived(
    svc: &Svc,
    token: &str,
    id: &str,
) -> Result<lemma_proto::lemma::v1::DeleteArchivedResponse, connectrpc::ConnectError> {
    let msg = lemma_proto::lemma::v1::DeleteArchivedRequest {
        id: id.into(),
        ..Default::default()
    };
    let bytes = msg.encode_to_bytes();
    let view = lemma_proto::lemma::v1::DeleteArchivedRequest::decode_view(&bytes).unwrap();
    match svc
        .delete_archived(bearer_ctx(token), ServiceRequest::from_parts(&view, &bytes))
        .await
    {
        Ok(resp) => Ok(owned_body(&resp.body)),
        Err(e) => Err(e),
    }
}

// 直插消息构造归档素材（seq 从 1 递增）
async fn seed_messages(pool: &PgPool, conv: &str, contents: &[&str]) {
    let conv = Uuid::parse_str(conv).unwrap();
    for (i, content) in contents.iter().enumerate() {
        sqlx::query(
            "INSERT INTO messages (id, conversation_id, role, content, seq) VALUES ($1, $2, 'user', $3, $4)",
        )
        .bind(Uuid::new_v4())
        .bind(conv)
        .bind(content)
        .bind(i as i64 + 1)
        .execute(pool)
        .await
        .unwrap();
    }
}

async fn message_contents(pool: &PgPool, conv: &str) -> Vec<String> {
    sqlx::query_scalar("SELECT content FROM messages WHERE conversation_id = $1 ORDER BY seq")
        .bind(Uuid::parse_str(conv).unwrap())
        .fetch_all(pool)
        .await
        .unwrap()
}

async fn rename(
    svc: &Svc,
    token: &str,
    id: &str,
    title: &str,
) -> Result<lemma_proto::lemma::v1::RenameConversationResponse, connectrpc::ConnectError> {
    let msg = lemma_proto::lemma::v1::RenameConversationRequest {
        id: id.into(),
        title: title.into(),
        ..Default::default()
    };
    let bytes = msg.encode_to_bytes();
    let view = lemma_proto::lemma::v1::RenameConversationRequest::decode_view(&bytes).unwrap();
    match svc
        .rename_conversation(bearer_ctx(token), ServiceRequest::from_parts(&view, &bytes))
        .await
    {
        Ok(resp) => Ok(owned_body(&resp.body)),
        Err(e) => Err(e),
    }
}

async fn list_messages(
    svc: &Svc,
    token: &str,
    conv: &str,
) -> Result<lemma_proto::lemma::v1::ListMessagesResponse, connectrpc::ConnectError> {
    let msg = lemma_proto::lemma::v1::ListMessagesRequest {
        conversation_id: conv.into(),
        ..Default::default()
    };
    let bytes = msg.encode_to_bytes();
    let view = lemma_proto::lemma::v1::ListMessagesRequest::decode_view(&bytes).unwrap();
    match svc
        .list_messages(bearer_ctx(token), ServiceRequest::from_parts(&view, &bytes))
        .await
    {
        Ok(resp) => Ok(owned_body(&resp.body)),
        Err(e) => Err(e),
    }
}

// 全字段直插一条消息，绕过 chat 链路
#[allow(clippy::too_many_arguments)]
async fn insert_msg(pool: &PgPool, conv: Uuid, seq: i64, status: &str, model: Option<&str>) {
    sqlx::query(
        "INSERT INTO messages (id, conversation_id, role, content, status, model, seq)
         VALUES ($1, $2, 'assistant', $3, $4, $5, $6)",
    )
    .bind(Uuid::new_v4())
    .bind(conv)
    .bind(format!("c{seq}"))
    .bind(status)
    .bind(model)
    .bind(seq)
    .execute(pool)
    .await
    .unwrap();
}

#[sqlx::test(migrations = "../lemma-db/migrations")]
async fn create_and_list(pool: PgPool) {
    let svc = ConversationService::new(pool.clone(), SECRET, None::<Arc<MemoryArchiveStore>>);
    let (_, token) = new_user(&pool).await;
    let created = create(&svc, &token).await;
    assert_eq!(created.conversation.title, "");
    assert_eq!(list_active_count(&svc, &token).await, 1);
}

#[sqlx::test(migrations = "../lemma-db/migrations")]
async fn rename_not_found_and_cross_user(pool: PgPool) {
    let svc = ConversationService::new(pool.clone(), SECRET, None::<Arc<MemoryArchiveStore>>);
    let (_, alice) = new_user(&pool).await;
    let (_, erin) = new_user(&pool).await;
    let id = create(&svc, &alice).await.conversation.id.clone();

    let err = rename(&svc, &alice, &Uuid::new_v4().to_string(), "x")
        .await
        .err()
        .unwrap();
    assert_eq!(err.code, ErrorCode::NotFound);

    let err = rename(&svc, &erin, &id, "hack").await.err().unwrap();
    assert_eq!(err.code, ErrorCode::NotFound);

    let ok = rename(&svc, &alice, &id, "我的会话").await.unwrap();
    assert_eq!(ok.conversation.title, "我的会话");
}

#[sqlx::test(migrations = "../lemma-db/migrations")]
async fn archive_restore_flow(pool: PgPool) {
    let svc = ConversationService::new(pool.clone(), SECRET, None::<Arc<MemoryArchiveStore>>);
    let (_, token) = new_user(&pool).await;
    let id = create(&svc, &token).await.conversation.id.clone();

    let msg = lemma_proto::lemma::v1::ArchiveConversationRequest {
        id: id.clone(),
        ..Default::default()
    };
    let bytes = msg.encode_to_bytes();
    let view = lemma_proto::lemma::v1::ArchiveConversationRequest::decode_view(&bytes).unwrap();
    svc.archive_conversation(
        bearer_ctx(&token),
        ServiceRequest::from_parts(&view, &bytes),
    )
    .await
    .unwrap();

    assert_eq!(list_active_count(&svc, &token).await, 0);
    assert_eq!(list_archived_count(&svc, &token).await, 1);

    let msg = lemma_proto::lemma::v1::RestoreConversationRequest {
        id,
        ..Default::default()
    };
    let bytes = msg.encode_to_bytes();
    let view = lemma_proto::lemma::v1::RestoreConversationRequest::decode_view(&bytes).unwrap();
    svc.restore_conversation(
        bearer_ctx(&token),
        ServiceRequest::from_parts(&view, &bytes),
    )
    .await
    .unwrap();

    assert_eq!(list_active_count(&svc, &token).await, 1);
    assert_eq!(list_archived_count(&svc, &token).await, 0);
}

#[sqlx::test(migrations = "../lemma-db/migrations")]
async fn list_messages_isolated(pool: PgPool) {
    let svc = ConversationService::new(pool.clone(), SECRET, None::<Arc<MemoryArchiveStore>>);
    let (_, alice) = new_user(&pool).await;
    let (_, erin) = new_user(&pool).await;
    let id = create(&svc, &alice).await.conversation.id.clone();

    let msg = lemma_proto::lemma::v1::ListMessagesRequest {
        conversation_id: id,
        ..Default::default()
    };
    let bytes = msg.encode_to_bytes();
    let view = lemma_proto::lemma::v1::ListMessagesRequest::decode_view(&bytes).unwrap();

    let r = svc
        .list_messages(
            bearer_ctx(&alice),
            ServiceRequest::from_parts(&view, &bytes),
        )
        .await
        .unwrap()
        .body;
    assert!(r.messages.is_empty());
    assert!(!r.has_more);

    let err = svc
        .list_messages(bearer_ctx(&erin), ServiceRequest::from_parts(&view, &bytes))
        .await
        .err()
        .unwrap();
    assert_eq!(err.code, ErrorCode::NotFound);
}

#[sqlx::test(migrations = "../lemma-db/migrations")]
async fn archive_moves_content_to_store(pool: PgPool) {
    let store = Arc::new(MemoryArchiveStore::new());
    let svc = ConversationService::new(pool.clone(), SECRET, Some(store.clone()));
    let (_, token) = new_user(&pool).await;
    let id = create(&svc, &token).await.conversation.id.clone();
    seed_messages(&pool, &id, &["一", "二", "三"]).await;

    archive(&svc, &token, &id).await.unwrap();

    // PG 只剩元数据，内容进对象
    assert!(message_contents(&pool, &id).await.is_empty());
    let key = object_key(Uuid::parse_str(&id).unwrap());
    let bytes = store.get(&key).await.unwrap().unwrap();
    let envelope = lemma_archive::deserialize_envelope(&bytes).unwrap();
    assert_eq!(envelope.messages.len(), 3);
    assert_eq!(envelope.messages[0].content, "一");
    assert_eq!(list_archived_count(&svc, &token).await, 1);
}

#[sqlx::test(migrations = "../lemma-db/migrations")]
async fn restore_reinserts_content_in_order(pool: PgPool) {
    let store = Arc::new(MemoryArchiveStore::new());
    let svc = ConversationService::new(pool.clone(), SECRET, Some(store.clone()));
    let (_, token) = new_user(&pool).await;
    let id = create(&svc, &token).await.conversation.id.clone();
    seed_messages(&pool, &id, &["一", "二", "三"]).await;
    archive(&svc, &token, &id).await.unwrap();

    restore(&svc, &token, &id).await.unwrap();

    // 回灌保序（原 seq 生效）
    assert_eq!(message_contents(&pool, &id).await, ["一", "二", "三"]);
    // 对象已清理
    let key = object_key(Uuid::parse_str(&id).unwrap());
    assert!(store.get(&key).await.unwrap().is_none());
}

#[sqlx::test(migrations = "../lemma-db/migrations")]
async fn restore_legacy_in_place_archive_keeps_messages(pool: PgPool) {
    // 历史就地归档（archive_key 为空、消息还在 PG）：解档只翻状态，消息原样保留
    let store = Arc::new(MemoryArchiveStore::new());
    let svc = ConversationService::new(pool.clone(), SECRET, Some(store.clone()));
    let (_, token) = new_user(&pool).await;
    let id = create(&svc, &token).await.conversation.id.clone();
    seed_messages(&pool, &id, &["旧"]).await;

    // 直接 SQL 模拟旧版就地归档：不写对象、不删消息
    sqlx::query(
        "UPDATE conversations SET status = 'archived', archived_at = now(), sync_seq = nextval('sync_seq') WHERE id = $1",
    )
    .bind(Uuid::parse_str(&id).unwrap())
    .execute(&pool)
    .await
    .unwrap();

    restore(&svc, &token, &id).await.unwrap();

    assert_eq!(message_contents(&pool, &id).await, ["旧"]);
}

#[sqlx::test(migrations = "../lemma-db/migrations")]
async fn delete_archived_removes_object(pool: PgPool) {
    let store = Arc::new(MemoryArchiveStore::new());
    let svc = ConversationService::new(pool.clone(), SECRET, Some(store.clone()));
    let (_, token) = new_user(&pool).await;
    let id = create(&svc, &token).await.conversation.id.clone();
    seed_messages(&pool, &id, &["x"]).await;
    archive(&svc, &token, &id).await.unwrap();
    let key = object_key(Uuid::parse_str(&id).unwrap());
    assert!(store.get(&key).await.unwrap().is_some());

    delete_archived(&svc, &token, &id).await.unwrap();

    assert!(store.get(&key).await.unwrap().is_none());
    assert_eq!(list_archived_count(&svc, &token).await, 0);
}

#[sqlx::test(migrations = "../lemma-db/migrations")]
async fn degrade_mode_keeps_content_in_pg(pool: PgPool) {
    // 未配置对象存储：就地归档，消息留在 PG（旧行为）
    let svc = ConversationService::new(pool.clone(), SECRET, None::<Arc<MemoryArchiveStore>>);
    let (_, token) = new_user(&pool).await;
    let id = create(&svc, &token).await.conversation.id.clone();
    seed_messages(&pool, &id, &["留"]).await;

    archive(&svc, &token, &id).await.unwrap();

    assert_eq!(message_contents(&pool, &id).await, ["留"]);
}

// 读回消息：四种状态映射、model 空值回退、倒序分页（最新在前）
#[sqlx::test(migrations = "../lemma-db/migrations")]
async fn list_messages_maps_statuses_and_fields(pool: PgPool) {
    let svc = ConversationService::new(pool.clone(), SECRET, None::<Arc<MemoryArchiveStore>>);
    let (_, token) = new_user(&pool).await;
    let id = create(&svc, &token).await.conversation.id.clone();
    let conv = Uuid::parse_str(&id).unwrap();
    insert_msg(&pool, conv, 1, "done", None).await;
    insert_msg(&pool, conv, 2, "streaming", Some("gpt-x")).await;
    insert_msg(&pool, conv, 3, "aborted", None).await;
    insert_msg(&pool, conv, 4, "error", None).await;

    let r = list_messages(&svc, &token, &id).await.unwrap();
    // 最新在前
    assert_eq!(r.messages[0].status, MessageStatus::Error);
    assert_eq!(r.messages[1].status, MessageStatus::Aborted);
    assert_eq!(r.messages[2].status, MessageStatus::Streaming);
    assert_eq!(r.messages[3].status, MessageStatus::Done);
    assert_eq!(r.messages[2].model, "gpt-x");
    assert_eq!(r.messages[0].model, "");
    assert_eq!(r.messages[0].content, "c4");
    assert_eq!(r.messages[3].seq, 1);
    assert!(!r.has_more);

    // 非法 before_id → InvalidArgument
    let msg = lemma_proto::lemma::v1::ListMessagesRequest {
        conversation_id: id.clone(),
        before_id: "not-a-uuid".into(),
        ..Default::default()
    };
    let bytes = msg.encode_to_bytes();
    let view = lemma_proto::lemma::v1::ListMessagesRequest::decode_view(&bytes).unwrap();
    let err = svc
        .list_messages(
            bearer_ctx(&token),
            ServiceRequest::from_parts(&view, &bytes),
        )
        .await
        .err()
        .unwrap();
    assert_eq!(err.code, ErrorCode::InvalidArgument);
}

// 对象存储故障：归档映射 internal，且事务回滚——会话保持 active 不丢数据
struct FailingStore;

impl ArchiveStore for FailingStore {
    async fn put(&self, _key: &str, _content: &[u8]) -> Result<(), ArchiveError> {
        Err(ArchiveError("s3 down".into()))
    }
    async fn get(&self, _key: &str) -> Result<Option<Vec<u8>>, ArchiveError> {
        Err(ArchiveError("s3 down".into()))
    }
    async fn delete(&self, _key: &str) -> Result<(), ArchiveError> {
        Err(ArchiveError("s3 down".into()))
    }
}

#[sqlx::test(migrations = "../lemma-db/migrations")]
async fn archive_store_failure_maps_internal_and_rolls_back(pool: PgPool) {
    let svc = ConversationService::new(pool.clone(), SECRET, Some(Arc::new(FailingStore)));
    let (_, token) = new_user(&pool).await;
    // FailingStore 与 helper 的类型别名不同，这里走内联调用（同 archive_restore_flow 惯例）
    let msg = lemma_proto::lemma::v1::CreateConversationRequest::default();
    let bytes = msg.encode_to_bytes();
    let view = lemma_proto::lemma::v1::CreateConversationRequest::decode_view(&bytes).unwrap();
    let created = svc
        .create_conversation(
            bearer_ctx(&token),
            ServiceRequest::from_parts(&view, &bytes),
        )
        .await
        .unwrap();
    let id = owned_body(&created.body).conversation.id.clone();
    seed_messages(&pool, &id, &["一"]).await;

    let err = archive_generic(&svc, &token, &id).await.err().unwrap();
    assert_eq!(err.code, ErrorCode::Internal);

    // 回滚验证：状态还是 active，消息一条不少
    let status: String = sqlx::query_scalar("SELECT status FROM conversations WHERE id = $1")
        .bind(Uuid::parse_str(&id).unwrap())
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(status, "active");
    assert_eq!(message_contents(&pool, &id).await, ["一"]);
}

// 归档请求的内联泛型版：svc 的 ArchiveStore 类型参数任意
async fn archive_generic<A: ArchiveStore>(
    svc: &ConversationService<A>,
    token: &str,
    id: &str,
) -> Result<lemma_proto::lemma::v1::ArchiveConversationResponse, connectrpc::ConnectError> {
    let msg = lemma_proto::lemma::v1::ArchiveConversationRequest {
        id: id.into(),
        ..Default::default()
    };
    let bytes = msg.encode_to_bytes();
    let view = lemma_proto::lemma::v1::ArchiveConversationRequest::decode_view(&bytes).unwrap();
    match svc
        .archive_conversation(bearer_ctx(token), ServiceRequest::from_parts(&view, &bytes))
        .await
    {
        Ok(resp) => Ok(owned_body(&resp.body)),
        Err(e) => Err(e),
    }
}
