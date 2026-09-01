//! Auth domain: signup, login, token refresh and rotation, and the
//! queries for the users and refresh_tokens tables.

mod jwt;
mod password;
mod service;
pub mod tokens;
pub mod users;

pub use jwt::{Claims, sign_access_token, verify_access_token};
pub use password::{hash_password, verify_password};
pub use service::AuthService;

use rand::Rng;
use sha2::{Digest, Sha256};

/// Generates a new refresh token: 256 random bits, hex-encoded. The
/// plaintext goes to the client; only [`hash_token`] of it is stored.
pub fn generate_refresh_token() -> String {
    let mut bytes = [0u8; 32];
    rand::rng().fill_bytes(&mut bytes);
    hex::encode(bytes)
}

/// SHA-256 of a refresh token, for storage and lookup. The plaintext
/// token is never persisted.
pub fn hash_token(token: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(token.as_bytes());
    hex::encode(hasher.finalize())
}

/// Authenticates a request via its `Authorization: Bearer` access token
/// and returns the user id. Every failure mode maps to the same
/// `TokenInvalid` error so callers cannot probe which check failed.
pub fn require_user(
    secret: &str,
    ctx: &connectrpc::RequestContext,
) -> Result<uuid::Uuid, connectrpc::ConnectError> {
    use lemma_proto::app_error;
    use lemma_proto::lemma::v1::ErrorReason;

    let token = ctx
        .header("authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .ok_or_else(|| app_error(ErrorReason::TokenInvalid))?;
    let claims =
        verify_access_token(secret, token).map_err(|_| app_error(ErrorReason::TokenInvalid))?;
    uuid::Uuid::parse_str(&claims.sub).map_err(|_| app_error(ErrorReason::TokenInvalid))
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
