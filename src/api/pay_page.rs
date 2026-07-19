use axum::{
    extract::{Path, State},
    http::{header, StatusCode},
    response::{Html, IntoResponse, Response},
    Form,
};
use serde::Deserialize;
use uuid::Uuid;

use crate::db::invoices::get_invoice_by_reference_public;
use crate::error::AppError;
use crate::models::{Invoice, PaymentMethod};
use crate::pay;
use crate::AppState;

use super::invoices::collect_invoice_internal;

#[derive(Deserialize)]
pub struct PayForm {
    pub phone: String,
    pub provider: String,
    pub idempotency_key: String,
    pub form_token: String,
}

pub async fn show_pay_page(
    State(state): State<AppState>,
    Path(reference): Path<String>,
) -> Response {
    match get_invoice_by_reference_public(state.db.pool(), &reference).await {
        Ok(invoice) => render_invoice_page(&state, &invoice),
        Err(AppError::NotFound) => html(StatusCode::NOT_FOUND, pay::render_not_found()),
        Err(e) => {
            tracing::error!(error = %e, reference = %reference, "pay page load failed");
            html(StatusCode::INTERNAL_SERVER_ERROR, pay::render_not_found())
        }
    }
}

pub async fn submit_pay_page(
    State(state): State<AppState>,
    Path(reference): Path<String>,
    Form(form): Form<PayForm>,
) -> Response {
    let rate_key = format!("pay:{reference}");
    if !state.pay_rate_limiter.check(&rate_key) {
        return html(
            StatusCode::TOO_MANY_REQUESTS,
            pay::render_error(&reference, "Too many attempts. Please wait and try again."),
        );
    }

    let invoice = match get_invoice_by_reference_public(state.db.pool(), &reference).await {
        Ok(inv) => inv,
        Err(AppError::NotFound) => {
            return html(StatusCode::NOT_FOUND, pay::render_not_found());
        }
        Err(e) => {
            tracing::error!(error = %e, reference = %reference, "pay page submit lookup failed");
            return html(StatusCode::INTERNAL_SERVER_ERROR, pay::render_not_found());
        }
    };

    if !pay::verify_form_token(
        &state.config.webhook_signing_secret,
        &invoice.reference,
        &invoice.expires_at,
        &form.form_token,
    ) {
        return html(
            StatusCode::BAD_REQUEST,
            pay::render_error(&reference, "Invalid or expired form. Refresh the page and try again."),
        );
    }

    if invoice.status == "paid" {
        return html(StatusCode::OK, pay::render_paid(&invoice));
    }

    if invoice.status == "expired" || invoice.status == "cancelled" {
        return html(
            StatusCode::PAYMENT_REQUIRED,
            if invoice.status == "cancelled" {
                pay::render_cancelled()
            } else {
                pay::render_expired()
            },
        );
    }

    if form.phone.trim().is_empty() || form.provider.trim().is_empty() {
        return html(
            StatusCode::BAD_REQUEST,
            pay::render_error(&reference, "Phone number and provider are required."),
        );
    }

    let payment_method = PaymentMethod {
        method_type: "mmo".into(),
        details: serde_json::json!({
            "provider": form.provider,
            "phoneNumber": form.phone.trim(),
        }),
    };

    match collect_invoice_internal(
        &state,
        invoice.system_id,
        invoice.clone(),
        payment_method,
        &form.idempotency_key,
    )
    .await
    {
        Ok(_tx) => {
            let updated = get_invoice_by_reference_public(state.db.pool(), &reference)
                .await
                .unwrap_or(invoice);
            html(StatusCode::OK, pay::render_success(&updated))
        }
        Err(AppError::InvoiceInvalid(msg)) => html(
            StatusCode::PAYMENT_REQUIRED,
            pay::render_error(&reference, &msg),
        ),
        Err(AppError::Gateway(msg)) => {
            html(StatusCode::OK, pay::render_error(&reference, &msg))
        }
        Err(AppError::Conflict(msg)) => {
            html(StatusCode::CONFLICT, pay::render_error(&reference, &msg))
        }
        Err(e) => {
            tracing::error!(error = %e, reference = %reference, "pay page collect failed");
            html(
                StatusCode::INTERNAL_SERVER_ERROR,
                pay::render_error(&reference, "Payment could not be processed. Try again."),
            )
        }
    }
}

fn render_invoice_page(state: &AppState, invoice: &Invoice) -> Response {
    match invoice.status.as_str() {
        "paid" => html(StatusCode::OK, pay::render_paid(invoice)),
        "expired" => html(StatusCode::OK, pay::render_expired()),
        "cancelled" => html(StatusCode::OK, pay::render_cancelled()),
        "open" => {
            if invoice.expires_at < chrono::Utc::now() {
                return html(StatusCode::OK, pay::render_expired());
            }
            let token = pay::generate_form_token(
                &state.config.webhook_signing_secret,
                &invoice.reference,
                &invoice.expires_at,
            );
            let idempotency_key = Uuid::new_v4();
            html(
                StatusCode::OK,
                pay::render_open(invoice, &token, &idempotency_key),
            )
        }
        _ => html(StatusCode::OK, pay::render_error(&invoice.reference, "Unknown invoice status.")),
    }
}

fn html(status: StatusCode, body: String) -> Response {
    (status, [(header::CONTENT_TYPE, "text/html; charset=utf-8")], Html(body)).into_response()
}
