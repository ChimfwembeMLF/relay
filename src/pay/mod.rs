use chrono::{DateTime, Utc};
use hmac::{Hmac, Mac};
use sha2::Sha256;

type HmacSha256 = Hmac<Sha256>;

pub fn format_amount(amount: i64, currency: &str) -> String {
    let major = amount / 100;
    let minor = (amount.abs() % 100) as i64;
    format!("{currency} {major}.{minor:02}")
}

/// Provider options as (correspondent, human label) from the PawaPay catalog.
pub fn providers_for_country(country: &str) -> Vec<(&'static str, &'static str)> {
    crate::catalog::providers_for_country(country)
        .iter()
        .map(|m| (m.correspondent, m.label))
        .collect()
}

pub fn generate_form_token(secret: &str, reference: &str, expires_at: &DateTime<Utc>) -> String {
    sign_reference_expiry(secret, reference, expires_at)
}

pub fn verify_form_token(
    secret: &str,
    reference: &str,
    expires_at: &DateTime<Utc>,
    token: &str,
) -> bool {
    let expected = sign_reference_expiry(secret, reference, expires_at);
    constant_time_eq(expected.as_bytes(), token.as_bytes())
}

fn sign_reference_expiry(secret: &str, reference: &str, expires_at: &DateTime<Utc>) -> String {
    let payload = format!("{reference}:{}", expires_at.to_rfc3339());
    let mut mac = HmacSha256::new_from_slice(secret.as_bytes()).expect("HMAC key");
    mac.update(payload.as_bytes());
    hex::encode(mac.finalize().into_bytes())
}

fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    a.iter().zip(b.iter()).fold(0u8, |acc, (x, y)| acc | (x ^ y)) == 0
}
