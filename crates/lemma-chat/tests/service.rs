#![allow(clippy::unwrap_used)]

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use buffa::Message;
use connectrpc::{
    CodecFormat, ConnectError, Encodable, ErrorCode, HasMessageView, JsonSerialize, RequestContext,
    ServiceRequest, ServiceStream,
};
use futures::{StreamExt, stream};
use http::HeaderMap;
use lemma_auth::{sign_access_token, users};
use lemma_chat::ChatService;
use lemma_chat::adapter::{
    AdapterError, AdapterEvent, BoxChatFuture, BoxEventStream, ChatRequest, LlmAdapter,
};
use lemma_chat::store;
use lemma_crypto::{derive_key, seal};
use lemma_db::entity::TokenUsage;
use lemma_proto::lemma::v1::__buffa::oneof::chat_event::Kind;
use lemma_proto::lemma::v1::ChatService as ChatServiceRpc;
use lemma_proto::lemma::v1::{
    AbortMessageRequest, AbortMessageResponse, ChatEvent, ResumeStreamRequest,
    ResumeStreamResponse, SendMessageRequest, SendMessageResponse,
};
use lemma_providers::providers::{self, NewProvider};
use sqlx::PgPool;
use uuid::Uuid;

const JWT_SECRET: &str = "jwt-test";
const SECRET_KEY: &str = "key-test";

enum Script {
    Done(Vec<String>),
    Hang(String),
    Fail(String),
}

struct FakeAdapter {
    script: Script,
    calls: AtomicUsize,
}

impl FakeAdapter {
    fn new(script: Script) -> Self {
        Self {
            script,
            calls: AtomicUsize::new(0),
        }
    }
}

impl LlmAdapter for FakeAdapter {
    fn stream_chat(&self, _req: ChatRequest) -> BoxChatFuture {
        self.calls.fetch_add(1, Ordering::SeqCst);
        match &self.script {
            Script::Fail(m) => {
                let m = m.clone();
                Box::pin(async move { Err(AdapterError { message: m }) })
            }
            script => {
                let s: BoxEventStream = match script {
                    Script::Done(deltas) => {
                        let mut items: Vec<Result<AdapterEvent, AdapterError>> = deltas
                            .iter()
                            .map(|d| Ok(AdapterEvent::Delta(d.clone())))
                            .collect();
                        items.push(Ok(AdapterEvent::Done(Some(TokenUsage {
                            prompt: 1,
                            completion: 2,
                            total: 3,
                        }))));
                        Box::pin(stream::iter(items))
                    }
                    Script::Hang(d) => Box::pin(
                        stream::once({
                            let d = d.clone();
                            async move { Ok(AdapterEvent::Delta(d)) }
                        })
                        .chain(stream::pending()),
                    ),
                    Script::Fail(_) => unreachable!(),
                };
                Box::pin(async move { Ok(s) })
            }
        }
    }
}

struct Fixture {
    user_id: Uuid,
    token: String,
    provider_id: Uuid,
    conversation_id: Uuid,
}

async fn fixture(pool: &PgPool, name: &str) -> Fixture {
    let user = users::insert(pool, name, &format!("{name}@example.com"), "hash")
        .await
        .unwrap();
    let token = sign_access_token(JWT_SECRET, user.id).unwrap();
    let sealed = seal(&derive_key(SECRET_KEY), "sk-test").unwrap();
    let provider = providers::insert(
        pool,
        &NewProvider {
            id: Uuid::new_v4(),
            user_id: user.id,
            kind: "openai",
            name: "p",
            base_url: "https://api.example.com/v1",
            api_key: &sealed,
            api_path: "",
            models_path: "",
            models: &[],
        },
    )
    .await
    .unwrap();
    let conversation = lemma_conversations::store::insert(pool, user.id)
        .await
        .unwrap();
    Fixture {
        user_id: user.id,
        token,
        provider_id: provider.id,
        conversation_id: conversation.id,
    }
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

async fn send(
    svc: &ChatService,
    token: &str,
    conversation_id: Uuid,
    provider_id: Uuid,
    content: &str,
    client_msg_id: &str,
) -> Result<ServiceStream<SendMessageResponse>, ConnectError> {
    let msg = SendMessageRequest {
        conversation_id: conversation_id.to_string(),
        content: content.into(),
        provider_id: provider_id.to_string(),
        model: "gpt-x".into(),
        client_msg_id: client_msg_id.into(),
        ..Default::default()
    };
    let bytes = msg.encode_to_bytes();
    let view = SendMessageRequest::decode_view(&bytes).unwrap();
    match svc
        .send_message(bearer_ctx(token), ServiceRequest::from_parts(&view, &bytes))
        .await
    {
        Ok(resp) => Ok(resp.body.map(|item| item.map(|m| owned_body(&m))).boxed()),
        Err(e) => Err(e),
    }
}

async fn resume(
    svc: &ChatService,
    token: &str,
    message_id: &str,
    offset: i64,
) -> Result<ServiceStream<ResumeStreamResponse>, ConnectError> {
    let msg = ResumeStreamRequest {
        message_id: message_id.into(),
        offset,
        ..Default::default()
    };
    let bytes = msg.encode_to_bytes();
    let view = ResumeStreamRequest::decode_view(&bytes).unwrap();
    match svc
        .resume_stream(bearer_ctx(token), ServiceRequest::from_parts(&view, &bytes))
        .await
    {
        Ok(resp) => Ok(resp.body.map(|item| item.map(|m| owned_body(&m))).boxed()),
        Err(e) => Err(e),
    }
}

async fn abort(
    svc: &ChatService,
    token: &str,
    message_id: &str,
) -> Result<AbortMessageResponse, ConnectError> {
    let msg = AbortMessageRequest {
        message_id: message_id.into(),
        ..Default::default()
    };
    let bytes = msg.encode_to_bytes();
    let view = AbortMessageRequest::decode_view(&bytes).unwrap();
    match svc
        .abort_message(bearer_ctx(token), ServiceRequest::from_parts(&view, &bytes))
        .await
    {
        Ok(resp) => Ok(owned_body(&resp.body)),
        Err(e) => Err(e),
    }
}

async fn collect_send(stream: ServiceStream<SendMessageResponse>) -> Vec<SendMessageResponse> {
    stream.map(|r| r.unwrap()).collect().await
}

fn kind_of(ev: &ChatEvent) -> &Kind {
    ev.kind.as_ref().unwrap()
}

fn event_of(r: &SendMessageResponse) -> &ChatEvent {
    r.event.as_option().unwrap()
}

fn revent_of(r: &ResumeStreamResponse) -> &ChatEvent {
    r.event.as_option().unwrap()
}

async fn insert_orphan_streaming(pool: &PgPool, conv: Uuid, content: &str) -> Uuid {
    let id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO messages (id, conversation_id, role, content, status, seq)
         SELECT $1, $2, 'assistant', $3, 'streaming', coalesce(max(seq), 0) + 1
         FROM messages WHERE conversation_id = $2",
    )
    .bind(id)
    .bind(conv)
    .bind(content)
    .execute(pool)
    .await
    .unwrap();
    id
}

#[sqlx::test(migrations = "../lemma-db/migrations")]
async fn send_streams_deltas_and_finalizes(pool: PgPool) {
    let f = fixture(&pool, "alice").await;
    let svc = ChatService::new(
        pool.clone(),
        JWT_SECRET,
        SECRET_KEY,
        Arc::new(FakeAdapter::new(Script::Done(vec![
            "你".into(),
            "好".into(),
        ]))),
    );
    let stream = send(
        &svc,
        &f.token,
        f.conversation_id,
        f.provider_id,
        "你好",
        "c1",
    )
    .await
    .unwrap();
    let events = collect_send(stream).await;

    assert_eq!(events.len(), 4);
    let message_id = match kind_of(event_of(&events[0])) {
        Kind::Started(s) => {
            assert_eq!(s.client_msg_id, "c1");
            s.message_id.clone()
        }
        other => panic!("expected started, got {other:?}"),
    };
    assert!(matches!(kind_of(event_of(&events[1])), Kind::Delta(d) if d.content == "你"));
    assert!(matches!(kind_of(event_of(&events[2])), Kind::Delta(d) if d.content == "好"));
    assert!(matches!(
        kind_of(event_of(&events[3])),
        Kind::Done(d) if d.usage.as_option().unwrap().total_tokens == 3
    ));

    let mid = Uuid::parse_str(&message_id).unwrap();
    let msg = store::find_by_id_and_user(&pool, mid, f.user_id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(msg.status, "done");
    assert_eq!(msg.content, "你好");
    assert_eq!(msg.token_usage.unwrap().0.total, 3);
}

#[sqlx::test(migrations = "../lemma-db/migrations")]
async fn send_idempotent_replay_skips_upstream(pool: PgPool) {
    let f = fixture(&pool, "alice").await;
    let adapter = Arc::new(FakeAdapter::new(Script::Done(vec!["好".into()])));
    let svc = ChatService::new(pool.clone(), JWT_SECRET, SECRET_KEY, adapter.clone());

    let first = collect_send(
        send(
            &svc,
            &f.token,
            f.conversation_id,
            f.provider_id,
            "你好",
            "dup",
        )
        .await
        .unwrap(),
    )
    .await;
    let first_id = match kind_of(event_of(&first[0])) {
        Kind::Started(s) => s.message_id.clone(),
        other => panic!("expected started, got {other:?}"),
    };

    let second = collect_send(
        send(
            &svc,
            &f.token,
            f.conversation_id,
            f.provider_id,
            "你好",
            "dup",
        )
        .await
        .unwrap(),
    )
    .await;
    assert_eq!(adapter.calls.load(Ordering::SeqCst), 1);
    match kind_of(event_of(&second[0])) {
        Kind::Started(s) => assert_eq!(s.message_id, first_id),
        other => panic!("expected started, got {other:?}"),
    }
    assert!(matches!(kind_of(event_of(&second[1])), Kind::Delta(d) if d.content == "好"));
    assert!(matches!(kind_of(event_of(&second[2])), Kind::Done(_)));

    let count: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM messages WHERE conversation_id = $1 AND role = 'assistant'",
    )
    .bind(f.conversation_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(count, 1);
}

#[sqlx::test(migrations = "../lemma-db/migrations")]
async fn abort_mid_stream_keeps_partial(pool: PgPool) {
    let f = fixture(&pool, "alice").await;
    let svc = ChatService::new(
        pool.clone(),
        JWT_SECRET,
        SECRET_KEY,
        Arc::new(FakeAdapter::new(Script::Hang("半".into()))),
    );
    let mut stream = send(&svc, &f.token, f.conversation_id, f.provider_id, "你好", "")
        .await
        .unwrap();

    let started = stream.next().await.unwrap().unwrap();
    let message_id = match kind_of(event_of(&started)) {
        Kind::Started(s) => s.message_id.clone(),
        other => panic!("expected started, got {other:?}"),
    };
    let delta = stream.next().await.unwrap().unwrap();
    assert!(matches!(kind_of(event_of(&delta)), Kind::Delta(d) if d.content == "半"));

    abort(&svc, &f.token, &message_id).await.unwrap();

    let aborted = stream.next().await.unwrap().unwrap();
    assert!(matches!(kind_of(event_of(&aborted)), Kind::Aborted(_)));
    assert!(stream.next().await.is_none());

    let mid = Uuid::parse_str(&message_id).unwrap();
    let msg = store::find_by_id_and_user(&pool, mid, f.user_id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(msg.status, "aborted");
    assert_eq!(msg.content, "半");

    abort(&svc, &f.token, &message_id).await.unwrap();
}

#[sqlx::test(migrations = "../lemma-db/migrations")]
async fn resume_replays_from_char_offset(pool: PgPool) {
    let f = fixture(&pool, "alice").await;
    let svc = ChatService::new(
        pool.clone(),
        JWT_SECRET,
        SECRET_KEY,
        Arc::new(FakeAdapter::new(Script::Done(vec!["你好".into()]))),
    );
    let first = collect_send(
        send(&svc, &f.token, f.conversation_id, f.provider_id, "hi", "")
            .await
            .unwrap(),
    )
    .await;
    let message_id = match kind_of(event_of(&first[0])) {
        Kind::Started(s) => s.message_id.clone(),
        other => panic!("expected started, got {other:?}"),
    };

    let events: Vec<ResumeStreamResponse> = resume(&svc, &f.token, &message_id, 1)
        .await
        .unwrap()
        .map(|r| r.unwrap())
        .collect()
        .await;
    assert_eq!(events.len(), 2);
    assert!(matches!(kind_of(revent_of(&events[0])), Kind::Delta(d) if d.content == "好"));
    assert!(matches!(kind_of(revent_of(&events[1])), Kind::Done(_)));
}

#[sqlx::test(migrations = "../lemma-db/migrations")]
async fn send_adapter_failure_marks_error(pool: PgPool) {
    let f = fixture(&pool, "alice").await;
    let svc = ChatService::new(
        pool.clone(),
        JWT_SECRET,
        SECRET_KEY,
        Arc::new(FakeAdapter::new(Script::Fail("connection refused".into()))),
    );
    let events = collect_send(
        send(&svc, &f.token, f.conversation_id, f.provider_id, "hi", "")
            .await
            .unwrap(),
    )
    .await;
    assert_eq!(events.len(), 2);
    let message_id = match kind_of(event_of(&events[0])) {
        Kind::Started(s) => s.message_id.clone(),
        other => panic!("expected started, got {other:?}"),
    };
    assert!(matches!(
        kind_of(event_of(&events[1])),
        Kind::Error(e) if e.message.contains("connection refused")
    ));

    let mid = Uuid::parse_str(&message_id).unwrap();
    let msg = store::find_by_id_and_user(&pool, mid, f.user_id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(msg.status, "error");
}

#[sqlx::test(migrations = "../lemma-db/migrations")]
async fn send_rejects_foreign_conversation(pool: PgPool) {
    let f = fixture(&pool, "alice").await;
    let other = fixture(&pool, "erin").await;
    let svc = ChatService::new(
        pool.clone(),
        JWT_SECRET,
        SECRET_KEY,
        Arc::new(FakeAdapter::new(Script::Done(vec![]))),
    );
    let err = send(
        &svc,
        &f.token,
        other.conversation_id,
        f.provider_id,
        "hi",
        "",
    )
    .await
    .err()
    .unwrap();
    assert_eq!(err.code, ErrorCode::NotFound);
}

#[sqlx::test(migrations = "../lemma-db/migrations")]
async fn resume_live_stream_replays_snapshot_then_follows(pool: PgPool) {
    let f = fixture(&pool, "alice").await;
    let svc = ChatService::new(
        pool.clone(),
        JWT_SECRET,
        SECRET_KEY,
        Arc::new(FakeAdapter::new(Script::Hang("半".into()))),
    );
    let mut stream = send(&svc, &f.token, f.conversation_id, f.provider_id, "hi", "")
        .await
        .unwrap();
    let started = stream.next().await.unwrap().unwrap();
    let message_id = match kind_of(event_of(&started)) {
        Kind::Started(s) => s.message_id.clone(),
        other => panic!("expected started, got {other:?}"),
    };
    let delta = stream.next().await.unwrap().unwrap();
    assert!(matches!(kind_of(event_of(&delta)), Kind::Delta(d) if d.content == "半"));

    let mut resumed = resume(&svc, &f.token, &message_id, 0).await.unwrap();
    let replay = tokio::time::timeout(std::time::Duration::from_secs(5), resumed.next())
        .await
        .unwrap()
        .unwrap()
        .unwrap();
    assert!(matches!(kind_of(revent_of(&replay)), Kind::Delta(d) if d.content == "半"));

    abort(&svc, &f.token, &message_id).await.unwrap();
    let aborted = tokio::time::timeout(std::time::Duration::from_secs(5), resumed.next())
        .await
        .unwrap()
        .unwrap()
        .unwrap();
    assert!(matches!(kind_of(revent_of(&aborted)), Kind::Aborted(_)));
}

#[sqlx::test(migrations = "../lemma-db/migrations")]
async fn resume_after_abort_replays_from_db(pool: PgPool) {
    let f = fixture(&pool, "alice").await;
    let svc = ChatService::new(
        pool.clone(),
        JWT_SECRET,
        SECRET_KEY,
        Arc::new(FakeAdapter::new(Script::Hang("半".into()))),
    );
    let mut stream = send(&svc, &f.token, f.conversation_id, f.provider_id, "hi", "")
        .await
        .unwrap();
    let started = stream.next().await.unwrap().unwrap();
    let message_id = match kind_of(event_of(&started)) {
        Kind::Started(s) => s.message_id.clone(),
        other => panic!("expected started, got {other:?}"),
    };
    stream.next().await.unwrap().unwrap();
    abort(&svc, &f.token, &message_id).await.unwrap();

    let events: Vec<ResumeStreamResponse> = resume(&svc, &f.token, &message_id, 0)
        .await
        .unwrap()
        .map(|r| r.unwrap())
        .collect()
        .await;
    assert_eq!(events.len(), 2);
    assert!(matches!(kind_of(revent_of(&events[0])), Kind::Delta(d) if d.content == "半"));
    assert!(matches!(kind_of(revent_of(&events[1])), Kind::Aborted(_)));
}

#[sqlx::test(migrations = "../lemma-db/migrations")]
async fn resume_orphan_streaming_marks_aborted(pool: PgPool) {
    let f = fixture(&pool, "alice").await;
    let svc = ChatService::new(
        pool.clone(),
        JWT_SECRET,
        SECRET_KEY,
        Arc::new(FakeAdapter::new(Script::Done(vec![]))),
    );
    let orphan = insert_orphan_streaming(&pool, f.conversation_id, "遗").await;

    let events: Vec<ResumeStreamResponse> = resume(&svc, &f.token, &orphan.to_string(), 0)
        .await
        .unwrap()
        .map(|r| r.unwrap())
        .collect()
        .await;
    assert_eq!(events.len(), 2);
    assert!(matches!(kind_of(revent_of(&events[0])), Kind::Delta(d) if d.content == "遗"));
    assert!(matches!(kind_of(revent_of(&events[1])), Kind::Aborted(_)));

    let status: String = sqlx::query_scalar("SELECT status FROM messages WHERE id = $1")
        .bind(orphan)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(status, "aborted");
}

#[sqlx::test(migrations = "../lemma-db/migrations")]
async fn send_rejects_bad_requests(pool: PgPool) {
    let f = fixture(&pool, "alice").await;
    let svc = ChatService::new(
        pool.clone(),
        JWT_SECRET,
        SECRET_KEY,
        Arc::new(FakeAdapter::new(Script::Done(vec![]))),
    );

    async fn raw_send(
        svc: &ChatService,
        token: &str,
        f: &Fixture,
        content: &str,
        model: &str,
    ) -> Result<ServiceStream<SendMessageResponse>, ConnectError> {
        let msg = SendMessageRequest {
            conversation_id: f.conversation_id.to_string(),
            content: content.into(),
            provider_id: f.provider_id.to_string(),
            model: model.into(),
            ..Default::default()
        };
        let bytes = msg.encode_to_bytes();
        let view = SendMessageRequest::decode_view(&bytes).unwrap();
        match svc
            .send_message(bearer_ctx(token), ServiceRequest::from_parts(&view, &bytes))
            .await
        {
            Ok(resp) => Ok(resp.body.map(|item| item.map(|m| owned_body(&m))).boxed()),
            Err(e) => Err(e),
        }
    }

    let err = raw_send(&svc, &f.token, &f, "  ", "gpt-x")
        .await
        .err()
        .unwrap();
    assert_eq!(err.code, ErrorCode::InvalidArgument);

    let err = raw_send(&svc, &f.token, &f, "hi", "").await.err().unwrap();
    assert_eq!(err.code, ErrorCode::InvalidArgument);

    sqlx::query("UPDATE providers SET enabled = false WHERE id = $1")
        .bind(f.provider_id)
        .execute(&pool)
        .await
        .unwrap();
    let err = raw_send(&svc, &f.token, &f, "hi", "gpt-x")
        .await
        .err()
        .unwrap();
    assert_eq!(err.code, ErrorCode::InvalidArgument);
}
