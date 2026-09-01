#![allow(clippy::unwrap_used)]

use base64::Engine as _;
use buffa::Message;
use connectrpc::ErrorCode;
use lemma_proto::lemma::v1::{ErrorInfo, ErrorReason};
use lemma_proto::{app_error, app_error_with};

fn error_info(e: &connectrpc::ConnectError) -> ErrorInfo {
    let d = e.details.first().unwrap();
    assert_eq!(d.type_url, "lemma.v1.ErrorInfo");
    let value = d.value.as_deref().unwrap();
    let bytes = base64::engine::general_purpose::STANDARD_NO_PAD
        .decode(value)
        .unwrap();
    ErrorInfo::decode_from_slice(&bytes).unwrap()
}

#[test]
fn app_error_carries_code_message_and_detail() {
    let e = app_error(ErrorReason::ProviderNotFound);
    assert_eq!(e.code, ErrorCode::NotFound);
    assert_eq!(e.message.as_deref(), Some("provider not found"));
    let info = error_info(&e);
    assert_eq!(info.reason.as_known(), Some(ErrorReason::ProviderNotFound));
    assert!(info.attrs.is_empty());
}

#[test]
fn attrs_survive_roundtrip() {
    let e = app_error_with(ErrorReason::BucketNotFound, &[("bucket", "lemma")]);
    assert_eq!(e.code, ErrorCode::NotFound);
    let info = error_info(&e);
    assert_eq!(info.reason.as_known(), Some(ErrorReason::BucketNotFound));
    assert_eq!(info.attrs["bucket"], "lemma");
}

#[test]
fn transport_codes_match_business_semantics() {
    assert_eq!(
        app_error(ErrorReason::TokenInvalid).code,
        ErrorCode::Unauthenticated
    );
    assert_eq!(
        app_error(ErrorReason::CredentialsInvalid).code,
        ErrorCode::Unauthenticated
    );
    assert_eq!(
        app_error(ErrorReason::StorageHasArchives).code,
        ErrorCode::FailedPrecondition
    );
    assert_eq!(
        app_error(ErrorReason::UsernameTaken).code,
        ErrorCode::InvalidArgument
    );
}

#[test]
fn error_reason_decodes_from_detail() {
    let e = app_error(ErrorReason::StorageHasArchives);
    assert_eq!(
        lemma_proto::error_reason(&e),
        Some(ErrorReason::StorageHasArchives)
    );
    assert_eq!(
        lemma_proto::error_reason(&connectrpc::ConnectError::internal("db: boom")),
        None
    );
}
