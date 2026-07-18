use rand::Rng;
use sha2::{Digest, Sha256};

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
