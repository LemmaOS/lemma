mod jwt;
mod password;
mod service;

pub use jwt::{Claims, sign_access_token, verify_access_token};
pub use password::{hash_password, verify_password};
pub use service::AuthService;

use rand::Rng;
use sha2::{Digest, Sha256};

// 32 字节随机 hex 明文，只在签发响应中出现一次，库侧存哈希
pub fn generate_refresh_token() -> String {
    let mut bytes = [0u8; 32];
    rand::rng().fill_bytes(&mut bytes);
    hex::encode(bytes)
}

pub fn hash_token(token: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(token.as_bytes());
    hex::encode(hasher.finalize())
}
