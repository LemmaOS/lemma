//! The JSON envelope format for archived conversations.

use chrono::{DateTime, Utc};
use lemma_db::entity::{Message, TokenUsage};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::ArchiveError;

/// Top-level archive object.
#[derive(Serialize, Deserialize)]
pub struct ArchiveEnvelope {
    /// Envelope schema version; currently 1.
    pub version: u32,
    pub conversation_id: String,
    pub archived_at: DateTime<Utc>,
    pub messages: Vec<ArchivedMessage>,
}

/// A message as stored in the archive. Ids are strings because the
/// envelope is a portable JSON document, not a database row.
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

/// Builds a version-1 envelope from a conversation's messages.
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

/// Serializes an envelope to JSON bytes.
pub fn serialize_envelope(envelope: &ArchiveEnvelope) -> Result<Vec<u8>, ArchiveError> {
    serde_json::to_vec(envelope).map_err(|e| ArchiveError(format!("serialize: {e}")))
}

/// Parses an envelope from JSON bytes.
pub fn deserialize_envelope(bytes: &[u8]) -> Result<ArchiveEnvelope, ArchiveError> {
    serde_json::from_slice(bytes).map_err(|e| ArchiveError(format!("deserialize: {e}")))
}

/// Converts an envelope back into message rows. The `sync_seq` field is a
/// zero placeholder; reinsertion draws a fresh sequence value.
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
