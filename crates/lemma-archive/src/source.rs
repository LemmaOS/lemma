//! Per-user resolution of archive stores.

use std::sync::Arc;

use lemma_crypto::{derive_key, open};
use sqlx::PgPool;
use uuid::Uuid;

use crate::store;
use crate::{ArchiveError, ArchiveStore, S3ArchiveStore, S3Config};

/// Resolves the archive store for a user. `None` means the user has no
/// storage configured, in which case archiving stays database-only.
pub trait ArchiveSource: Send + Sync + 'static {
    /// The store implementation this source produces.
    type Store: ArchiveStore;
    /// Resolves the user's store, or `None` when no storage is
    /// configured.
    fn store_for(
        &self,
        user_id: Uuid,
    ) -> impl Future<Output = Result<Option<Arc<Self::Store>>, ArchiveError>> + Send;
}

/// Resolves stores from the s3_configs table, opening the sealed
/// credentials on each call so config edits take effect immediately.
pub struct DbArchiveSource {
    pool: PgPool,
    secret_key: String,
}

impl DbArchiveSource {
    /// Creates the source. `secret_key` derives the key that opens the
    /// sealed credentials stored in s3_configs.
    pub fn new(pool: PgPool, secret_key: impl Into<String>) -> Self {
        Self {
            pool,
            secret_key: secret_key.into(),
        }
    }
}

impl ArchiveSource for DbArchiveSource {
    type Store = S3ArchiveStore;

    fn store_for(
        &self,
        user_id: Uuid,
    ) -> impl Future<Output = Result<Option<Arc<S3ArchiveStore>>, ArchiveError>> + Send {
        let pool = self.pool.clone();
        let secret_key = self.secret_key.clone();
        async move {
            let Some(cfg) = store::find_by_user(&pool, user_id)
                .await
                .map_err(|e| ArchiveError(format!("db: {e}")))?
            else {
                return Ok(None);
            };
            let key = derive_key(&secret_key);
            let access_key_id = open(&key, &cfg.access_key)
                .map_err(|e| ArchiveError(format!("open access_key: {e}")))?;
            let secret_access_key = open(&key, &cfg.secret_key)
                .map_err(|e| ArchiveError(format!("open secret_key: {e}")))?;
            Ok(Some(Arc::new(S3ArchiveStore::new(&S3Config {
                endpoint: cfg.endpoint,
                region: cfg.region,
                bucket: cfg.bucket,
                access_key_id,
                secret_access_key,
            }))))
        }
    }
}
