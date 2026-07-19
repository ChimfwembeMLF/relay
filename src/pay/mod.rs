use chrono::{DateTime, Utc};
use hmac::{Hmac, Mac};
use sha2::Sha256;
use uuid::Uuid;

use crate::models::Invoice;

type HmacSha256 = Hmac<Sha256>;

const TEMPLATE_OPEN: &str = include_str!("templates/open.html");
const TEMPLATE_PAID: &str = include_str!("templates/paid.html");
const TEMPLATE_EXPIRED: &str = include_str!("templates/expired.html");
const TEMPLATE_CANCELLED: &str = include_str!("templates/cancelled.html");
const TEMPLATE_NOT_FOUND: &str = include_str!("templates/not_found.html");
const TEMPLATE_ERROR: &str = include_str!("templates/error.html");

pub fn format_amount(amount: i64, currency: &str) -> String {
    let major = amount / 100;
    let minor = (amount.abs() % 100) as i64;
    format!("{currency} {major}.{minor:02}")
}

pub fn providers_for_country(country: &str) -> &'static [(&'static str, &'static str)] {
    match country {
        "ZM" => &[("MTN_MOMO_ZMB", "MTN MoMo Zambia")],
        "US" => &[("STRIPE_US", "Card (US)")],
        _ => &[],
    }
}

pub fn provider_options_html(country: &str) -> String {
    providers_for_country(country)
        .iter()
        .map(|(value, label)| format!(r#"<option value="{value}">{label}</option>"#))
        .collect::<Vec<_>>()
        .join("\n")
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

pub fn render_not_found() -> String {
    TEMPLATE_NOT_FOUND.to_string()
}

pub fn render_expired() -> String {
    TEMPLATE_EXPIRED.to_string()
}

pub fn render_cancelled() -> String {
    TEMPLATE_CANCELLED.to_string()
}

pub fn render_paid(invoice: &Invoice) -> String {
    TEMPLATE_PAID
        .replace("{{amount_display}}", &format_amount(invoice.amount, &invoice.currency))
        .replace("{{reference}}", &invoice.reference)
}

pub fn render_open(invoice: &Invoice, form_token: &str, idempotency_key: &Uuid) -> String {
    let description_line = invoice
        .description
        .as_ref()
        .map(|d| format!("{d}<br>"))
        .unwrap_or_default();

    TEMPLATE_OPEN
        .replace("{{amount_display}}", &format_amount(invoice.amount, &invoice.currency))
        .replace("{{description_line}}", &description_line)
        .replace("{{expires_at}}", &invoice.expires_at.to_rfc3339())
        .replace("{{reference}}", &invoice.reference)
        .replace("{{provider_options}}", &provider_options_html(&invoice.country))
        .replace("{{form_token}}", form_token)
        .replace("{{idempotency_key}}", &idempotency_key.to_string())
}

pub fn render_error(reference: &str, message: &str) -> String {
    TEMPLATE_ERROR
        .replace("{{reference}}", reference)
        .replace("{{message}}", message)
}

pub fn render_success(invoice: &Invoice) -> String {
    render_paid(invoice)
}
