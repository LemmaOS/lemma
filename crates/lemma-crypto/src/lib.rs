//! Credential sealing for secrets at rest, such as provider API keys and
//! S3 credentials.
//!
//! Secrets are encrypted with AES-256-GCM under a key derived from the
//! configured master secret and stored base64-encoded in the database.
//! They never leave the backend in plaintext; responses carry the masked
//! form produced by [`mask`].

use aes_gcm::Aes256Gcm;
use aes_gcm::aead::{Aead, Generate, Key, KeyInit, Nonce};
use base64::prelude::*;
use sha2::{Digest, Sha256};

/// Failure modes of [`seal`] and [`open`].
#[derive(Debug)]
pub enum CryptoError {
    /// AEAD operation failed. In practice this means decryption with the
    /// wrong key or a tampered payload.
    Decrypt,
    /// Input is not a well-formed sealed payload: bad base64, truncated,
    /// or not valid UTF-8 after decryption.
    Encoding,
}

impl std::fmt::Display for CryptoError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CryptoError::Decrypt => write!(f, "decrypt failed"),
            CryptoError::Encoding => write!(f, "invalid sealed key encoding"),
        }
    }
}

impl std::error::Error for CryptoError {}

/// Derives the AES-256 key from the master secret via SHA-256.
///
/// The derivation is deterministic and unsalted, so rotating the master
/// secret makes every previously sealed value unreadable.
pub fn derive_key(secret: &str) -> Key<Aes256Gcm> {
    let digest = Sha256::digest(secret.as_bytes());
    let mut key = [0u8; 32];
    key.copy_from_slice(&digest);
    Key::<Aes256Gcm>::from(key)
}

/// Encrypts `plaintext` and returns the sealed payload as
/// base64(nonce ‖ ciphertext) with a fresh random 12-byte nonce per call.
pub fn seal(key: &Key<Aes256Gcm>, plaintext: &str) -> Result<String, CryptoError> {
    let cipher = Aes256Gcm::new(key);
    let nonce = Nonce::<Aes256Gcm>::generate();
    // AES-GCM encryption only fails on payload size overflow, so Decrypt
    // doubles as the generic AEAD error here.
    let ct = cipher
        .encrypt(&nonce, plaintext.as_bytes())
        .map_err(|_| CryptoError::Decrypt)?;
    let mut out = Vec::with_capacity(12 + ct.len());
    out.extend_from_slice(nonce.as_slice());
    out.extend_from_slice(&ct);
    Ok(BASE64_STANDARD.encode(out))
}

/// Reverses [`seal`]: [`CryptoError::Encoding`] for malformed input,
/// [`CryptoError::Decrypt`] for a wrong key or tampered data.
pub fn open(key: &Key<Aes256Gcm>, sealed: &str) -> Result<String, CryptoError> {
    let raw = BASE64_STANDARD
        .decode(sealed)
        .map_err(|_| CryptoError::Encoding)?;
    let (nonce, ct) = raw.split_at_checked(12).ok_or(CryptoError::Encoding)?;
    let cipher = Aes256Gcm::new(key);
    let nonce = Nonce::<Aes256Gcm>::try_from(nonce).map_err(|_| CryptoError::Encoding)?;
    let pt = cipher
        .decrypt(&nonce, ct)
        .map_err(|_| CryptoError::Decrypt)?;
    String::from_utf8(pt).map_err(|_| CryptoError::Encoding)
}

/// Renders a secret for display: the first 3 and last 4 characters around
/// `****`, or just `****` when the secret is 8 characters or fewer.
///
/// Display-only. A masked value must never be passed back into [`seal`]:
/// it would be sealed as-is and silently destroy the real secret.
pub fn mask(plain: &str) -> String {
    let len = plain.chars().count();
    if len <= 8 {
        return "****".to_string();
    }
    let head: String = plain.chars().take(3).collect();
    let tail: String = plain.chars().skip(len - 4).collect();
    format!("{head}****{tail}")
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use super::*;

    #[test]
    fn roundtrip() {
        let key = derive_key("test-secret");
        let sealed = seal(&key, "sk-abc123xyz987").unwrap();
        assert_ne!(sealed, "sk-abc123xyz987");
        assert_eq!(open(&key, &sealed).unwrap(), "sk-abc123xyz987");
    }

    #[test]
    fn wrong_key_fails() {
        let sealed = seal(&derive_key("a"), "secret").unwrap();
        assert!(open(&derive_key("b"), &sealed).is_err());
    }

    #[test]
    fn mask_works() {
        assert_eq!(mask("sk-1234567890abc"), "sk-****0abc");
        assert_eq!(mask("short"), "****");
    }
}
