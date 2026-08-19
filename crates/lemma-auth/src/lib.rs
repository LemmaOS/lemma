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

// 校验 Bearer access token，返回 user_id；各服务统一走这里做鉴权
pub fn require_user(
    secret: &str,
    ctx: &connectrpc::RequestContext,
) -> Result<uuid::Uuid, connectrpc::ConnectError> {
    let token = ctx
        .header("authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .ok_or_else(|| connectrpc::ConnectError::unauthenticated("missing bearer token"))?;
    let claims = verify_access_token(secret, token)
        .map_err(|_| connectrpc::ConnectError::unauthenticated("invalid access token"))?;
    uuid::Uuid::parse_str(&claims.sub)
        .map_err(|_| connectrpc::ConnectError::unauthenticated("invalid access token"))
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use super::*;

    #[test]
    fn hash_token_is_deterministic() {
        assert_eq!(hash_token("abc"), hash_token("abc"));
        assert_ne!(hash_token("abc"), hash_token("abd"));
    }

    #[test]
    fn refresh_token_is_unique_hex() {
        let a = generate_refresh_token();
        let b = generate_refresh_token();
        assert_eq!(a.len(), 64);
        assert!(a.chars().all(|c| c.is_ascii_hexdigit()));
        assert_ne!(a, b);
    }
}
