use std::sync::Arc;

use lemma_crypto::{derive_key, open};
use sqlx::PgPool;
use uuid::Uuid;

use crate::store;
use crate::{ArchiveError, ArchiveStore, S3ArchiveStore, S3Config};

/// 按用户解析归档存储：返回 None = 未配置（就地归档）。
/// 每次 DB 直查、不缓存——配置改动运行时天然生效（与 providers 每次查库同哲学）
pub trait ArchiveSource: Send + Sync + 'static {
    type Store: ArchiveStore;
    fn store_for(
        &self,
        user_id: Uuid,
    ) -> impl Future<Output = Result<Option<Arc<Self::Store>>, ArchiveError>> + Send;
}

/// 生产实现：查 s3_configs、解密凭证、现场构造 S3 客户端
pub struct DbArchiveSource {
    pool: PgPool,
    secret_key: String,
}

impl DbArchiveSource {
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
