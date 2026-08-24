use std::sync::Arc;
use std::time::{Duration, Instant};

use buffa::MessageField;
use connectrpc::{
    ConnectError, RequestContext, Response, ServiceRequest, ServiceResult, ServiceStream,
};
use futures::{StreamExt, stream};
use lemma_auth::require_user;
use lemma_db::entity::{Message as DbMessage, TokenUsage as DbTokenUsage};
use lemma_proto::lemma::v1::{
    AbortMessageResponse, ChatAborted, ChatDelta, ChatDone, ChatError, ChatEvent, ChatStarted,
    ResumeStreamResponse, SendMessageResponse, TokenUsage,
};
use sqlx::PgPool;
use tokio_stream::wrappers::BroadcastStream;
use uuid::Uuid;

use crate::adapter::{AdapterEvent, BoxEventStream, ChatMessage, ChatRequest, LlmAdapter};
use crate::registry::{StreamEvent, StreamHandle, StreamRegistry, StreamStatus};
use crate::store;

// 落库节流：间隔或增量字节先到先触发
const FLUSH_INTERVAL: Duration = Duration::from_millis(500);
const FLUSH_BYTES: usize = 2048;

pub struct ChatService {
    pool: PgPool,
    jwt_secret: String,
    secret_key: String,
    adapter: Arc<dyn LlmAdapter>,
    registry: StreamRegistry,
}

impl ChatService {
    pub fn new(
        pool: PgPool,
        jwt_secret: impl Into<String>,
        secret_key: impl Into<String>,
        adapter: Arc<dyn LlmAdapter>,
    ) -> Self {
        Self {
            pool,
            jwt_secret: jwt_secret.into(),
            secret_key: secret_key.into(),
            adapter,
            registry: StreamRegistry::new(),
        }
    }

    // 幂等重放 / 断线续传共用：按字符 offset 组装事件流
    async fn chat_event_stream(
        &self,
        mut msg: DbMessage,
        user_id: Uuid,
        offset: usize,
    ) -> Result<ServiceStream<ChatEvent>, ConnectError> {
        if msg.status == "streaming" {
            match self.registry.get(&msg.id) {
                Some(handle) => {
                    let (replay, rx) = handle.snapshot_and_subscribe(offset);
                    if handle.status() == StreamStatus::Live {
                        // 进行中：补差额 + 挂广播续播
                        let prefix: Vec<Result<ChatEvent, ConnectError>> = if replay.is_empty() {
                            Vec::new()
                        } else {
                            vec![Ok(delta_event(replay))]
                        };
                        return Ok(Box::pin(stream::iter(prefix).chain(live_event_stream(rx))));
                    }
                    // drive 先落库再置终态；重读库拿最终内容
                    msg = store::find_by_id_and_user(&self.pool, msg.id, user_id)
                        .await
                        .map_err(map_db)?
                        .ok_or_else(|| ConnectError::internal("message vanished"))?;
                }
                None => {
                    // 孤儿 streaming（服务重启）：按中断收尾
                    msg = store::mark_aborted(&self.pool, msg.id, &msg.content)
                        .await
                        .map_err(map_db)?
                        .ok_or_else(|| ConnectError::internal("message vanished"))?;
                }
            }
        }
        Ok(Box::pin(stream::iter(
            replay_events(&msg, offset)
                .into_iter()
                .map(Ok::<ChatEvent, ConnectError>),
        )))
    }
}

#[allow(refining_impl_trait)]
impl lemma_proto::lemma::v1::ChatService for ChatService {
    async fn send_message(
        &self,
        ctx: RequestContext,
        request: ServiceRequest<'_, lemma_proto::lemma::v1::SendMessageRequest>,
    ) -> ServiceResult<ServiceStream<SendMessageResponse>> {
        let user_id = require_user(&self.jwt_secret, &ctx)?;
        let conversation_id = parse_id(request.conversation_id)?;
        let provider_id = parse_id(request.provider_id)?;
        if request.content.trim().is_empty() {
            return Err(ConnectError::invalid_argument("content required"));
        }
        if request.model.is_empty() {
            return Err(ConnectError::invalid_argument("model required"));
        }
        let client_msg_id = request.client_msg_id;

        // 归属校验
        lemma_conversations::store::find_by_id_and_user(&self.pool, conversation_id, user_id)
            .await
            .map_err(map_db)?
            .ok_or_else(|| ConnectError::not_found("conversation not found"))?;
        let provider =
            lemma_providers::providers::find_by_id_and_user(&self.pool, provider_id, user_id)
                .await
                .map_err(map_db)?
                .ok_or_else(|| ConnectError::not_found("provider not found"))?;
        if !provider.enabled {
            return Err(ConnectError::invalid_argument("provider disabled"));
        }

        // 幂等：同一 client_msg_id 重发 → 重放已有 assistant 消息
        if !client_msg_id.is_empty()
            && let Some(existing) =
                store::find_assistant_by_client_msg_id(&self.pool, conversation_id, client_msg_id)
                    .await
                    .map_err(map_db)?
        {
            let started = started_response(existing.id, client_msg_id);
            let events = self.chat_event_stream(existing, user_id, 0).await?;
            return Response::stream_ok(stream::once(async { Ok(started) }).chain(events.map(
                |r| {
                    r.map(|e| SendMessageResponse {
                        event: e.into(),
                        ..Default::default()
                    })
                },
            )));
        }

        // 解密 api key
        let key = lemma_providers::derive_key(&self.secret_key);
        let api_key = lemma_providers::open(&key, &provider.api_key)
            .map_err(|_| ConnectError::internal("decrypt api key"))?;

        // 事务：user 消息 + assistant 占位
        let mut tx = self.pool.begin().await.map_err(map_db)?;
        store::lock_conversation(&mut *tx, conversation_id)
            .await
            .map_err(map_db)?;
        store::insert_user_message(&mut *tx, conversation_id, request.content)
            .await
            .map_err(map_db)?;
        let client_msg_id_opt = if client_msg_id.is_empty() {
            None
        } else {
            Some(client_msg_id)
        };
        let assistant = match store::insert_assistant_placeholder(
            &mut *tx,
            conversation_id,
            provider_id,
            request.model,
            client_msg_id_opt,
        )
        .await
        {
            Ok(m) => m,
            // 并发双发撞唯一索引：回滚后按幂等重放处理
            Err(e) if is_unique_violation(&e) && !client_msg_id.is_empty() => {
                drop(tx);
                let existing = store::find_assistant_by_client_msg_id(
                    &self.pool,
                    conversation_id,
                    client_msg_id,
                )
                .await
                .map_err(map_db)?
                .ok_or_else(|| ConnectError::internal("idempotent lookup failed"))?;
                let started = started_response(existing.id, client_msg_id);
                let events = self.chat_event_stream(existing, user_id, 0).await?;
                return Response::stream_ok(stream::once(async { Ok(started) }).chain(events.map(
                    |r| {
                        r.map(|e| SendMessageResponse {
                            event: e.into(),
                            ..Default::default()
                        })
                    },
                )));
            }
            Err(e) => return Err(map_db(e)),
        };
        tx.commit().await.map_err(map_db)?;

        // 上下文：含刚插入的 user 消息，排除 streaming 占位
        let history = store::list_context(&self.pool, conversation_id)
            .await
            .map_err(map_db)?;
        let messages = history
            .iter()
            .map(|m| ChatMessage {
                role: m.role.clone(),
                content: m.content.clone(),
            })
            .collect();
        let chat_req = ChatRequest {
            kind: lemma_providers::kind_to_proto(&provider.kind),
            base_url: provider.base_url.clone(),
            api_path: provider.api_path.clone(),
            api_key,
            model: request.model.to_owned(),
            messages,
        };
        let started = started_response(assistant.id, client_msg_id);

        // 建连失败：落 error + 返回 [started, error] 两事件流
        let upstream = match self.adapter.stream_chat(chat_req).await {
            Ok(s) => s,
            Err(e) => {
                let _ = store::mark_error(&self.pool, assistant.id, "").await;
                return Response::stream_ok(stream::iter(vec![
                    Ok::<SendMessageResponse, ConnectError>(started),
                    Ok(SendMessageResponse {
                        event: error_event(&e.message).into(),
                        ..Default::default()
                    }),
                ]));
            }
        };

        let handle = self.registry.register(assistant.id);
        let (_replay, rx) = handle.snapshot_and_subscribe(0);
        tokio::spawn(drive(
            self.pool.clone(),
            self.registry.clone(),
            Arc::clone(&handle),
            assistant.id,
            upstream,
        ));
        // 自己这份先释放：producer 终结后频道关闭，响应流自然收尾
        drop(handle);

        let events = live_event_stream(rx).map(|r| {
            r.map(|e| SendMessageResponse {
                event: e.into(),
                ..Default::default()
            })
        });
        Response::stream_ok(stream::once(async { Ok(started) }).chain(events))
    }

    async fn abort_message(
        &self,
        ctx: RequestContext,
        request: ServiceRequest<'_, lemma_proto::lemma::v1::AbortMessageRequest>,
    ) -> ServiceResult<AbortMessageResponse> {
        let user_id = require_user(&self.jwt_secret, &ctx)?;
        let id = parse_id(request.message_id)?;
        let msg = store::find_by_id_and_user(&self.pool, id, user_id)
            .await
            .map_err(map_db)?
            .ok_or_else(|| ConnectError::not_found("message not found"))?;
        // 幂等：非 streaming 无需处理
        if msg.status != "streaming" {
            return Response::ok(AbortMessageResponse::default());
        }
        match self.registry.get(&id) {
            // producer 监听中断信号，负责落库与广播
            Some(handle) => {
                handle.abort();
            }
            // 孤儿 streaming（服务重启）：直接落库
            None => {
                store::mark_aborted(&self.pool, id, &msg.content)
                    .await
                    .map_err(map_db)?;
            }
        }
        Response::ok(AbortMessageResponse::default())
    }

    async fn resume_stream(
        &self,
        ctx: RequestContext,
        request: ServiceRequest<'_, lemma_proto::lemma::v1::ResumeStreamRequest>,
    ) -> ServiceResult<ServiceStream<ResumeStreamResponse>> {
        let user_id = require_user(&self.jwt_secret, &ctx)?;
        let message_id = parse_id(request.message_id)?;
        let offset = usize::try_from(request.offset.max(0)).unwrap_or(usize::MAX);
        let msg = store::find_by_id_and_user(&self.pool, message_id, user_id)
            .await
            .map_err(map_db)?
            .ok_or_else(|| ConnectError::not_found("message not found"))?;
        if msg.role != "assistant" {
            return Err(ConnectError::invalid_argument("not an assistant message"));
        }
        let events = self.chat_event_stream(msg, user_id, offset).await?;
        Response::stream_ok(events.map(|r| {
            r.map(|e| ResumeStreamResponse {
                event: e.into(),
                ..Default::default()
            })
        }))
    }
}

// 生产者：消费上游流推 registry，节流落库；终结落库后移除注册表项
async fn drive(
    pool: PgPool,
    registry: StreamRegistry,
    handle: Arc<StreamHandle>,
    message_id: Uuid,
    mut upstream: BoxEventStream,
) {
    let mut pending_bytes = 0usize;
    let mut last_flush = Instant::now();
    loop {
        tokio::select! {
            _ = handle.aborted() => {
                let content = handle.content();
                let _ = store::mark_aborted(&pool, message_id, &content).await;
                handle.mark_aborted();
                break;
            }
            item = upstream.next() => match item {
                Some(Ok(AdapterEvent::Delta(d))) => {
                    pending_bytes += d.len();
                    handle.push_delta(&d);
                    if pending_bytes >= FLUSH_BYTES || last_flush.elapsed() >= FLUSH_INTERVAL {
                        let content = handle.content();
                        if store::flush_content(&pool, message_id, &content).await.is_ok() {
                            pending_bytes = 0;
                            last_flush = Instant::now();
                        }
                    }
                }
                Some(Ok(AdapterEvent::Done(usage))) => {
                    let content = handle.content();
                    let _ = store::finalize(&pool, message_id, &content, usage.clone()).await;
                    handle.finish(usage);
                    break;
                }
                Some(Err(e)) => {
                    let content = handle.content();
                    let _ = store::mark_error(&pool, message_id, &content).await;
                    handle.fail(&e.message);
                    break;
                }
                // 上游静默结束按 done 处理（usage 未知）
                None => {
                    let content = handle.content();
                    let _ = store::finalize(&pool, message_id, &content, None).await;
                    handle.finish(None);
                    break;
                }
            },
        }
    }
    registry.remove(&message_id);
}

fn live_event_stream(
    rx: tokio::sync::broadcast::Receiver<StreamEvent>,
) -> ServiceStream<ChatEvent> {
    Box::pin(BroadcastStream::new(rx).map(|item| match item {
        Ok(ev) => Ok(stream_event_to_chat_event(ev)),
        // 客户端消费太慢溢出广播缓冲：本 RPC 报错，流本身不受影响
        Err(_) => Err(ConnectError::internal("stream lagged")),
    }))
}

fn replay_events(msg: &DbMessage, offset: usize) -> Vec<ChatEvent> {
    let mut out = Vec::new();
    let replay: String = msg.content.chars().skip(offset).collect();
    if !replay.is_empty() {
        out.push(delta_event(replay));
    }
    match msg.status.as_str() {
        "aborted" => out.push(aborted_event()),
        // 原始错误消息没落库，用通用文案
        "error" => out.push(error_event("generation failed")),
        _ => out.push(done_event(msg.token_usage.as_ref().map(|u| u.0.clone()))),
    }
    out
}

fn stream_event_to_chat_event(e: StreamEvent) -> ChatEvent {
    match e {
        StreamEvent::Delta(c) => delta_event(c),
        StreamEvent::Done(u) => done_event(u),
        StreamEvent::Aborted => aborted_event(),
        StreamEvent::Failed(m) => error_event(&m),
    }
}

fn started_response(message_id: Uuid, client_msg_id: &str) -> SendMessageResponse {
    SendMessageResponse {
        event: ChatEvent {
            kind: ChatStarted {
                message_id: message_id.to_string(),
                client_msg_id: client_msg_id.to_owned(),
                ..Default::default()
            }
            .into(),
            ..Default::default()
        }
        .into(),
        ..Default::default()
    }
}

fn delta_event(content: String) -> ChatEvent {
    ChatEvent {
        kind: ChatDelta {
            content,
            ..Default::default()
        }
        .into(),
        ..Default::default()
    }
}

fn done_event(usage: Option<DbTokenUsage>) -> ChatEvent {
    let usage = match usage {
        Some(u) => MessageField::some(TokenUsage {
            prompt_tokens: i32::try_from(u.prompt).unwrap_or(i32::MAX),
            completion_tokens: i32::try_from(u.completion).unwrap_or(i32::MAX),
            total_tokens: i32::try_from(u.total).unwrap_or(i32::MAX),
            ..Default::default()
        }),
        None => MessageField::none(),
    };
    ChatEvent {
        kind: ChatDone {
            usage,
            ..Default::default()
        }
        .into(),
        ..Default::default()
    }
}

fn aborted_event() -> ChatEvent {
    ChatEvent {
        kind: ChatAborted::default().into(),
        ..Default::default()
    }
}

fn error_event(message: &str) -> ChatEvent {
    ChatEvent {
        kind: ChatError {
            message: message.to_owned(),
            ..Default::default()
        }
        .into(),
        ..Default::default()
    }
}

fn parse_id(id: &str) -> Result<Uuid, ConnectError> {
    Uuid::parse_str(id).map_err(|_| ConnectError::invalid_argument("invalid id"))
}

fn map_db(e: sqlx::Error) -> ConnectError {
    ConnectError::internal(format!("db: {e}"))
}

fn is_unique_violation(e: &sqlx::Error) -> bool {
    matches!(e, sqlx::Error::Database(dbe) if dbe.code().as_deref() == Some("23505"))
}
