use argon2::Argon2;
use argon2::password_hash::{PasswordHash, PasswordHasher as _, PasswordVerifier, SaltString};
use async_trait::async_trait;
use rand::RngCore;
use sqlx::PgPool;

use crate::port::{PasswordHasher, TokenIssuer, UserStore};

const SQL_SIGNUP: &str =
    "INSERT INTO users (username, password_hash) VALUES ($1, $2) RETURNING username";
const SQL_FIND_USER: &str = "SELECT password_hash FROM users WHERE username = $1";
const SQL_EXISTS: &str = "SELECT username FROM users WHERE username = $1";

const TOKEN_BYTES: usize = 32;
const HEX_CHARS: &[u8] = b"0123456789abcdef";
const ERR_HASH: &str = "hash failed";

pub struct Argon2Hasher;

impl Argon2Hasher {
    pub fn hash_password(password: &str) -> Result<String, String> {
        let salt = SaltString::generate(&mut rand::thread_rng());
        Argon2::default()
            .hash_password(password.as_bytes(), &salt)
            .map(|h| h.to_string())
            .map_err(|_| ERR_HASH.to_string())
    }

    pub fn verify_password(password: &str, hash: &str) -> bool {
        PasswordHash::new(hash)
            .map(|parsed| {
                Argon2::default()
                    .verify_password(password.as_bytes(), &parsed)
                    .is_ok()
            })
            .unwrap_or(false)
    }
}

impl PasswordHasher for Argon2Hasher {
    fn hash(&self, password: &str) -> Result<String, String> {
        Self::hash_password(password)
    }

    fn verify(&self, password: &str, hash: &str) -> bool {
        Self::verify_password(password, hash)
    }
}

pub struct HexTokenIssuer;

impl TokenIssuer for HexTokenIssuer {
    fn issue(&self) -> String {
        let mut bytes = [0u8; TOKEN_BYTES];
        rand::thread_rng().fill_bytes(&mut bytes);
        let mut out = String::with_capacity(TOKEN_BYTES * 2);
        for b in bytes {
            out.push(HEX_CHARS[(b >> 4) as usize] as char);
            out.push(HEX_CHARS[(b & 0x0f) as usize] as char);
        }
        out
    }
}

#[derive(Clone)]
pub struct PgUserStore {
    pool: PgPool,
}

impl PgUserStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl UserStore for PgUserStore {
    async fn exists(&self, username: &str) -> Result<bool, String> {
        sqlx::query_scalar::<_, String>(SQL_EXISTS)
            .bind(username)
            .fetch_optional(&self.pool)
            .await
            .map(|row| row.is_some())
            .map_err(|e| e.to_string())
    }

    async fn create(&self, username: &str, password_hash: &str) -> Result<String, String> {
        sqlx::query_scalar(SQL_SIGNUP)
            .bind(username)
            .bind(password_hash)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| e.to_string())
    }

    async fn password_hash(&self, username: &str) -> Result<Option<String>, String> {
        sqlx::query_scalar(SQL_FIND_USER)
            .bind(username)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| e.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const PASSWORD: &str = "s3cret-pass";

    #[test]
    fn hash_roundtrip() {
        let hash = Argon2Hasher::hash_password(PASSWORD).expect("hash");
        assert!(Argon2Hasher::verify_password(PASSWORD, &hash));
        assert!(!Argon2Hasher::verify_password("wrong", &hash));
    }

    #[test]
    fn token_is_unique_hex() {
        let issuer = HexTokenIssuer;
        let a = issuer.issue();
        let b = issuer.issue();
        assert_eq!(a.len(), TOKEN_BYTES * 2);
        assert_ne!(a, b);
        assert!(a.chars().all(|c| HEX_CHARS.contains(&(c as u8))));
    }
}
