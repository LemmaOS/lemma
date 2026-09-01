//! Handler for the SyncService RPCs.

use std::time::{Duration, Instant};

use buffa::MessageField;
use buffa_types::google::protobuf::Timestamp;
use connectrpc::{
    ConnectError, RequestContext, Response, ServiceRequest, ServiceResult, ServiceStream,
};
use futures::stream;
use lemma_auth::require_user;
use lemma_db::entity::{Conversation as DbConversation, Message as DbMessage};
use lemma_proto::lemma::v1::{
    Conversation, ConversationStatus, Message, MessageStatus, PullResponse, SyncConversation,
    SyncMessage, WatchHeartbeat, WatchHint, WatchResponse,
};
use sqlx::PgPool;

use crate::store;

const PAGE_LIMIT: i64 = 500;
const POLL_INTERVAL: Duration = Duration::from_secs(3);
const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(15);

/// Connect handler implementing the SyncService RPCs.
pub struct SyncService {
    pool: PgPool,
    jwt_secret: String,
}

impl SyncService {
    pub fn new(pool: PgPool, jwt_secret: impl Into<String>) -> Self {
        Self {
            pool,
            jwt_secret: jwt_secret.into(),
        }
    }
}

#[allow(refining_impl_trait)]
impl lemma_proto::lemma::v1::SyncService for SyncService {
    async fn pull(
        &self,
        ctx: RequestContext,
        request: ServiceRequest<'_, lemma_proto::lemma::v1::PullRequest>,
    ) -> ServiceResult<PullResponse> {
        let user_id = require_user(&self.jwt_secret, &ctx)?;
        let after = request.after.max(0);

        let mut convs = store::pull_conversations(&self.pool, user_id, after, PAGE_LIMIT + 1)
            .await
            .map_err(map_db)?;
        let mut msgs = store::pull_messages(&self.pool, user_id, after, PAGE_LIMIT + 1)
            .await
            .map_err(map_db)?;
        let conv_trunc = convs.len() as i64 > PAGE_LIMIT;
        let msg_trunc = msgs.len() as i64 > PAGE_LIMIT;
        if conv_trunc {
            convs.pop();
        }
        if msg_trunc {
            msgs.pop();
        }

        let (next_after, has_more) = if conv_trunc || msg_trunc {
            // One page covers both lists. When either side overflows,
            // cut both at the smaller last sync_seq so the next pull
            // resumes at a consistent boundary for conversations and
            // messages alike.
            let boundary = match (conv_trunc, msg_trunc) {
                (true, true) => match (convs.last(), msgs.last()) {
                    (Some(c), Some(m)) => c.sync_seq.min(m.sync_seq),
                    _ => return Err(ConnectError::internal("page boundary")),
                },
                (true, false) => convs
                    .last()
                    .map(|c| c.sync_seq)
                    .ok_or_else(|| ConnectError::internal("page boundary"))?,
                (false, true) => msgs
                    .last()
                    .map(|m| m.sync_seq)
                    .ok_or_else(|| ConnectError::internal("page boundary"))?,
                (false, false) => unreachable!(),
            };
            convs.retain(|c| c.sync_seq <= boundary);
            msgs.retain(|m| m.sync_seq <= boundary);
            (boundary, true)
        } else {
            let max_seq = convs
                .last()
                .map(|c| c.sync_seq)
                .into_iter()
                .chain(msgs.last().map(|m| m.sync_seq))
                .max()
                .unwrap_or(after);
            (max_seq, false)
        };

        let archived = lemma_conversations::store::list_archived_by_user(&self.pool, user_id)
            .await
            .map_err(map_db)?;

        let active = lemma_conversations::store::list_active_by_user(&self.pool, user_id)
            .await
            .map_err(map_db)?;

        Response::ok(PullResponse {
            conversations: convs
                .iter()
                .map(|c| SyncConversation {
                    conversation: conversation_to_proto(c).into(),
                    sync_seq: c.sync_seq,
                    ..Default::default()
                })
                .collect(),
            messages: msgs
                .iter()
                .map(|m| SyncMessage {
                    message: message_to_proto(m).into(),
                    sync_seq: m.sync_seq,
                    ..Default::default()
                })
                .collect(),
            archived: archived.iter().map(conversation_to_proto).collect(),
            active: active.iter().map(conversation_to_proto).collect(),
            next_after,
            has_more,
            ..Default::default()
        })
    }

    async fn watch(
        &self,
        ctx: RequestContext,
        _request: ServiceRequest<'_, lemma_proto::lemma::v1::WatchRequest>,
    ) -> ServiceResult<ServiceStream<WatchResponse>> {
        require_user(&self.jwt_secret, &ctx)?;
        let pool = self.pool.clone();
        // Polls the global sequence head: any user's write can produce a
        // hint, so a hint only means "pull to find out". Heartbeats keep
        // proxies from killing the idle stream.
        let s = stream::unfold(
            (pool, 0i64, Instant::now()),
            |(pool, last_head, mut last_beat)| async move {
                loop {
                    tokio::time::sleep(POLL_INTERVAL).await;
                    let head = match store::head_sync_seq(&pool).await {
                        Ok(h) => h,
                        Err(_) => continue,
                    };
                    if head > last_head {
                        return Some((Ok(hint_response(head)), (pool, head, last_beat)));
                    }
                    if last_beat.elapsed() >= HEARTBEAT_INTERVAL {
                        last_beat = Instant::now();
                        return Some((Ok(heartbeat_response()), (pool, last_head, last_beat)));
                    }
                }
            },
        );
        Response::stream_ok(s)
    }
}

fn hint_response(head: i64) -> WatchResponse {
    WatchResponse {
        kind: WatchHint {
            sync_seq: head,
            ..Default::default()
        }
        .into(),
        ..Default::default()
    }
}

fn heartbeat_response() -> WatchResponse {
    WatchResponse {
        kind: WatchHeartbeat::default().into(),
        ..Default::default()
    }
}

fn conversation_to_proto(c: &DbConversation) -> Conversation {
    Conversation {
        id: c.id.to_string(),
        title: c.title.clone(),
        status: match c.status.as_str() {
            "archived" => ConversationStatus::Archived,
            _ => ConversationStatus::Active,
        }
        .into(),
        archived_at: match c.archived_at {
            Some(t) => MessageField::some(Timestamp::from(t)),
            None => MessageField::none(),
        },
        message_count: c.message_count.unwrap_or(0),
        created_at: Timestamp::from(c.created_at).into(),
        updated_at: Timestamp::from(c.updated_at).into(),
        ..Default::default()
    }
}

fn message_to_proto(m: &DbMessage) -> Message {
    Message {
        id: m.id.to_string(),
        conversation_id: m.conversation_id.to_string(),
        role: m.role.clone(),
        content: m.content.clone(),
        provider_id: m.provider_id.map(|p| p.to_string()).unwrap_or_default(),
        model: m.model.clone().unwrap_or_default(),
        status: match m.status.as_str() {
            "streaming" => MessageStatus::Streaming,
            "aborted" => MessageStatus::Aborted,
            "error" => MessageStatus::Error,
            _ => MessageStatus::Done,
        }
        .into(),
        created_at: Timestamp::from(m.created_at).into(),
        updated_at: Timestamp::from(m.updated_at).into(),
        seq: m.seq,
        ..Default::default()
    }
}

fn map_db(e: sqlx::Error) -> ConnectError {
    ConnectError::internal(format!("db: {e}"))
}
