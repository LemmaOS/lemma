use lemma_archive::S3Config;

#[derive(Clone)]
pub struct Config {
    pub database_url: String,
    pub jwt_secret: String,
    pub secret_key: String,
    pub s3: Option<S3Config>,
}

impl Config {
    // 启动即失败，不给缺配置跑起来的机会；S3 可选（缺任一即降级就地归档）
    pub fn from_env() -> Result<Self, Box<dyn std::error::Error>> {
        let s3 = match (
            std::env::var("LEMMA_S3_ENDPOINT"),
            std::env::var("LEMMA_S3_REGION"),
            std::env::var("LEMMA_S3_BUCKET"),
            std::env::var("LEMMA_S3_ACCESS_KEY_ID"),
            std::env::var("LEMMA_S3_SECRET_ACCESS_KEY"),
        ) {
            (Ok(endpoint), Ok(region), Ok(bucket), Ok(access_key_id), Ok(secret_access_key)) => {
                Some(S3Config {
                    endpoint,
                    region,
                    bucket,
                    access_key_id,
                    secret_access_key,
                })
            }
            _ => None,
        };
        Ok(Self {
            database_url: std::env::var("DATABASE_URL")?,
            jwt_secret: std::env::var("LEMMA_JWT_SECRET")?,
            secret_key: std::env::var("LEMMA_SECRET_KEY")?,
            s3,
        })
    }
}
