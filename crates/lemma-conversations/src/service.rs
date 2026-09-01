use buffa::MessageField;
use buffa_types::google::protobuf::Timestamp;
use chrono::Utc;
use connectrpc::{ConnectError, RequestContext, Response, ServiceRequest, ServiceResult};
use lemma_archive::{
    ArchiveError, ArchiveSource, ArchiveStore, deserialize_envelope, envelope_from_messages,
    messages_from_envelope, object_key, serialize_envelope,
};
use lemma_auth::require_user;
use lemma_db::entity::{Conversation as DbConversation, Message as DbMessage};
use lemma_proto::app_error;
use lemma_proto::lemma::v1::{
    ArchiveConversationResponse, Conversation, ConversationStatus, CreateConversationResponse,
    DeleteArchivedResponse, ErrorReason, ListArchivedResponse, ListConversationsResponse,
    ListMessagesResponse, Message, MessageStatus, RenameConversationResponse,
    RestoreConversationResponse,
};
use sqlx::PgPool;
use uuid::Uuid;

use crate::store;

const DEFAULT_PAGE_LIMIT: i32 = 50;
const MAX_PAGE_LIMIT: i32 = 100;

pub struct ConversationService<S: ArchiveSource> {
    pool: PgPool,
    jwt_secret: String,
    archive: S,
}

impl<S: ArchiveSource> ConversationService<S> {
    pub fn new(pool: PgPool, jwt_secret: impl Into<String>, archive: S) -> Self {
        Self {
            pool,
            jwt_secret: jwt_secret.into(),
            archive,
        }
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

fn parse_id(id: &str) -> Result<Uuid, ConnectError> {
    Uuid::parse_str(id).map_err(|_| app_error(ErrorReason::IdInvalid))
}

fn map_archive(e: ArchiveError) -> ConnectError {
    ConnectError::internal(format!("{e}"))
}

fn map_db(e: sqlx::Error) -> ConnectError {
    ConnectError::internal(format!("db: {e}"))
}

#[allow(refining_impl_trait)]
impl<S: ArchiveSource> lemma_proto::lemma::v1::ConversationService for ConversationService<S> {
    async fn list_conversations(
        &self,
        ctx: RequestContext,
        _request: ServiceRequest<'_, lemma_proto::lemma::v1::ListConversationsRequest>,
    ) -> ServiceResult<ListConversationsResponse> {
        let user_id = require_user(&self.jwt_secret, &ctx)?;
        let list = store::list_active_by_user(&self.pool, user_id)
            .await
            .map_err(map_db)?;
        Response::ok(ListConversationsResponse {
            conversations: list.iter().map(conversation_to_proto).collect(),
            ..Default::default()
        })
    }

    async fn create_conversation(
        &self,
        ctx: RequestContext,
        _request: ServiceRequest<'_, lemma_proto::lemma::v1::CreateConversationRequest>,
    ) -> ServiceResult<CreateConversationResponse> {
        let user_id = require_user(&self.jwt_secret, &ctx)?;
        let c = store::insert(&self.pool, user_id).await.map_err(map_db)?;
        Response::ok(CreateConversationResponse {
            conversation: conversation_to_proto(&c).into(),
            ..Default::default()
        })
    }

    async fn rename_conversation(
        &self,
        ctx: RequestContext,
        request: ServiceRequest<'_, lemma_proto::lemma::v1::RenameConversationRequest>,
    ) -> ServiceResult<RenameConversationResponse> {
        let user_id = require_user(&self.jwt_secret, &ctx)?;
        let id = parse_id(request.id)?;
        let title = request.title.trim();
        if title.is_empty() {
            return Err(app_error(ErrorReason::TitleRequired));
        }
        let c = store::rename(&self.pool, id, user_id, title)
            .await
            .map_err(map_db)?
            .ok_or_else(|| app_error(ErrorReason::ConversationNotFound))?;
        Response::ok(RenameConversationResponse {
            conversation: conversation_to_proto(&c).into(),
            ..Default::default()
        })
    }

    async fn list_messages(
        &self,
        ctx: RequestContext,
        request: ServiceRequest<'_, lemma_proto::lemma::v1::ListMessagesRequest>,
    ) -> ServiceResult<ListMessagesResponse> {
        let user_id = require_user(&self.jwt_secret, &ctx)?;
        let conversation_id = parse_id(request.conversation_id)?;
        store::find_by_id_and_user(&self.pool, conversation_id, user_id)
            .await
            .map_err(map_db)?
            .ok_or_else(|| app_error(ErrorReason::ConversationNotFound))?;
        let before_id = if request.before_id.is_empty() {
            None
        } else {
            Some(parse_id(request.before_id)?)
        };
        let limit = if request.limit <= 0 {
            DEFAULT_PAGE_LIMIT
        } else {
            request.limit.min(MAX_PAGE_LIMIT)
        };
        let (messages, has_more) =
            store::list_messages(&self.pool, conversation_id, before_id, limit as i64)
                .await
                .map_err(map_db)?;
        Response::ok(ListMessagesResponse {
            messages: messages.iter().map(message_to_proto).collect(),
            has_more,
            ..Default::default()
        })
    }

    async fn archive_conversation(
        &self,
        ctx: RequestContext,
        request: ServiceRequest<'_, lemma_proto::lemma::v1::ArchiveConversationRequest>,
    ) -> ServiceResult<ArchiveConversationResponse> {
        let user_id = require_user(&self.jwt_secret, &ctx)?;
        let id = parse_id(request.id)?;
        let store = self.archive.store_for(user_id).await.map_err(map_archive)?;
        let mut tx = self.pool.begin().await.map_err(map_db)?;
        if store::lock_active(&mut tx, id, user_id)
            .await
            .map_err(map_db)?
            .is_none()
        {
            return Err(app_error(ErrorReason::ConversationNotActive));
        }

        let conversation = if let Some(archive) = store {
            let messages = store::list_all_messages(&mut tx, id)
                .await
                .map_err(map_db)?;
            let envelope = envelope_from_messages(id, Utc::now(), &messages);
            let bytes = serialize_envelope(&envelope).map_err(map_archive)?;
            let key = object_key(id);
            archive.put(&key, &bytes).await.map_err(map_archive)?;
            let c = store::mark_archived_with_key(&mut tx, id, &key)
                .await
                .map_err(map_db)?;
            store::delete_all_messages(&mut tx, id)
                .await
                .map_err(map_db)?;
            c
        } else {
            store::archive(&mut *tx, id, user_id)
                .await
                .map_err(map_db)?
                .ok_or_else(|| app_error(ErrorReason::ConversationNotActive))?
        };
        tx.commit().await.map_err(map_db)?;

        Response::ok(ArchiveConversationResponse {
            conversation: conversation_to_proto(&conversation).into(),
            ..Default::default()
        })
    }

    async fn restore_conversation(
        &self,
        ctx: RequestContext,
        request: ServiceRequest<'_, lemma_proto::lemma::v1::RestoreConversationRequest>,
    ) -> ServiceResult<RestoreConversationResponse> {
        let user_id = require_user(&self.jwt_secret, &ctx)?;
        let id = parse_id(request.id)?;
        let store = self.archive.store_for(user_id).await.map_err(map_archive)?;
        let mut tx = self.pool.begin().await.map_err(map_db)?;
        let locked = store::lock_archived(&mut tx, id, user_id)
            .await
            .map_err(map_db)?;
        let key = locked
            .ok_or_else(|| app_error(ErrorReason::ConversationNotArchived))?
            .archive_key;

        if let (Some(archive), Some(key)) = (store.as_ref(), key.as_deref()) {
            let bytes = archive
                .get(key)
                .await
                .map_err(map_archive)?
                .ok_or_else(|| ConnectError::internal("archive object missing"))?;
            let envelope = deserialize_envelope(&bytes).map_err(map_archive)?;
            let messages = messages_from_envelope(&envelope).map_err(map_archive)?;
            store::insert_restored(&mut tx, &messages)
                .await
                .map_err(map_db)?;
        }
        let conversation = store::restore(&mut *tx, id, user_id)
            .await
            .map_err(map_db)?
            .ok_or_else(|| app_error(ErrorReason::ConversationNotArchived))?;
        tx.commit().await.map_err(map_db)?;

        if let (Some(archive), Some(key)) = (store.as_ref(), key.as_deref()) {
            let _ = archive.delete(key).await;
        }

        Response::ok(RestoreConversationResponse {
            conversation: conversation_to_proto(&conversation).into(),
            ..Default::default()
        })
    }

    async fn list_archived(
        &self,
        ctx: RequestContext,
        _request: ServiceRequest<'_, lemma_proto::lemma::v1::ListArchivedRequest>,
    ) -> ServiceResult<ListArchivedResponse> {
        let user_id = require_user(&self.jwt_secret, &ctx)?;
        let list = store::list_archived_by_user(&self.pool, user_id)
            .await
            .map_err(map_db)?;
        Response::ok(ListArchivedResponse {
            conversations: list.iter().map(conversation_to_proto).collect(),
            ..Default::default()
        })
    }

    async fn delete_archived(
        &self,
        ctx: RequestContext,
        request: ServiceRequest<'_, lemma_proto::lemma::v1::DeleteArchivedRequest>,
    ) -> ServiceResult<DeleteArchivedResponse> {
        let user_id = require_user(&self.jwt_secret, &ctx)?;
        let id = parse_id(request.id)?;
        let store = self.archive.store_for(user_id).await.map_err(map_archive)?;
        let key = store::find_archive_key(&self.pool, id, user_id)
            .await
            .map_err(map_db)?;
        if key.is_none() {
            return Err(app_error(ErrorReason::ArchivedConversationNotFound));
        }
        let deleted = store::delete_archived(&self.pool, id, user_id)
            .await
            .map_err(map_db)?;
        if !deleted {
            return Err(app_error(ErrorReason::ArchivedConversationNotFound));
        }
        if let (Some(archive), Some(key)) = (store.as_ref(), key.flatten().as_deref()) {
            let _ = archive.delete(key).await;
        }
        Response::ok(DeleteArchivedResponse::default())
    }
}
