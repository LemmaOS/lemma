#![allow(clippy::unwrap_used)]

use buffa::Message;
use connectrpc::{CodecFormat, Encodable, JsonSerialize};
use connectrpc::{ErrorCode, HasMessageView, RequestContext, ServiceRequest};
use http::HeaderMap;
use lemma_archive::MemoryArchiveStore;
use lemma_archive::copy_archive_objects;
use lemma_archive::store;
use lemma_archive::{ArchiveError, ArchiveStore, StorageService};
use lemma_auth::{sign_access_token, users};
use lemma_crypto::{derive_key, open};
use lemma_proto::lemma::v1::StorageService as StorageServiceRpc;
use sqlx::PgPool;
use uuid::Uuid;

const SECRET: &str = "test-secret";
const CRYPTO_SECRET: &str = "crypto-secret";

fn svc(pool: &PgPool) -> StorageService {
    StorageService::new(pool.clone(), SECRET, CRYPTO_SECRET)
}

async fn new_user(pool: &PgPool) -> (Uuid, String) {
    let name = format!("u-{}", Uuid::new_v4());
    let id = users::insert(pool, &name, &format!("{name}@example.com"), "hash")
        .await
        .unwrap()
        .id;
    let token = sign_access_token(SECRET, id).unwrap();
    (id, token)
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

async fn update(
    svc: &StorageService,
    token: &str,
    endpoint: Option<&str>,
    region: Option<&str>,
    bucket: Option<&str>,
    access: Option<&str>,
    secret: Option<&str>,
) -> Result<lemma_proto::lemma::v1::UpdateStorageConfigResponse, connectrpc::ConnectError> {
    let msg = lemma_proto::lemma::v1::UpdateStorageConfigRequest {
        endpoint: endpoint.map(Into::into),
        region: region.map(Into::into),
        bucket: bucket.map(Into::into),
        access_key: access.map(Into::into),
        secret_key: secret.map(Into::into),
        ..Default::default()
    };
    let bytes = msg.encode_to_bytes();
    let view = lemma_proto::lemma::v1::UpdateStorageConfigRequest::decode_view(&bytes).unwrap();
    match svc
        .update_storage_config(bearer_ctx(token), ServiceRequest::from_parts(&view, &bytes))
        .await
    {
        Ok(resp) => Ok(owned_body(&resp.body)),
        Err(e) => Err(e),
    }
}

async fn get(
    svc: &StorageService,
    token: &str,
) -> lemma_proto::lemma::v1::GetStorageConfigResponse {
    let msg = lemma_proto::lemma::v1::GetStorageConfigRequest::default();
    let bytes = msg.encode_to_bytes();
    let view = lemma_proto::lemma::v1::GetStorageConfigRequest::decode_view(&bytes).unwrap();
    owned_body(
        &svc.get_storage_config(bearer_ctx(token), ServiceRequest::from_parts(&view, &bytes))
            .await
            .unwrap()
            .body,
    )
}

async fn test_config(
    svc: &StorageService,
    token: &str,
    endpoint: &str,
    bucket: &str,
    access: &str,
    secret: &str,
) -> Result<lemma_proto::lemma::v1::TestStorageConfigResponse, connectrpc::ConnectError> {
    let msg = lemma_proto::lemma::v1::TestStorageConfigRequest {
        endpoint: endpoint.into(),
        bucket: bucket.into(),
        access_key: access.into(),
        secret_key: secret.into(),
        ..Default::default()
    };
    let bytes = msg.encode_to_bytes();
    let view = lemma_proto::lemma::v1::TestStorageConfigRequest::decode_view(&bytes).unwrap();
    match svc
        .test_storage_config(bearer_ctx(token), ServiceRequest::from_parts(&view, &bytes))
        .await
    {
        Ok(resp) => Ok(owned_body(&resp.body)),
        Err(e) => Err(e),
    }
}

async fn migrate_err(svc: &StorageService, token: &str) -> connectrpc::ConnectError {
    let msg = lemma_proto::lemma::v1::MigrateArchivesRequest::default();
    let bytes = msg.encode_to_bytes();
    let view = lemma_proto::lemma::v1::MigrateArchivesRequest::decode_view(&bytes).unwrap();
    svc.migrate_archives(bearer_ctx(token), ServiceRequest::from_parts(&view, &bytes))
        .await
        .err()
        .unwrap()
}

async fn seed_archived(pool: &PgPool, user: Uuid, key: &str) {
    sqlx::query(
        "INSERT INTO conversations (id, user_id, status, archive_key) VALUES ($1, $2, 'archived', $3)",
    )
    .bind(Uuid::new_v4())
    .bind(user)
    .bind(key)
    .execute(pool)
    .await
    .unwrap();
}

#[sqlx::test(migrations = "../lemma-db/migrations")]
async fn first_save_masks_secrets(pool: PgPool) {
    let s = svc(&pool);
    let (_, token) = new_user(&pool).await;

    assert!(get(&s, &token).await.config.as_option().is_none());

    let saved = update(
        &s,
        &token,
        Some("http://127.0.0.1:9000"),
        Some("us-east-1"),
        Some("lemma"),
        Some("AKIA-abcdefgh123"),
        Some("wJalrXUtnFEMIdeadbeef"),
    )
    .await
    .unwrap();
    assert_eq!(saved.migration_total, 0);
    assert_eq!(saved.config.access_key, "AKI****h123");
    assert_eq!(saved.config.secret_key, "wJa****beef");
}

#[sqlx::test(migrations = "../lemma-db/migrations")]
async fn first_save_requires_core_fields(pool: PgPool) {
    let s = svc(&pool);
    let (_, token) = new_user(&pool).await;

    let err = update(&s, &token, None, None, None, None, None)
        .await
        .err()
        .unwrap();
    assert_eq!(err.code, ErrorCode::InvalidArgument);
    assert_eq!(
        lemma_proto::error_reason(&err),
        Some(lemma_proto::lemma::v1::ErrorReason::StorageEndpointRequired)
    );

    let err = update(&s, &token, Some("http://x"), None, Some("b"), None, None)
        .await
        .err()
        .unwrap();
    assert_eq!(err.code, ErrorCode::InvalidArgument);
    assert_eq!(
        lemma_proto::error_reason(&err),
        Some(lemma_proto::lemma::v1::ErrorReason::StorageAccessKeyRequired)
    );
}

#[sqlx::test(migrations = "../lemma-db/migrations")]
async fn patch_keeps_unset_fields_and_secrets(pool: PgPool) {
    let s = svc(&pool);
    let (user, token) = new_user(&pool).await;
    update(
        &s,
        &token,
        Some("http://old:9000"),
        Some("us-east-1"),
        Some("b1"),
        Some("ak-old-key-12345"),
        Some("sk-old-key-12345"),
    )
    .await
    .unwrap();

    let saved = update(&s, &token, Some("http://new:9000"), None, None, None, None)
        .await
        .unwrap();
    assert_eq!(saved.config.endpoint, "http://new:9000");
    assert_eq!(saved.config.bucket, "b1");
    assert_eq!(saved.config.region, "us-east-1");

    let row = store::find_by_user(&pool, user).await.unwrap().unwrap();
    let key = derive_key(CRYPTO_SECRET);
    assert_eq!(open(&key, &row.access_key).unwrap(), "ak-old-key-12345");
    assert_eq!(open(&key, &row.secret_key).unwrap(), "sk-old-key-12345");
}

#[sqlx::test(migrations = "../lemma-db/migrations")]
async fn backend_change_snapshots_and_counts(pool: PgPool) {
    let s = svc(&pool);
    let (user, token) = new_user(&pool).await;
    update(
        &s,
        &token,
        Some("http://old:9000"),
        None,
        Some("b1"),
        Some("aaaaaaaaaa111"),
        Some("bbbbbbbbbb111"),
    )
    .await
    .unwrap();
    seed_archived(&pool, user, "archives/x.json").await;
    seed_archived(&pool, user, "archives/y.json").await;

    let saved = update(&s, &token, Some("http://new:9000"), None, None, None, None)
        .await
        .unwrap();
    assert_eq!(saved.migration_total, 2);
    assert!(saved.config.pending_migration);
    assert!(get(&s, &token).await.config.pending_migration);

    let saved = update(&s, &token, None, Some("ap-northeast-1"), None, None, None)
        .await
        .unwrap();
    assert_eq!(saved.migration_total, 2);

    let (_, t2) = new_user(&pool).await;
    update(
        &s,
        &t2,
        Some("http://a:9000"),
        None,
        Some("bx"),
        Some("cccccccccc111"),
        Some("dddddddddd111"),
    )
    .await
    .unwrap();
    let saved = update(&s, &t2, Some("http://b:9000"), None, None, None, None)
        .await
        .unwrap();
    assert_eq!(saved.migration_total, 0);
    assert!(!saved.config.pending_migration);
}

#[sqlx::test(migrations = "../lemma-db/migrations")]
async fn delete_guarded_by_archives(pool: PgPool) {
    let s = svc(&pool);
    let (user, token) = new_user(&pool).await;
    update(
        &s,
        &token,
        Some("http://x:9000"),
        None,
        Some("b"),
        Some("eeeeeeeeee111"),
        Some("ffffffffff111"),
    )
    .await
    .unwrap();
    seed_archived(&pool, user, "archives/g.json").await;

    let msg = lemma_proto::lemma::v1::DeleteStorageConfigRequest::default();
    let bytes = msg.encode_to_bytes();
    let view = lemma_proto::lemma::v1::DeleteStorageConfigRequest::decode_view(&bytes).unwrap();
    let err = s
        .delete_storage_config(
            bearer_ctx(&token),
            ServiceRequest::from_parts(&view, &bytes),
        )
        .await
        .err()
        .unwrap();
    assert_eq!(err.code, ErrorCode::FailedPrecondition);
    assert_eq!(
        lemma_proto::error_reason(&err),
        Some(lemma_proto::lemma::v1::ErrorReason::StorageHasArchives)
    );

    let (_, t2) = new_user(&pool).await;
    update(
        &s,
        &t2,
        Some("http://y:9000"),
        None,
        Some("b2"),
        Some("gggggggggg111"),
        Some("hhhhhhhhhh111"),
    )
    .await
    .unwrap();
    s.delete_storage_config(bearer_ctx(&t2), ServiceRequest::from_parts(&view, &bytes))
        .await
        .unwrap();
    assert!(get(&s, &t2).await.config.as_option().is_none());
}

#[sqlx::test(migrations = "../lemma-db/migrations")]
async fn test_validation_paths(pool: PgPool) {
    let s = svc(&pool);
    let (_, token) = new_user(&pool).await;

    let err = test_config(&s, &token, "", "", "", "").await.err().unwrap();
    assert_eq!(err.code, ErrorCode::InvalidArgument);

    let err = test_config(&s, &token, "", "b", "ak", "sk")
        .await
        .err()
        .unwrap();
    assert_eq!(err.code, ErrorCode::InvalidArgument);

    let err = test_config(&s, &token, "http://x", "b", "", "")
        .await
        .err()
        .unwrap();
    assert_eq!(err.code, ErrorCode::InvalidArgument);
}

#[sqlx::test(migrations = "../lemma-db/migrations")]
async fn migrate_requires_config_and_snapshot(pool: PgPool) {
    let s = svc(&pool);
    let (_, token) = new_user(&pool).await;

    assert_eq!(
        migrate_err(&s, &token).await.code,
        ErrorCode::InvalidArgument
    );

    update(
        &s,
        &token,
        Some("http://x:9000"),
        None,
        Some("b"),
        Some("iiiiiiiiii111"),
        Some("jjjjjjjjjj111"),
    )
    .await
    .unwrap();
    assert_eq!(
        migrate_err(&s, &token).await.code,
        ErrorCode::InvalidArgument
    );
}

#[tokio::test]
async fn copy_objects_core() {
    let from = MemoryArchiveStore::new();
    let to = MemoryArchiveStore::new();
    from.put("a", b"1".as_slice()).await.unwrap();
    from.put("b", b"2".as_slice()).await.unwrap();

    let keys = vec!["a".to_string(), "b".to_string(), "c".to_string()];
    let mut seen = Vec::new();
    let (done, total, skipped) =
        copy_archive_objects(&from, &to, &keys, |d, t, sk| seen.push((d, t, sk)))
            .await
            .unwrap();
    assert_eq!((done, total, skipped), (2, 3, 1));
    assert_eq!(seen.last().copied(), Some((2, 3, 1)));
    assert_eq!(to.get("a").await.unwrap().unwrap(), b"1".to_vec());
    assert!(to.get("c").await.unwrap().is_none());
}

struct FailingSink;

impl ArchiveStore for FailingSink {
    async fn put(&self, _key: &str, _content: &[u8]) -> Result<(), ArchiveError> {
        Err(ArchiveError("sink down".into()))
    }
    async fn get(&self, _key: &str) -> Result<Option<Vec<u8>>, ArchiveError> {
        Ok(None)
    }
    async fn delete(&self, _key: &str) -> Result<(), ArchiveError> {
        Ok(())
    }
}

#[tokio::test]
async fn copy_objects_error_propagates() {
    let from = MemoryArchiveStore::new();
    from.put("a", b"x".as_slice()).await.unwrap();
    let err = copy_archive_objects(&from, &FailingSink, &["a".to_string()], |_, _, _| {})
        .await
        .err()
        .unwrap();
    assert!(err.0.contains("sink down"));
}
