//! 业务错误构造：码进 proto 闭集，英文文案兜底，前端按 reason 出 i18n 文案。
//! 运维错误（internal + 原文）不经过这里。

use base64::Engine as _;
use buffa::Message;
use connectrpc::{ConnectError, ErrorCode, ErrorDetail};

use crate::lemma::v1::{ErrorInfo, ErrorReason};

// type_url 用裸名：Connect-JSON 通道原样透传，connect-es findDetails 按全名精确匹配
const ERROR_INFO_TYPE: &str = "lemma.v1.ErrorInfo";

// reason → connect 传输层错误码（HTTP 语义，与既有行为一致）
fn transport_code(reason: ErrorReason) -> ErrorCode {
    match reason {
        ErrorReason::CredentialsInvalid | ErrorReason::TokenInvalid => ErrorCode::Unauthenticated,
        ErrorReason::UserNotFound
        | ErrorReason::ProviderNotFound
        | ErrorReason::ConversationNotFound
        | ErrorReason::ConversationNotActive
        | ErrorReason::ConversationNotArchived
        | ErrorReason::ArchivedConversationNotFound
        | ErrorReason::MessageNotFound
        | ErrorReason::BucketNotFound => ErrorCode::NotFound,
        ErrorReason::StorageHasArchives => ErrorCode::FailedPrecondition,
        _ => ErrorCode::InvalidArgument,
    }
}

// reason → 英文兜底文案（curl / 日志 / 前端未知码时的展示）
fn message(reason: ErrorReason) -> &'static str {
    match reason {
        ErrorReason::CredentialsInvalid => "invalid credentials",
        ErrorReason::UsernameTaken => "username or email already taken",
        ErrorReason::SignupFieldsRequired => {
            "username and email required, password at least 8 chars"
        }
        ErrorReason::LoginTargetRequired => "provide exactly one of username or email",
        ErrorReason::TokenInvalid => "invalid token",
        ErrorReason::UserNotFound => "user not found",
        ErrorReason::ProviderFieldsRequired => "name, base_url and api_key required",
        ErrorReason::ProviderKindInvalid => "invalid provider kind",
        ErrorReason::ProviderNotFound => "provider not found",
        ErrorReason::ProviderDisabled => "provider disabled",
        ErrorReason::IdInvalid => "invalid id",
        ErrorReason::TitleRequired => "title required",
        ErrorReason::ConversationNotFound => "conversation not found",
        ErrorReason::ConversationNotActive => "conversation not found or already archived",
        ErrorReason::ConversationNotArchived => "conversation not found or not archived",
        ErrorReason::ArchivedConversationNotFound => "archived conversation not found",
        ErrorReason::MessageNotFound => "message not found",
        ErrorReason::NotAssistantMessage => "not an assistant message",
        ErrorReason::ContentRequired => "content required",
        ErrorReason::ModelRequired => "model required",
        ErrorReason::StorageEndpointRequired => "endpoint required",
        ErrorReason::StorageBucketRequired => "bucket required",
        ErrorReason::StorageAccessKeyRequired => "access_key required",
        ErrorReason::StorageSecretKeyRequired => "secret_key required",
        ErrorReason::StorageNotConfigured => "storage not configured",
        ErrorReason::MigrationNotPending => "no pending migration",
        ErrorReason::StorageHasArchives => {
            "archived conversations still reference this storage; restore or delete them first"
        }
        ErrorReason::BucketNotFound => "bucket not found",
        ErrorReason::Unspecified => "unspecified error reason",
    }
}

/// 造一个带 ErrorInfo detail 的业务错误
pub fn app_error(reason: ErrorReason) -> ConnectError {
    app_error_with(reason, &[])
}

/// 同上，携带 i18n 插值参数（如桶名）
pub fn app_error_with(reason: ErrorReason, attrs: &[(&str, &str)]) -> ConnectError {
    let info = ErrorInfo {
        reason: reason.into(),
        attrs: attrs
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect(),
        ..Default::default()
    };
    ConnectError::new(transport_code(reason), message(reason))
        .with_detail(ErrorDetail::from_message(ERROR_INFO_TYPE, &info))
}

/// 解出错误携带的业务码（无 detail 或解码失败返回 None）
pub fn error_reason(e: &ConnectError) -> Option<ErrorReason> {
    let d = e.details.iter().find(|d| d.type_url == ERROR_INFO_TYPE)?;
    let value = d.value.as_deref()?;
    let bytes = base64::engine::general_purpose::STANDARD_NO_PAD
        .decode(value)
        .ok()?;
    ErrorInfo::decode_from_slice(&bytes).ok()?.reason.as_known()
}
