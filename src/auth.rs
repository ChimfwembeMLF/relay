use rand::Rng;
use sha2::{Digest, Sha256};

use crate::error::AppError;

pub fn generate_api_key() -> String {
    let mut rng = rand::thread_rng();
    let random: String = (0..32)
        .map(|_| format!("{:02x}", rng.gen::<u8>()))
        .collect();
    format!("sk_live_{random}")
}

pub fn hash_api_key(api_key: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(api_key.as_bytes());
    hex::encode(hasher.finalize())
}

pub fn hash_request_body(body: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(body.as_bytes());
    hex::encode(hasher.finalize())
}

pub fn verify_api_key(api_key: &str, stored_hash: &str) -> bool {
    hash_api_key(api_key) == stored_hash
}

pub fn hash_password(password: &str) -> Result<String, AppError> {
    use argon2::{
        password_hash::{rand_core::OsRng, PasswordHasher, SaltString},
        Argon2,
    };

    let salt = SaltString::generate(&mut OsRng);
    Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map(|h| h.to_string())
        .map_err(|e| AppError::Internal(format!("password hash failed: {e}")))
}

pub fn verify_password(password: &str, password_hash: &str) -> Result<bool, AppError> {
    use argon2::{
        password_hash::{PasswordHash, PasswordVerifier},
        Argon2,
    };

    let parsed = PasswordHash::new(password_hash)
        .map_err(|e| AppError::Internal(format!("invalid password hash: {e}")))?;
    Ok(Argon2::default()
        .verify_password(password.as_bytes(), &parsed)
        .is_ok())
}
