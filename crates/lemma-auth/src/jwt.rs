//! HS256 access-token signing and verification.

use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Lifetime of an access token: 15 minutes.
pub const ACCESS_TOKEN_TTL_SECS: i64 = 15 * 60;

/// JWT claims. `sub` carries the user id.
#[derive(Serialize, Deserialize)]
#[allow(missing_docs)]
pub struct Claims {
    pub sub: String,
    pub exp: usize,
}

/// Signs an access token for `user_id` expiring after
/// [`ACCESS_TOKEN_TTL_SECS`].
pub fn sign_access_token(
    secret: &str,
    user_id: Uuid,
) -> Result<String, jsonwebtoken::errors::Error> {
    let exp = (chrono::Utc::now() + chrono::Duration::seconds(ACCESS_TOKEN_TTL_SECS)).timestamp()
        as usize;
    let claims = Claims {
        sub: user_id.to_string(),
        exp,
    };
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
}

/// Verifies an access token's signature and expiry.
pub fn verify_access_token(
    secret: &str,
    token: &str,
) -> Result<Claims, jsonwebtoken::errors::Error> {
    decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )
    .map(|data| data.claims)
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use super::*;
    use jsonwebtoken::{EncodingKey, Header};

    #[test]
    fn roundtrip() {
        let uid = Uuid::new_v4();
        let token = sign_access_token("secret", uid).unwrap();
        let claims = verify_access_token("secret", &token).unwrap();
        assert_eq!(claims.sub, uid.to_string());
    }

    #[test]
    fn wrong_secret_rejected() {
        let token = sign_access_token("secret", Uuid::new_v4()).unwrap();
        assert!(verify_access_token("other", &token).is_err());
    }

    #[test]
    fn expired_token_rejected() {
        let claims = Claims {
            sub: Uuid::new_v4().to_string(),
            exp: (chrono::Utc::now() - chrono::Duration::hours(2)).timestamp() as usize,
        };
        let token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(b"secret"),
        )
        .unwrap();
        assert!(verify_access_token("secret", &token).is_err());
    }
}
