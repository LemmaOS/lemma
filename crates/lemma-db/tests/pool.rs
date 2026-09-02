#![allow(clippy::unwrap_used, clippy::expect_used, missing_docs)]

use sqlx::{AssertSqlSafe, Row};

fn base_url() -> String {
    dotenvy::dotenv().ok();
    std::env::var("DATABASE_URL").expect("DATABASE_URL must be set")
}

fn fresh_db_url(base: &str, db: &str) -> String {
    let (server, _) = base.rsplit_once('/').expect("DATABASE_URL has no path");
    format!("{server}/{db}")
}

#[tokio::test]
async fn connect_and_migrate_against_fresh_database() {
    let url = base_url();
    let db = format!("lemma_db_test_{}", uuid::Uuid::new_v4().simple());

    let admin = lemma_db::connect(&url).await.unwrap();
    sqlx::raw_sql(AssertSqlSafe(format!("CREATE DATABASE {db}")))
        .execute(&admin)
        .await
        .unwrap();

    let pool = lemma_db::connect(&fresh_db_url(&url, &db)).await.unwrap();
    lemma_db::migrate(&pool).await.unwrap();
    let row = sqlx::query("SELECT count(*) FROM conversations")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(row.get::<i64, _>(0), 0);

    pool.close().await;
    sqlx::raw_sql(AssertSqlSafe(format!("DROP DATABASE {db}")))
        .execute(&admin)
        .await
        .unwrap();
}

#[tokio::test]
async fn connect_rejects_unreachable_database() {
    assert!(
        lemma_db::connect("postgres://127.0.0.1:5433/postgres")
            .await
            .is_err()
    );
}
