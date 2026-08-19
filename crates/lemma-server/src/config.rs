#[derive(Clone)]
pub struct Config {
    pub database_url: String,
    pub jwt_secret: String,
}

impl Config {
    // 启动即失败，不给缺配置跑起来的机会
    pub fn from_env() -> Self {
        Self {
            database_url: std::env::var("DATABASE_URL").expect("DATABASE_URL not set"),
            jwt_secret: std::env::var("LEMMA_JWT_SECRET").expect("LEMMA_JWT_SECRET not set"),
        }
    }
}
