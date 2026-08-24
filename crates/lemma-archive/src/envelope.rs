use chrono::{DateTime, Utc};
use lemma_db::entity::{Message, TokenUsage};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::ArchiveError;

/// 归档对象信封：version 字段留向前兼容余地
#[derive(Serialize, Deserialize)]
pub struct ArchiveEnvelope {
    pub version: u32,
    pub conversation_id: String,
    pub archived_at: DateTime<Utc>,
    pub messages: Vec<ArchivedMessage>,
}

/// 消息的纯数据快照（id 用 String：信封不依赖 uuid 的 serde feature）
#[derive(Serialize, Deserialize)]
pub struct ArchivedMessage {
    pub id: String,
    pub role: String,
    pub content: String,
    pub provider_id: Option<String>,
    pub model: Option<String>,
    pub client_msg_id: Option<String>,
    pub status: String,
    pub token_usage: Option<TokenUsage>,
    pub seq: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub fn envelope_from_messages(
    conversation_id: Uuid,
    archived_at: DateTime<Utc>,
    messages: &[Message],
) -> ArchiveEnvelope {
    ArchiveEnvelope {
        version: 1,
        conversation_id: conversation_id.to_string(),
        archived_at,
        messages: messages
            .iter()
            .map(|m| ArchivedMessage {
                id: m.id.to_string(),
                role: m.role.clone(),
                content: m.content.clone(),
                provider_id: m.provider_id.map(|p| p.to_string()),
                model: m.model.clone(),
                client_msg_id: m.client_msg_id.clone(),
                status: m.status.clone(),
                token_usage: m.token_usage.clone().map(|j| j.0),
                seq: m.seq,
                created_at: m.created_at,
                updated_at: m.updated_at,
            })
            .collect(),
    }
}

pub fn serialize_envelope(envelope: &ArchiveEnvelope) -> Result<Vec<u8>, ArchiveError> {
    serde_json::to_vec(envelope).map_err(|e| ArchiveError(format!("serialize: {e}")))
}

pub fn deserialize_envelope(bytes: &[u8]) -> Result<ArchiveEnvelope, ArchiveError> {
    serde_json::from_slice(bytes).map_err(|e| ArchiveError(format!("deserialize: {e}")))
}

/// 信封还原为 DB 实体；sync_seq 填 0（回灌 INSERT 走列默认取新号）
pub fn messages_from_envelope(envelope: &ArchiveEnvelope) -> Result<Vec<Message>, ArchiveError> {
    envelope
        .messages
        .iter()
        .map(|m| {
            Ok(Message {
                id: Uuid::parse_str(&m.id)
                    .map_err(|e| ArchiveError(format!("bad message id: {e}")))?,
                conversation_id: Uuid::parse_str(&envelope.conversation_id)
                    .map_err(|e| ArchiveError(format!("bad conversation id: {e}")))?,
                role: m.role.clone(),
                content: m.content.clone(),
                provider_id: m
                    .provider_id
                    .as_deref()
                    .and_then(|p| Uuid::parse_str(p).ok()),
                model: m.model.clone(),
                client_msg_id: m.client_msg_id.clone(),
                status: m.status.clone(),
                token_usage: m.token_usage.clone().map(sqlx::types::Json),
                seq: m.seq,
                sync_seq: 0,
                created_at: m.created_at,
                updated_at: m.updated_at,
            })
        })
        .collect()
}
