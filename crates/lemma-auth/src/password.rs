use argon2::Argon2;
use argon2::password_hash::{PasswordHasher, PasswordVerifier};

pub fn hash_password(password: &str) -> Result<String, argon2::password_hash::Error> {
    // 盐由 password-hash 自动生成（16 字节随机）
    Ok(Argon2::default()
        .hash_password(password.as_bytes())?
        .to_string())
}

pub fn verify_password(password: &str, hash: &str) -> bool {
    // PHC 串解析失败按验证失败处理（fail-closed）
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

    // 升级 argon2 0.5→0.6 时钉的存量格式样本（argon2id v19 标准参数）：旧哈希必须仍能验过
    #[test]
    fn legacy_hash_still_verifies() {
        let legacy = "$argon2id$v=19$m=19456,t=2,p=1$MDEyMzQ1Njc4OWFiY2RlZg$QsiXxtKuiCjdMPc/fr4G3IAsWreaz0M5T+SSHJdxTu0";
        assert!(verify_password("password123", legacy));
        assert!(!verify_password("wrong", legacy));
    }
}
