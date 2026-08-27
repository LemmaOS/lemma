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

    // 桶探测：Ok(false) = 桶不存在（HeadBucket 对缺失桶建模为 NotFound）
    pub async fn bucket_exists(&self) -> Result<bool, ArchiveError> {
        match self.client.head_bucket().bucket(&self.bucket).send().await {
            Ok(_) => Ok(true),
            Err(e) => {
                // HEAD 响应无 body：服务端拒绝时 SDK 解析不到错误码，用 HTTP 状态兜底
                let status = e.raw_response().map(|r| r.status().as_u16());
                let svc = e.into_service_error();
                if svc.is_not_found() {
                    Ok(false)
                } else {
                    let meta = svc.meta();
                    let code = meta
                        .code()
                        .map(|c| c.to_string())
                        .or_else(|| status.map(|s| format!("HTTP {s}")))
                        .unwrap_or_else(|| "unreachable".to_string());
                    let detail = match (meta.message(), status) {
                        (Some(m), _) => m.to_string(),
                        (None, Some(403)) => "credentials or permission rejected".to_string(),
                        (None, Some(_)) => String::new(),
                        (None, None) => "cannot reach endpoint".to_string(),
                    };
                    Err(ArchiveError(format!(
                        "head {}: {} {}",
                        self.bucket, code, detail
                    )))
                }
            }
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
