use base64::Engine as _;
use buffa::Message;
use connectrpc::{ConnectError, ErrorCode, ErrorDetail};

use crate::lemma::v1::{ErrorInfo, ErrorReason};

const ERROR_INFO_TYPE: &str = "lemma.v1.ErrorInfo";

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

pub fn app_error(reason: ErrorReason) -> ConnectError {
    app_error_with(reason, &[])
}

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

pub fn error_reason(e: &ConnectError) -> Option<ErrorReason> {
    let d = e.details.iter().find(|d| d.type_url == ERROR_INFO_TYPE)?;
    let value = d.value.as_deref()?;
    let bytes = base64::engine::general_purpose::STANDARD_NO_PAD
        .decode(value)
        .ok()?;
    ErrorInfo::decode_from_slice(&bytes).ok()?.reason.as_known()
}
