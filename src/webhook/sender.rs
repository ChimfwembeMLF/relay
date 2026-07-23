use std::time::Duration;

use chrono::Utc;
use hmac::{Hmac, Mac};
use reqwest::Client;
use serde::Serialize;
use sha2::Sha256;
use uuid::Uuid;

use crate::db::queries;
use crate::db::webhook_endpoints;
use crate::error::AppError;
use crate::models::{Invoice, Transaction};
use crate::queue::{self, WebhookJob};
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
    system_id: uuid::Uuid,
    reference: &'a str,
    amount: i64,
    currency: &'a str,
    country: &'a str,
    status: &'static str,
    transaction_id: uuid::Uuid,
    timestamp: String,
}

/// Fan-out a payment event to every enabled webhook endpoint across all tenants.
pub async fn broadcast_payment_webhook(
    state: &AppState,
    transaction: &Transaction,
) -> Result<(), AppError> {
    let subscribers = webhook_endpoints::list_enabled_webhook_urls(state.db.pool()).await?;
    if subscribers.is_empty() {
        tracing::debug!(payment_id = %transaction.id, "no webhook subscribers");
        return Ok(());
    }

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

    let mut handles = Vec::new();
    for (subscriber_id, url) in subscribers {
        let state = state.clone();
        let body = body.clone();
        let transaction_id = transaction.id;
        handles.push(tokio::spawn(async move {
            if let Err(e) = dispatch_webhook_body(
                &state,
                &url,
                transaction_id,
                "payment.status_changed",
                &body,
            )
            .await
            {
                tracing::error!(
                    payment_id = %transaction_id,
                    subscriber_system_id = %subscriber_id,
                    error = %e,
                    "webhook fan-out failed for subscriber"
                );
                return false;
            }
            true
        }));
    }

    let mut ok = 0usize;
    let mut err = 0usize;
    for handle in handles {
        match handle.await {
            Ok(true) => ok += 1,
            Ok(false) => err += 1,
            Err(e) => {
                err += 1;
                tracing::error!(error = %e, "webhook fan-out task join failed");
            }
        }
    }

    tracing::info!(
        payment_id = %transaction.id,
        delivered_or_enqueued = ok,
        failed = err,
        "payment webhook fan-out complete"
    );
    Ok(())
}

/// Fan-out an invoice.paid event to every enabled webhook endpoint across all tenants.
pub async fn broadcast_invoice_webhook(
    state: &AppState,
    invoice: &Invoice,
    transaction: &Transaction,
) -> Result<(), AppError> {
    let subscribers = webhook_endpoints::list_enabled_webhook_urls(state.db.pool()).await?;
    if subscribers.is_empty() {
        tracing::debug!(invoice_id = %invoice.id, "no webhook subscribers");
        return Ok(());
    }

    let payload = InvoiceWebhookPayload {
        event: "invoice.paid",
        invoice_id: invoice.id,
        system_id: invoice.system_id,
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

    let mut handles = Vec::new();
    for (subscriber_id, url) in subscribers {
        let state = state.clone();
        let body = body.clone();
        let transaction_id = transaction.id;
        let invoice_id = invoice.id;
        handles.push(tokio::spawn(async move {
            if let Err(e) =
                dispatch_webhook_body(&state, &url, transaction_id, "invoice.paid", &body).await
            {
                tracing::error!(
                    invoice_id = %invoice_id,
                    subscriber_system_id = %subscriber_id,
                    error = %e,
                    "invoice webhook fan-out failed for subscriber"
                );
                return false;
            }
            true
        }));
    }

    let mut ok = 0usize;
    let mut err = 0usize;
    for handle in handles {
        match handle.await {
            Ok(true) => ok += 1,
            Ok(false) => err += 1,
            Err(e) => {
                err += 1;
                tracing::error!(error = %e, "invoice webhook fan-out task join failed");
            }
        }
    }

    tracing::info!(
        invoice_id = %invoice.id,
        delivered_or_enqueued = ok,
        failed = err,
        "invoice webhook fan-out complete"
    );
    Ok(())
}

/// Enqueue to Redis/BullMQ when available; otherwise deliver in-process.
pub async fn dispatch_payment_webhook(
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

    dispatch_webhook_body(
        state,
        url,
        transaction.id,
        "payment.status_changed",
        &body,
    )
    .await
}

pub async fn dispatch_invoice_webhook(
    state: &AppState,
    url: &str,
    invoice: &Invoice,
    transaction: &Transaction,
) -> Result<(), AppError> {
    let payload = InvoiceWebhookPayload {
        event: "invoice.paid",
        invoice_id: invoice.id,
        system_id: invoice.system_id,
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

    dispatch_webhook_body(state, url, transaction.id, "invoice.paid", &body).await
}

async fn dispatch_webhook_body(
    state: &AppState,
    url: &str,
    transaction_id: Uuid,
    event_type: &str,
    body: &str,
) -> Result<(), AppError> {
    let signature = sign_payload(&state.config.webhook_signing_secret, body);

    if let Some(redis) = &state.redis {
        let job = WebhookJob {
            job_id: Uuid::new_v4().to_string(),
            url: url.to_string(),
            body: body.to_string(),
            signature: format!("sha256={signature}"),
            transaction_id,
            event_type: event_type.to_string(),
        };
        let mut conn = redis.clone();
        queue::enqueue_webhook(&mut conn, &job).await?;
        tracing::info!(
            transaction_id = %transaction_id,
            event_type,
            url,
            "webhook job enqueued to Redis"
        );
        return Ok(());
    }

    deliver_with_retries(state, url, transaction_id, event_type, body).await
}

pub async fn deliver_payment_webhook(
    state: &AppState,
    url: &str,
    transaction: &Transaction,
) -> Result<(), AppError> {
    dispatch_payment_webhook(state, url, transaction).await
}

pub async fn deliver_invoice_webhook(
    state: &AppState,
    url: &str,
    invoice: &Invoice,
    transaction: &Transaction,
) -> Result<(), AppError> {
    dispatch_invoice_webhook(state, url, invoice, transaction).await
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
