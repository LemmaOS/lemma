use aws_credential_types::Credentials;
use aws_sdk_s3::Client;
use aws_sdk_s3::config::{BehaviorVersion, Region};
use aws_sdk_s3::primitives::ByteStream;

use crate::{ArchiveError, ArchiveStore};

/// S3 连接参数（服务端从环境变量读出后传入）
#[derive(Clone)]
pub struct S3Config {
    pub endpoint: String,
    pub region: String,
    pub bucket: String,
    pub access_key_id: String,
    pub secret_access_key: String,
}

pub struct S3ArchiveStore {
    client: Client,
    bucket: String,
}

impl S3ArchiveStore {
    pub fn new(cfg: &S3Config) -> Self {
        let creds = Credentials::new(
            &cfg.access_key_id,
            &cfg.secret_access_key,
            None,
            None,
            "lemma-static",
        );
        let config = aws_sdk_s3::Config::builder()
            .behavior_version(BehaviorVersion::latest())
            .region(Region::new(cfg.region.clone()))
            .endpoint_url(&cfg.endpoint)
            .credentials_provider(creds)
            // MinIO / R2 均为 path-style；AWS 旧区兼容
            .force_path_style(true)
            .build();
        Self {
            client: Client::from_conf(config),
            bucket: cfg.bucket.clone(),
        }
    }
}

impl ArchiveStore for S3ArchiveStore {
    async fn put(&self, key: &str, content: &[u8]) -> Result<(), ArchiveError> {
        self.client
            .put_object()
            .bucket(&self.bucket)
            .key(key)
            .body(ByteStream::from(content.to_vec()))
            .send()
            .await
            .map(|_| ())
            .map_err(|e| ArchiveError(format!("put {key}: {e}")))
    }

    async fn get(&self, key: &str) -> Result<Option<Vec<u8>>, ArchiveError> {
        match self
            .client
            .get_object()
            .bucket(&self.bucket)
            .key(key)
            .send()
            .await
        {
            Ok(obj) => {
                let bytes = obj
                    .body
                    .collect()
                    .await
                    .map_err(|e| ArchiveError(format!("get {key}: {e}")))?
                    .into_bytes()
                    .to_vec();
                Ok(Some(bytes))
            }
            Err(e) => {
                let svc = e.into_service_error();
                if svc.is_no_such_key() {
                    Ok(None)
                } else {
                    Err(ArchiveError(format!("get {key}: {svc}")))
                }
            }
        }
    }

    async fn delete(&self, key: &str) -> Result<(), ArchiveError> {
        self.client
            .delete_object()
            .bucket(&self.bucket)
            .key(key)
            .send()
            .await
            .map(|_| ())
            .map_err(|e| ArchiveError(format!("delete {key}: {e}")))
    }
}
