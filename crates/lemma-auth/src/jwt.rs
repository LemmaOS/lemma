use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// access token 寿命 15 分钟
pub const ACCESS_TOKEN_TTL_SECS: i64 = 15 * 60;

#[derive(Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub exp: usize,
}

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

    // 手工构造过期 token
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
