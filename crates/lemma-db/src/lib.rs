//! Storage kernel: connection pool, migrations, and shared row entities.
//!
//! This crate owns no domain logic. Queries live in the domain crates
//! (lemma-auth, lemma-providers, lemma-conversations, lemma-archive);
//! only the row types shared across them are defined here.

pub mod entity;

use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;

/// Creates a PostgreSQL connection pool with default options.
pub async fn connect(url: &str) -> sqlx::Result<PgPool> {
    PgPoolOptions::new().connect(url).await
}

/// Runs the migrations embedded from `./migrations` at compile time.
pub async fn migrate(pool: &PgPool) -> Result<(), sqlx::migrate::MigrateError> {
    sqlx::migrate!("./migrations").run(pool).await
}
