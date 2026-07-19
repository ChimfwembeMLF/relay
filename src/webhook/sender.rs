use std::time::Duration;

use chrono::Utc;
use hmac::{Hmac, Mac};
use reqwest::Client;
use serde::Serialize;
use sha2::Sha256;

use crate::db::queries;
use crate::error::AppError;
use crate::models::{Invoice, Transaction};
use crate::AppState;

type HmacSha256 = Hmac<Sha256>;

#[derive(Serialize)]
struct WebhookPayload<'a> {
    event: &'static str,
    payment_id: uuid::Uuid,
    system_id: uuid::Uuid,
    external_id: &'a str,
    status: &'a str,
    amount: i64,
    currency: &'a str,
    country: &'a str,
    gateway_reference: Option<&'a str>,
    error: Option<&'a str>,
    timestamp: String,
}

#[derive(Serialize)]
struct InvoiceWebhookPayload<'a> {
    event: &'static str,
    invoice_id: uuid::Uuid,
    reference: &'a str,
    amount: i64,
    currency: &'a str,
    country: &'a str,
    status: &'static str,
    transaction_id: uuid::Uuid,
    timestamp: String,
}

pub async fn deliver_payment_webhook(
    state: &AppState,
    url: &str,
    transaction: &Transaction,
) -> Result<(), AppError> {
    let payload = WebhookPayload {
        event: "payment.status_changed",
        payment_id: transaction.id,
        system_id: transaction.system_id,
        external_id: &transaction.external_id,
        status: &transaction.status,
        amount: transaction.amount,
        currency: &transaction.currency,
        country: &transaction.country,
        gateway_reference: transaction.gateway_reference.as_deref(),
        error: transaction.error.as_deref(),
        timestamp: Utc::now().to_rfc3339(),
    };

    let body = serde_json::to_string(&payload)
        .map_err(|e| AppError::Internal(format!("webhook serialization failed: {e}")))?;

    deliver_with_retries(
        state,
        url,
        transaction.id,
        "payment.status_changed",
        &body,
    )
    .await
}

pub async fn deliver_invoice_webhook(
    state: &AppState,
    url: &str,
    invoice: &Invoice,
    transaction: &Transaction,
) -> Result<(), AppError> {
    let payload = InvoiceWebhookPayload {
        event: "invoice.paid",
        invoice_id: invoice.id,
        reference: &invoice.reference,
        amount: invoice.amount,
        currency: &invoice.currency,
        country: &invoice.country,
        status: "paid",
        transaction_id: transaction.id,
        timestamp: Utc::now().to_rfc3339(),
    };

    let body = serde_json::to_string(&payload)
        .map_err(|e| AppError::Internal(format!("webhook serialization failed: {e}")))?;

    deliver_with_retries(state, url, transaction.id, "invoice.paid", &body).await
}

async fn deliver_with_retries(
    state: &AppState,
    url: &str,
    transaction_id: uuid::Uuid,
    event_type: &str,
    body: &str,
) -> Result<(), AppError> {
    let client = Client::builder()
        .timeout(Duration::from_secs(10))
        .build()
        .map_err(|e| AppError::Internal(e.to_string()))?;

    let signature = sign_payload(&state.config.webhook_signing_secret, body);

    for attempt in 1..=3 {
        if attempt > 1 {
            tokio::time::sleep(Duration::from_millis(200 * attempt as u64)).await;
        }

        let result = client
            .post(url)
            .header("Content-Type", "application/json")
            .header("X-Relay-Signature", format!("sha256={signature}"))
            .body(body.to_string())
            .send()
            .await;

        match result {
            Ok(resp) => {
                let status = resp.status().as_u16() as i32;
                let success = resp.status().is_success();
                queries::record_webhook_attempt(
                    state.db.pool(),
                    transaction_id,
                    attempt,
                    url,
                    Some(status),
                    success,
                    None,
                    event_type,
                )
                .await?;
                if success {
                    return Ok(());
                }
            }
            Err(e) => {
                queries::record_webhook_attempt(
                    state.db.pool(),
                    transaction_id,
                    attempt,
                    url,
                    None,
                    false,
                    Some(&e.to_string()),
                    event_type,
                )
                .await?;
            }
        }
    }

    Err(AppError::Internal("webhook delivery failed after retries".into()))
}

fn sign_payload(secret: &str, body: &str) -> String {
    let mut mac = HmacSha256::new_from_slice(secret.as_bytes()).expect("HMAC key");
    mac.update(body.as_bytes());
    hex::encode(mac.finalize().into_bytes())
}
