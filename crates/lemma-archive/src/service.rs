//! Handler for the StorageService RPCs.

use buffa::MessageField;
use buffa_types::google::protobuf::Timestamp;
use connectrpc::{
    ConnectError, RequestContext, Response, ServiceRequest, ServiceResult, ServiceStream,
};
use lemma_auth::require_user;
use lemma_crypto::{derive_key, mask, open, seal};
use lemma_db::entity::S3Config as DbS3Config;
use lemma_proto::lemma::v1::{
    DeleteStorageConfigResponse, ErrorReason, GetStorageConfigResponse, MigrateArchivesResponse,
    StorageConfig, TestStorageConfigResponse, UpdateStorageConfigResponse,
};
use lemma_proto::{app_error, app_error_with};
use sqlx::PgPool;

use crate::store::{self, UpsertS3Config};
use crate::{ArchiveError, ArchiveStore, S3ArchiveStore, S3Config};

/// Snapshot of the previous storage backend, taken when the config
/// changes while archives exist. Credentials stay sealed. Drives the
/// migrate_archives stream.
#[derive(serde::Serialize, serde::Deserialize)]
struct MigrationFrom {
    endpoint: String,
    region: String,
    bucket: String,
    access_key: String,
    secret_key: String,
}

impl From<&DbS3Config> for MigrationFrom {
    fn from(c: &DbS3Config) -> Self {
        Self {
            endpoint: c.endpoint.clone(),
            region: c.region.clone(),
            bucket: c.bucket.clone(),
            access_key: c.access_key.clone(),
            secret_key: c.secret_key.clone(),
        }
    }
}

/// Connect handler implementing the StorageService RPCs. Holds two
/// secrets: `jwt_secret` authenticates requests, `secret_key` derives the
/// key that seals storage credentials at rest.
pub struct StorageService {
    pool: PgPool,
    jwt_secret: String,
    secret_key: String,
}

impl StorageService {
    /// Creates the handler.
    pub fn new(pool: PgPool, jwt_secret: impl Into<String>, secret_key: impl Into<String>) -> Self {
        Self {
            pool,
            jwt_secret: jwt_secret.into(),
            secret_key: secret_key.into(),
        }
    }
}

fn to_proto(c: &DbS3Config, secret_key: &str) -> StorageConfig {
    let key = derive_key(secret_key);
    let masked = |sealed: &str| {
        open(&key, sealed)
            .map(|k| mask(&k))
            .unwrap_or_else(|_| "****".to_string())
    };
    StorageConfig {
        endpoint: c.endpoint.clone(),
        region: c.region.clone(),
        bucket: c.bucket.clone(),
        access_key: masked(&c.access_key),
        secret_key: masked(&c.secret_key),
        pending_migration: c.migration_from.is_some(),
        migrated_at: match c.migrated_at {
            Some(t) => Timestamp::from(t).into(),
            None => MessageField::none(),
        },
        ..Default::default()
    }
}

fn map_db(e: sqlx::Error) -> ConnectError {
    ConnectError::internal(format!("db: {e}"))
}

fn map_archive(e: ArchiveError) -> ConnectError {
    ConnectError::internal(format!("{e}"))
}

fn seal_with(secret_key: &str, plain: &str) -> Result<String, ConnectError> {
    let key = derive_key(secret_key);
    seal(&key, plain).map_err(|e| ConnectError::internal(format!("seal key: {e}")))
}

fn open_with(secret_key: &str, sealed: &str) -> Result<String, ConnectError> {
    let key = derive_key(secret_key);
    open(&key, sealed).map_err(|e| ConnectError::internal(format!("open key: {e}")))
}

// Empty or absent input keeps the old value. The frontend never
// resubmits the masked display value, so whatever arrives is real input.
fn pick(new: Option<&str>, old: Option<&str>) -> Option<String> {
    match new {
        Some(s) if !s.trim().is_empty() => Some(s.trim().to_string()),
        _ => old.map(|s| s.to_string()),
    }
}

/// Copies archive objects between two stores, invoking `on_progress`
/// after each key. Objects missing at the source are counted as skipped
/// rather than failing the run. Returns (done, total, skipped).
pub async fn copy_archive_objects<F>(
    from: &impl ArchiveStore,
    to: &impl ArchiveStore,
    keys: &[String],
    mut on_progress: F,
) -> Result<(u32, u32, u32), ArchiveError>
where
    F: FnMut(u32, u32, u32),
{
    let total = keys.len() as u32;
    let (mut done, mut skipped) = (0u32, 0u32);
    for key in keys {
        match from.get(key).await {
            Ok(Some(bytes)) => {
                to.put(key, &bytes).await?;
                done += 1;
            }
            Ok(None) => skipped += 1,
            Err(e) => return Err(e),
        }
        on_progress(done, total, skipped);
    }
    Ok((done, total, skipped))
}

#[allow(refining_impl_trait)]
impl lemma_proto::lemma::v1::StorageService for StorageService {
    async fn get_storage_config(
        &self,
        ctx: RequestContext,
        _request: ServiceRequest<'_, lemma_proto::lemma::v1::GetStorageConfigRequest>,
    ) -> ServiceResult<GetStorageConfigResponse> {
        let user_id = require_user(&self.jwt_secret, &ctx)?;
        let cfg = store::find_by_user(&self.pool, user_id)
            .await
            .map_err(map_db)?;
        let config = cfg.map(|c| to_proto(&c, &self.secret_key));
        Response::ok(GetStorageConfigResponse {
            config: match config {
                Some(c) => MessageField::some(c),
                None => MessageField::none(),
            },
            ..Default::default()
        })
    }

    async fn update_storage_config(
        &self,
        ctx: RequestContext,
        request: ServiceRequest<'_, lemma_proto::lemma::v1::UpdateStorageConfigRequest>,
    ) -> ServiceResult<UpdateStorageConfigResponse> {
        let user_id = require_user(&self.jwt_secret, &ctx)?;
        let existing = store::find_by_user(&self.pool, user_id)
            .await
            .map_err(map_db)?;
        let old = existing.as_ref();

        let endpoint = pick(request.endpoint, old.map(|c| c.endpoint.as_str()))
            .ok_or_else(|| app_error(ErrorReason::StorageEndpointRequired))?;
        let bucket = pick(request.bucket, old.map(|c| c.bucket.as_str()))
            .ok_or_else(|| app_error(ErrorReason::StorageBucketRequired))?;
        let region = pick(request.region, old.map(|c| c.region.as_str()))
            .unwrap_or_else(|| "us-east-1".to_string());

        let sealed_access = match request.access_key {
            Some(k) if !k.is_empty() => seal_with(&self.secret_key, k)?,
            _ => old
                .map(|c| c.access_key.clone())
                .ok_or_else(|| app_error(ErrorReason::StorageAccessKeyRequired))?,
        };
        let sealed_secret = match request.secret_key {
            Some(k) if !k.is_empty() => seal_with(&self.secret_key, k)?,
            _ => old
                .map(|c| c.secret_key.clone())
                .ok_or_else(|| app_error(ErrorReason::StorageSecretKeyRequired))?,
        };

        let backend_changed = old.is_some_and(|c| c.endpoint != endpoint || c.bucket != bucket);
        let preserved = old.and_then(|c| c.migration_from.as_ref().map(|j| j.0.clone()));
        let mut counted: Option<Vec<String>> = None;
        let migration_from = if backend_changed {
            // Switching endpoint or bucket strands existing archives on
            // the old backend: snapshot it so migrate_archives can copy
            // the objects over. No archives, no migration needed.
            let keys = store::list_archive_keys(&self.pool, user_id)
                .await
                .map_err(map_db)?;
            if keys.is_empty() {
                preserved
            } else {
                counted = Some(keys);
                old.map(|c| {
                    serde_json::to_value(MigrationFrom::from(c))
                        .map_err(|e| ConnectError::internal(format!("snapshot: {e}")))
                })
                .transpose()?
            }
        } else {
            preserved
        };

        let cfg = store::upsert(
            &self.pool,
            &UpsertS3Config {
                user_id,
                endpoint: &endpoint,
                region: &region,
                bucket: &bucket,
                access_key: &sealed_access,
                secret_key: &sealed_secret,
                migration_from,
            },
        )
        .await
        .map_err(map_db)?;

        let migration_total = if cfg.migration_from.is_some() {
            match counted {
                Some(keys) => keys.len() as u32,
                None => store::list_archive_keys(&self.pool, user_id)
                    .await
                    .map_err(map_db)?
                    .len() as u32,
            }
        } else {
            0
        };
        Response::ok(UpdateStorageConfigResponse {
            config: to_proto(&cfg, &self.secret_key).into(),
            migration_total,
            ..Default::default()
        })
    }

    async fn delete_storage_config(
        &self,
        ctx: RequestContext,
        _request: ServiceRequest<'_, lemma_proto::lemma::v1::DeleteStorageConfigRequest>,
    ) -> ServiceResult<DeleteStorageConfigResponse> {
        let user_id = require_user(&self.jwt_secret, &ctx)?;
        let keys = store::list_archive_keys(&self.pool, user_id)
            .await
            .map_err(map_db)?;
        // Deleting the storage config would orphan the archive objects
        // it points at, so archives must be restored or deleted first.
        if !keys.is_empty() {
            return Err(app_error(ErrorReason::StorageHasArchives));
        }
        store::delete_by_user(&self.pool, user_id)
            .await
            .map_err(map_db)?;
        Response::ok(DeleteStorageConfigResponse::default())
    }

    async fn test_storage_config(
        &self,
        ctx: RequestContext,
        request: ServiceRequest<'_, lemma_proto::lemma::v1::TestStorageConfigRequest>,
    ) -> ServiceResult<TestStorageConfigResponse> {
        let user_id = require_user(&self.jwt_secret, &ctx)?;
        let saved = store::find_by_user(&self.pool, user_id)
            .await
            .map_err(map_db)?;
        let old = saved.as_ref();

        let pick = |new: &str, old: Option<&str>| -> Option<String> {
            if !new.trim().is_empty() {
                Some(new.trim().trim_end_matches('/').to_string())
            } else {
                old.map(|s| s.to_string())
            }
        };
        let endpoint = pick(request.endpoint, old.map(|c| c.endpoint.as_str()))
            .ok_or_else(|| app_error(ErrorReason::StorageEndpointRequired))?;
        let bucket = pick(request.bucket, old.map(|c| c.bucket.as_str()))
            .ok_or_else(|| app_error(ErrorReason::StorageBucketRequired))?;
        let region = pick(request.region, old.map(|c| c.region.as_str()))
            .unwrap_or_else(|| "us-east-1".to_string());
        let access = if !request.access_key.is_empty() {
            request.access_key.to_string()
        } else {
            match old {
                Some(c) => open_with(&self.secret_key, &c.access_key)?,
                None => return Err(app_error(ErrorReason::StorageAccessKeyRequired)),
            }
        };
        let secret = if !request.secret_key.is_empty() {
            request.secret_key.to_string()
        } else {
            match old {
                Some(c) => open_with(&self.secret_key, &c.secret_key)?,
                None => return Err(app_error(ErrorReason::StorageSecretKeyRequired)),
            }
        };

        let probe = S3ArchiveStore::new(&S3Config {
            endpoint,
            region,
            bucket: bucket.clone(),
            access_key_id: access,
            secret_access_key: secret,
        });
        // Probe only. Auto-creating a missing bucket was removed
        // deliberately; do not reintroduce it here.
        if !probe.bucket_exists().await.map_err(map_archive)? {
            return Err(app_error_with(
                ErrorReason::BucketNotFound,
                &[("bucket", &bucket)],
            ));
        }
        let message = "bucket reachable".to_string();
        Response::ok(TestStorageConfigResponse {
            message,
            ..Default::default()
        })
    }

    async fn migrate_archives(
        &self,
        ctx: RequestContext,
        _request: ServiceRequest<'_, lemma_proto::lemma::v1::MigrateArchivesRequest>,
    ) -> ServiceResult<ServiceStream<MigrateArchivesResponse>> {
        let user_id = require_user(&self.jwt_secret, &ctx)?;
        let cfg = store::find_by_user(&self.pool, user_id)
            .await
            .map_err(map_db)?
            .ok_or_else(|| app_error(ErrorReason::StorageNotConfigured))?;
        let Some(snapshot) = cfg.migration_from.as_ref().map(|j| j.0.clone()) else {
            return Err(app_error(ErrorReason::MigrationNotPending));
        };
        let from: MigrationFrom = serde_json::from_value(snapshot)
            .map_err(|e| ConnectError::internal(format!("snapshot: {e}")))?;

        let old_store = S3ArchiveStore::new(&S3Config {
            endpoint: from.endpoint,
            region: from.region,
            bucket: from.bucket,
            access_key_id: open_with(&self.secret_key, &from.access_key)?,
            secret_access_key: open_with(&self.secret_key, &from.secret_key)?,
        });
        let new_store = S3ArchiveStore::new(&S3Config {
            endpoint: cfg.endpoint.clone(),
            region: cfg.region.clone(),
            bucket: cfg.bucket.clone(),
            access_key_id: open_with(&self.secret_key, &cfg.access_key)?,
            secret_access_key: open_with(&self.secret_key, &cfg.secret_key)?,
        });

        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
        let pool = self.pool.clone();
        // The copy runs in a spawned task; failures travel in-band as a
        // final frame with finished=true and the error message set.
        tokio::spawn(async move {
            let fail = |msg: String| {
                let _ = tx.send(Ok(MigrateArchivesResponse {
                    finished: true,
                    error: msg,
                    ..Default::default()
                }));
            };
            let keys = match store::list_archive_keys(&pool, user_id).await {
                Ok(k) => k,
                Err(e) => return fail(format!("db: {e}")),
            };
            let result =
                copy_archive_objects(&old_store, &new_store, &keys, |done, total, skipped| {
                    let _ = tx.send(Ok(MigrateArchivesResponse {
                        done,
                        total,
                        skipped,
                        ..Default::default()
                    }));
                })
                .await;
            match result {
                Ok((done, total, skipped)) => match store::clear_migration(&pool, user_id).await {
                    Ok(_) => {
                        let _ = tx.send(Ok(MigrateArchivesResponse {
                            done,
                            total,
                            skipped,
                            finished: true,
                            ..Default::default()
                        }));
                    }
                    Err(e) => fail(format!("db: {e}")),
                },
                Err(e) => fail(format!("{e}")),
            }
        });

        Response::stream_ok(Box::pin(
            tokio_stream::wrappers::UnboundedReceiverStream::new(rx),
        ))
    }
}
