use axum::{
    extract::{Path, State},
    http::{header, StatusCode},
    response::{IntoResponse, Json, Response},
};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::db::invoices::get_invoice_by_reference_public;
use crate::error::AppError;
use crate::models::{Invoice, PaymentMethod};
use crate::pay;
use crate::AppState;

use super::invoices::collect_invoice_internal;

#[derive(Serialize)]
pub struct ProviderOption {
    pub value: String,
    pub label: String,
}

#[derive(Serialize)]
pub struct PayPageApiResponse {
    pub reference: String,
    pub amount: i64,
    pub amount_display: String,
    pub currency: String,
    pub country: String,
    pub description: Option<String>,
    pub status: String,
    pub expires_at: String,
    pub payable: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub form_token: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub idempotency_key: Option<String>,
    pub providers: Vec<ProviderOption>,
}

#[derive(Serialize)]
pub struct PaySubmitApiResponse {
    pub status: String,
    pub message: String,
    pub amount_display: String,
    pub reference: String,
}

#[derive(Serialize)]
struct PayErrorResponse {
    error: String,
    message: String,
}

#[derive(Deserialize)]
pub struct PaySubmitRequest {
    pub phone: String,
    pub provider: String,
    pub idempotency_key: String,
    pub form_token: String,
}

pub async fn get_pay_api(
    State(state): State<AppState>,
    Path(reference): Path<String>,
) -> Response {
    match get_invoice_by_reference_public(state.db.pool(), &reference).await {
        Ok(invoice) => Json(build_pay_response(&state, &invoice)).into_response(),
        Err(AppError::NotFound) => pay_error(
            StatusCode::NOT_FOUND,
            "not_found",
            "The payment link is invalid or has expired.",
        ),
        Err(e) => {
            tracing::error!(error = %e, reference = %reference, "pay api load failed");
            pay_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "load_failed",
                "Could not load invoice.",
            )
        }
    }
}

pub async fn submit_pay_api(
    State(state): State<AppState>,
    Path(reference): Path<String>,
    Json(body): Json<PaySubmitRequest>,
) -> Response {
    let rate_key = format!("pay:{reference}");
    if !state.pay_rate_limiter.check(&rate_key) {
        return pay_error(
            StatusCode::TOO_MANY_REQUESTS,
            "rate_limited",
            "Too many attempts. Please wait and try again.",
        );
    }

    let invoice = match get_invoice_by_reference_public(state.db.pool(), &reference).await {
        Ok(inv) => inv,
        Err(AppError::NotFound) => {
            return pay_error(
                StatusCode::NOT_FOUND,
                "not_found",
                "The payment link is invalid or has expired.",
            );
        }
        Err(e) => {
            tracing::error!(error = %e, reference = %reference, "pay api submit lookup failed");
            return pay_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "load_failed",
                "Could not load invoice.",
            );
        }
    };

    if !pay::verify_form_token(
        &state.config.webhook_signing_secret,
        &invoice.reference,
        &invoice.expires_at,
        &body.form_token,
    ) {
        return pay_error(
            StatusCode::BAD_REQUEST,
            "invalid_form",
            "Invalid or expired form. Refresh the page and try again.",
        );
    }

    if invoice.status == "paid" {
        return Json(PaySubmitApiResponse {
            status: "paid".into(),
            message: "Payment successful".into(),
            amount_display: pay::format_amount(invoice.amount, &invoice.currency),
            reference: invoice.reference,
        })
        .into_response();
    }

    if invoice.status == "expired" || invoice.status == "cancelled" {
        return pay_error(
            StatusCode::PAYMENT_REQUIRED,
            "invoice_invalid",
            &format!("Invoice is {}", invoice.status),
        );
    }

    if body.phone.trim().is_empty() || body.provider.trim().is_empty() {
        return pay_error(
            StatusCode::BAD_REQUEST,
            "validation_error",
            "Phone number and provider are required.",
        );
    }

    let payment_method = PaymentMethod {
        method_type: "mmo".into(),
        details: serde_json::json!({
            "provider": body.provider,
            "phoneNumber": body.phone.trim(),
        }),
    };

    match collect_invoice_internal(
        &state,
        invoice.system_id,
        invoice.clone(),
        payment_method,
        &body.idempotency_key,
    )
    .await
    {
        Ok(_) => {
            let updated = get_invoice_by_reference_public(state.db.pool(), &reference)
                .await
                .unwrap_or(invoice);
            Json(PaySubmitApiResponse {
                status: "paid".into(),
                message: "Payment successful".into(),
                amount_display: pay::format_amount(updated.amount, &updated.currency),
                reference: updated.reference,
            })
            .into_response()
        }
        Err(AppError::InvoiceInvalid(msg)) => {
            pay_error(StatusCode::PAYMENT_REQUIRED, "invoice_invalid", &msg)
        }
        Err(AppError::Gateway(msg)) => pay_error(StatusCode::BAD_GATEWAY, "gateway_error", &msg),
        Err(AppError::Conflict(msg)) => pay_error(StatusCode::CONFLICT, "conflict", &msg),
        Err(e) => {
            tracing::error!(error = %e, reference = %reference, "pay api collect failed");
            pay_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "internal_error",
                "Payment could not be processed. Try again.",
            )
        }
    }
}

pub async fn serve_spa() -> Response {
    match std::fs::read_to_string("frontend/dist/index.html") {
        Ok(html) => (
            StatusCode::OK,
            [(header::CONTENT_TYPE, "text/html; charset=utf-8")],
            html,
        )
            .into_response(),
        Err(_) => (
            StatusCode::SERVICE_UNAVAILABLE,
            [(header::CONTENT_TYPE, "text/plain; charset=utf-8")],
            "Frontend not built. Run `cargo build` (builds frontend/), or unset SKIP_FRONTEND_BUILD.",
        )
            .into_response(),
    }
}

fn build_pay_response(state: &AppState, invoice: &Invoice) -> PayPageApiResponse {
    let providers = pay::providers_for_country(&invoice.country)
        .iter()
        .map(|(value, label)| ProviderOption {
            value: (*value).into(),
            label: (*label).into(),
        })
        .collect();

    let payable = invoice.status == "open" && invoice.expires_at >= Utc::now();
    let (form_token, idempotency_key) = if payable {
        (
            Some(pay::generate_form_token(
                &state.config.webhook_signing_secret,
                &invoice.reference,
                &invoice.expires_at,
            )),
            Some(Uuid::new_v4().to_string()),
        )
    } else {
        (None, None)
    };

    PayPageApiResponse {
        reference: invoice.reference.clone(),
        amount: invoice.amount,
        amount_display: pay::format_amount(invoice.amount, &invoice.currency),
        currency: invoice.currency.clone(),
        country: invoice.country.clone(),
        description: invoice.description.clone(),
        status: invoice.status.clone(),
        expires_at: invoice.expires_at.to_rfc3339(),
        payable,
        form_token,
        idempotency_key,
        providers,
    }
}

fn pay_error(status: StatusCode, error: &str, message: &str) -> Response {
    (
        status,
        Json(PayErrorResponse {
            error: error.into(),
            message: message.into(),
        }),
    )
        .into_response()
}
