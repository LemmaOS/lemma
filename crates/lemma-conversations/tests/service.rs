#![allow(clippy::unwrap_used)]

use buffa::Message;
use connectrpc::{ErrorCode, HasMessageView, RequestContext, ServiceRequest};
use http::HeaderMap;
use lemma_auth::{sign_access_token, users};
use lemma_conversations::ConversationService;
use lemma_proto::lemma::v1::ConversationService as ConversationServiceRpc;
use sqlx::PgPool;
use uuid::Uuid;

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

// 返回完整响应；conversation 字段经 MessageField Deref 取值
async fn create(
    svc: &ConversationService,
    token: &str,
) -> lemma_proto::lemma::v1::CreateConversationResponse {
    let msg = lemma_proto::lemma::v1::CreateConversationRequest::default();
    let bytes = msg.encode_to_bytes();
    let view = lemma_proto::lemma::v1::CreateConversationRequest::decode_view(&bytes).unwrap();
    svc.create_conversation(bearer_ctx(token), ServiceRequest::from_parts(&view, &bytes))
        .await
        .unwrap()
        .body
}

async fn list_active_count(svc: &ConversationService, token: &str) -> usize {
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

async fn list_archived_count(svc: &ConversationService, token: &str) -> usize {
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

async fn rename(
    svc: &ConversationService,
    token: &str,
    id: &str,
    title: &str,
) -> connectrpc::ServiceResult<lemma_proto::lemma::v1::RenameConversationResponse> {
    let msg = lemma_proto::lemma::v1::RenameConversationRequest {
        id: id.into(),
        title: title.into(),
        ..Default::default()
    };
    let bytes = msg.encode_to_bytes();
    let view = lemma_proto::lemma::v1::RenameConversationRequest::decode_view(&bytes).unwrap();
    svc.rename_conversation(bearer_ctx(token), ServiceRequest::from_parts(&view, &bytes))
        .await
}

#[sqlx::test(migrations = "../lemma-db/migrations")]
async fn create_and_list(pool: PgPool) {
    let svc = ConversationService::new(pool.clone(), SECRET);
    let (_, token) = new_user(&pool).await;
    let created = create(&svc, &token).await;
    assert_eq!(created.conversation.title, "");
    assert_eq!(list_active_count(&svc, &token).await, 1);
}

#[sqlx::test(migrations = "../lemma-db/migrations")]
async fn rename_not_found_and_cross_user(pool: PgPool) {
    let svc = ConversationService::new(pool.clone(), SECRET);
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
    assert_eq!(ok.body.conversation.title, "我的会话");
}

#[sqlx::test(migrations = "../lemma-db/migrations")]
async fn archive_restore_flow(pool: PgPool) {
    let svc = ConversationService::new(pool.clone(), SECRET);
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
    let svc = ConversationService::new(pool.clone(), SECRET);
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
