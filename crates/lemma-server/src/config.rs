//! Runtime configuration from environment variables.

/// All variables are required; main() loads `.env` first.
#[derive(Clone)]
pub struct Config {
    /// PostgreSQL connection string. Use 127.0.0.1, not localhost.
    pub database_url: String,
    /// Signs and verifies access tokens.
    pub jwt_secret: String,
    /// Master secret from which credential-sealing keys are derived.
    /// Rotating it makes all sealed values unreadable.
    pub secret_key: String,
}

impl Config {
    pub fn from_env() -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            database_url: std::env::var("DATABASE_URL")?,
            jwt_secret: std::env::var("LEMMA_JWT_SECRET")?,
            secret_key: std::env::var("LEMMA_SECRET_KEY")?,
        })
    }
}
