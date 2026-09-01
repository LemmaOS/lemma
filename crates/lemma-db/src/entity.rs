//! Row types for the shared tables, mapped with `sqlx::FromRow`.
//!
//! Field docs are limited to columns whose semantics are not obvious from
//! the name: sealed credentials, the sync sequence, and lifecycle pointers.

use chrono::{DateTime, Utc};
use sqlx::types::Json;
use uuid::Uuid;

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub password_hash: String,
    pub role: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct RefreshToken {
    pub id: Uuid,
    pub user_id: Uuid,
    /// SHA-256 of the token; the plaintext token is never stored.
    pub token_hash: String,
    pub label: Option<String>,
    /// Points at the token that replaced this one during rotation.
    pub replaced_by: Option<Uuid>,
    /// Non-null once the token has been revoked.
    pub revoked_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct Provider {
    pub id: Uuid,
    pub user_id: Uuid,
    pub kind: String,
    pub name: String,
    pub base_url: String,
    /// Sealed with lemma-crypto; must be opened before use and masked on
    /// the way out.
    pub api_key: String,
    pub models: Json<Vec<String>>,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub api_path: String,
    pub models_path: String,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct S3Config {
    pub id: Uuid,
    pub user_id: Uuid,
    pub endpoint: String,
    pub region: String,
    pub bucket: String,
    /// Sealed with lemma-crypto.
    pub access_key: String,
    /// Sealed with lemma-crypto.
    pub secret_key: String,
    pub migration_from: Option<Json<serde_json::Value>>,
    pub migrated_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct Conversation {
    pub id: Uuid,
    pub user_id: Uuid,
    pub title: String,
    pub status: String,
    pub archived_at: Option<DateTime<Utc>>,
    /// S3 object key of the archived payload, set when archived.
    pub archive_key: Option<String>,
    /// Message count snapshot taken at archive time.
    pub message_count: Option<i32>,
    /// Sync version. Every UPDATE must set it explicitly via
    /// `nextval('sync_seq')`; the column default only applies to INSERT.
    pub sync_seq: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TokenUsage {
    pub prompt: i64,
    pub completion: i64,
    pub total: i64,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct Message {
    pub id: Uuid,
    pub conversation_id: Uuid,
    pub role: String,
    pub content: String,
    pub provider_id: Option<Uuid>,
    pub model: Option<String>,
    /// Client-supplied idempotency key for send retries.
    pub client_msg_id: Option<String>,
    pub status: String,
    pub token_usage: Option<Json<TokenUsage>>,
    /// Sync version. Every UPDATE must set it explicitly via
    /// `nextval('sync_seq')`; the column default only applies to INSERT.
    pub sync_seq: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    /// Per-conversation ordering sequence.
    pub seq: i64,
}
