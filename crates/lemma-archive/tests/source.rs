#![allow(clippy::unwrap_used, missing_docs)]

use lemma_archive::store::{self, UpsertS3Config};
use lemma_archive::{ArchiveSource, DbArchiveSource};
use lemma_auth::users;
use lemma_crypto::{derive_key, seal};
use sqlx::PgPool;
use uuid::Uuid;

const SECRET: &str = "test-master-key";

async fn new_user(pool: &PgPool) -> Uuid {
    let name = format!("u-{}", Uuid::new_v4());
    users::insert(pool, &name, &format!("{name}@example.com"), "hash")
        .await
        .unwrap()
        .id
}

async fn upsert_sealed_config(pool: &PgPool, user_id: Uuid, sealing_secret: &str) {
    let key = derive_key(sealing_secret);
    store::upsert(
        pool,
        &UpsertS3Config {
            user_id,
            endpoint: "http://s3:9000",
            region: "us-east-1",
            bucket: "bucket",
            access_key: &seal(&key, "ak").unwrap(),
            secret_key: &seal(&key, "sk").unwrap(),
            migration_from: None,
        },
    )
    .await
    .unwrap();
}

#[sqlx::test(migrations = "../lemma-db/migrations")]
async fn store_for_returns_none_without_config(pool: PgPool) {
    let source = DbArchiveSource::new(pool, SECRET);
    assert!(source.store_for(Uuid::new_v4()).await.unwrap().is_none());
}

#[sqlx::test(migrations = "../lemma-db/migrations")]
async fn store_for_resolves_store_with_unsealed_credentials(pool: PgPool) {
    let user = new_user(&pool).await;
    upsert_sealed_config(&pool, user, SECRET).await;

    let source = DbArchiveSource::new(pool, SECRET);
    assert!(source.store_for(user).await.unwrap().is_some());
}

#[sqlx::test(migrations = "../lemma-db/migrations")]
async fn store_for_fails_when_master_key_does_not_match(pool: PgPool) {
    let user = new_user(&pool).await;
    upsert_sealed_config(&pool, user, "other-secret").await;

    let source = DbArchiveSource::new(pool, SECRET);
    assert!(source.store_for(user).await.is_err());
}
