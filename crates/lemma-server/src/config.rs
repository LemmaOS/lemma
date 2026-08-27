#[derive(Clone)]
pub struct Config {
    pub database_url: String,
    pub jwt_secret: String,
    pub secret_key: String,
}

impl Config {
    // 启动即失败，不给缺配置跑起来的机会
    pub fn from_env() -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            database_url: std::env::var("DATABASE_URL")?,
            jwt_secret: std::env::var("LEMMA_JWT_SECRET")?,
            secret_key: std::env::var("LEMMA_SECRET_KEY")?,
        })
    }
}
