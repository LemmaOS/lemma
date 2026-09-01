//! Argon2id password hashing.

use argon2::Argon2;
use argon2::password_hash::{PasswordHasher, PasswordVerifier};

/// Hashes a password into a PHC string with embedded salt and parameters,
/// so hashes produced with older parameters keep verifying.
pub fn hash_password(password: &str) -> Result<String, argon2::password_hash::Error> {
    Ok(Argon2::default()
        .hash_password(password.as_bytes())?
        .to_string())
}

/// Verifies a password against a PHC hash. Fails closed: a malformed hash
/// is a mismatch, not an error.
pub fn verify_password(password: &str, hash: &str) -> bool {
    Argon2::default()
        .verify_password(password.as_bytes(), hash)
        .is_ok()
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use super::*;

    #[test]
    fn hash_and_verify_roundtrip() {
        let hash = hash_password("password123").unwrap();
        assert!(verify_password("password123", &hash));
        assert!(!verify_password("wrong", &hash));
    }

    #[test]
    fn malformed_hash_fails_closed() {
        assert!(!verify_password("password123", "not-a-hash"));
    }

    #[test]
    fn legacy_hash_still_verifies() {
        let legacy = "$argon2id$v=19$m=19456,t=2,p=1$MDEyMzQ1Njc4OWFiY2RlZg$QsiXxtKuiCjdMPc/fr4G3IAsWreaz0M5T+SSHJdxTu0";
        assert!(verify_password("password123", legacy));
        assert!(!verify_password("wrong", legacy));
    }
}
