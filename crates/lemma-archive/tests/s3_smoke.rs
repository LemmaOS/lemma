#![allow(clippy::unwrap_used, missing_docs)]

use std::time::Duration;

use lemma_archive::{ArchiveStore, S3ArchiveStore, S3Config};
use uuid::Uuid;

fn cfg(bucket: &str) -> S3Config {
    let env_or = |k: &str, d: &str| std::env::var(k).unwrap_or_else(|_| d.to_string());
    S3Config {
        endpoint: env_or("LEMMA_S3_SMOKE_ENDPOINT", "http://127.0.0.1:9000"),
        region: env_or("LEMMA_S3_SMOKE_REGION", "us-east-1"),
        bucket: bucket.to_string(),
        access_key_id: env_or("LEMMA_S3_SMOKE_ACCESS_KEY_ID", "lemma"),
        secret_access_key: env_or("LEMMA_S3_SMOKE_SECRET_ACCESS_KEY", "lemma-secret"),
    }
}

async fn probe() -> bool {
    let store = S3ArchiveStore::new(&cfg(&format!("probe-{}", Uuid::new_v4())));
    matches!(
        tokio::time::timeout(Duration::from_secs(2), store.bucket_exists()).await,
        Ok(Ok(false))
    )
}

#[tokio::test]
async fn put_overwrite_get_delete_roundtrip() {
    if !probe().await {
        eprintln!("skip: rustfs not reachable at 127.0.0.1:9000");
        return;
    }
    let store = S3ArchiveStore::new(&cfg("lemma"));
    let key = format!("smoke/{}.json", Uuid::new_v4());

    store.put(&key, b"first".as_slice()).await.unwrap();
    assert_eq!(store.get(&key).await.unwrap().unwrap(), b"first".to_vec());

    store.put(&key, b"second".as_slice()).await.unwrap();
    assert_eq!(store.get(&key).await.unwrap().unwrap(), b"second".to_vec());

    store.delete(&key).await.unwrap();
    store.delete(&key).await.unwrap();
    assert!(store.get(&key).await.unwrap().is_none());
}

#[tokio::test]
async fn bucket_exists_true_and_false() {
    if !probe().await {
        eprintln!("skip: rustfs not reachable at 127.0.0.1:9000");
        return;
    }
    let main = S3ArchiveStore::new(&cfg("lemma"));
    assert!(main.bucket_exists().await.unwrap());

    let missing = S3ArchiveStore::new(&cfg(&format!("missing-{}", Uuid::new_v4())));
    assert!(!missing.bucket_exists().await.unwrap());
}
