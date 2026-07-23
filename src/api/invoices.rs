use axum::{
    extract::{Path, Query, State},
    Extension, Json,
};
use chrono::{Duration, Utc};
use uuid::Uuid;

use crate::api::routes::AuthenticatedSystem;
use crate::auth::hash_request_body;
use crate::db::invoices::{
    cancel_invoice, create_deposit_with_credit, create_invoice, get_invoice_by_id,
    get_invoice_by_reference, list_invoices, mark_invoice_paid,
};
use crate::db::queries::{self, NewTransaction};
use crate::error::AppError;
use crate::gateway::GatewayDepositRequest;
use crate::models::{
    CollectInvoiceRequest, CreateInvoiceRequest, Invoice, InvoiceResponse, PaymentMethod,
    ProcessPaymentResponse, Transaction,
};
use crate::qr::generate_qr_png_base64;
use crate::webhook::sender;
use crate::AppState;

#[derive(serde::Deserialize)]
pub struct ListInvoicesQuery {
    pub status: Option<String>,
    pub limit: Option<i64>,
}

pub async fn collect_invoice_internal(
    state: &AppState,
    system_id: Uuid,
    invoice: Invoice,
    payment_method: PaymentMethod,
    idempotency_key: &str,
) -> Result<Transaction, AppError> {
    if invoice.status == "paid" {
        if let Some(tx_id) = invoice.transaction_id {
            return queries::get_transaction_by_id(state.db.pool(), tx_id, system_id).await;
        }
        return Err(AppError::Conflict("invoice already paid".into()));
    }

    if invoice.status == "expired" || invoice.status == "cancelled" {
        return Err(AppError::InvoiceInvalid(format!(
            "invoice is {}",
            invoice.status
        )));
    }

    if invoice.expires_at < Utc::now() {
        return Err(AppError::InvoiceInvalid("invoice is expired".into()));
    }

    if let Some(existing) =
        queries::get_transaction_by_idempotency(state.db.pool(), system_id, idempotency_key).await?
    {
        return Ok(existing);
    }

    let wallet = queries::get_or_create_wallet(
        state.db.pool(),
        system_id,
        &invoice.country,
        &invoice.currency,
    )
    .await?;

    let deposit_id = Uuid::new_v4();
    let gateway_result = state
        .gateway
        .process_deposit(GatewayDepositRequest {
            deposit_id,
            amount: invoice.amount,
            currency: invoice.currency.clone(),
            payment_method: payment_method.clone(),
            client_reference: Some(invoice.reference.clone()),
        })
        .await?;

    if !gateway_result.success {
        return Err(AppError::Gateway(
            gateway_result
                .error
                .unwrap_or_else(|| "deposit failed".into()),
        ));
    }

    let request_hash = hash_request_body(&format!("collect-{idempotency_key}"));
    let tx = create_deposit_with_credit(
        state.db.pool(),
        wallet.id,
        invoice.amount,
        NewTransaction {
            system_id,
            wallet_id: wallet.id,
            external_id: &invoice.reference,
            idempotency_key,
            request_hash: &request_hash,
            amount: invoice.amount,
            currency: &invoice.currency,
            country: &invoice.country,
            status: "completed",
            gateway: &state.config.fallback_gateway,
            gateway_reference: gateway_result.reference.as_deref(),
            gateway_status: Some(&gateway_result.status),
            error: None,
            invoice_id: Some(invoice.id),
            direction: "deposit",
        },
    )
    .await?;

    mark_invoice_paid(state.db.pool(), invoice.id, tx.id).await?;

    tracing::info!(
        system_id = %system_id,
        invoice_id = %invoice.id,
        payment_id = %tx.id,
        "invoice collected"
    );

    let webhook_state = state.clone();
    let inv = invoice.clone();
    let tx_clone = tx.clone();
    tokio::spawn(async move {
        if let Err(e) = sender::broadcast_invoice_webhook(&webhook_state, &inv, &tx_clone).await {
            tracing::error!(
                invoice_id = %inv.id,
                payment_id = %tx_clone.id,
                error = %e,
                "invoice webhook broadcast failed"
            );
        }
    });

    Ok(tx)
}

pub async fn create_invoice_handler(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthenticatedSystem>,
    Json(req): Json<CreateInvoiceRequest>,
) -> Result<Json<InvoiceResponse>, AppError> {
    if req.amount <= 0 {
        return Err(AppError::Validation("amount must be positive".into()));
    }
    if !auth.system.enabled_countries.contains(&req.country) {
        return Err(AppError::CountryNotEnabled);
    }
    if let Err(msg) = crate::catalog::validate_country_currency(&req.country, &req.currency) {
        return Err(AppError::Validation(msg));
    }

    let reference = format!(
        "INV_{}_{}",
        auth.system.prefix,
        &Uuid::new_v4().to_string()[..8].to_uppercase()
    );
    let qr_url = format!(
        "{}/pay/{}",
        state.config.invoice_pay_base_url.trim_end_matches('/'),
        reference
    );
    let expires_at = Utc::now() + Duration::hours(req.expires_in_hours);
    let qr_code = generate_qr_png_base64(&qr_url)?;

    let invoice = create_invoice(
        state.db.pool(),
        auth.system.id,
        &reference,
        req.description.as_deref(),
        req.amount,
        &req.currency,
        &req.country,
        expires_at,
        &qr_url,
    )
    .await?;

    tracing::info!(
        system_id = %auth.system.id,
        invoice_id = %invoice.id,
        reference = %invoice.reference,
        "invoice created"
    );

    Ok(Json(invoice.to_response(qr_code)))
}

pub async fn list_invoices_handler(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthenticatedSystem>,
    Query(q): Query<ListInvoicesQuery>,
) -> Result<Json<Vec<InvoiceResponse>>, AppError> {
    let limit = q.limit.unwrap_or(50).min(100);
    let invoices = list_invoices(
        state.db.pool(),
        auth.system.id,
        q.status.as_deref(),
        limit,
    )
    .await?;

    let mut out = Vec::new();
    for inv in invoices {
        let qr = generate_qr_png_base64(&inv.qr_payload_url)?;
        out.push(inv.to_response(qr));
    }
    Ok(Json(out))
}

pub async fn get_invoice_by_reference_handler(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthenticatedSystem>,
    Path(reference): Path<String>,
) -> Result<Json<InvoiceResponse>, AppError> {
    let invoice = get_invoice_by_reference(state.db.pool(), &reference, auth.system.id).await?;
    let qr = generate_qr_png_base64(&invoice.qr_payload_url)?;
    Ok(Json(invoice.to_response(qr)))
}

pub async fn collect_invoice(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthenticatedSystem>,
    Path(id): Path<Uuid>,
    Json(req): Json<CollectInvoiceRequest>,
) -> Result<Json<ProcessPaymentResponse>, AppError> {
    let invoice = get_invoice_by_id(state.db.pool(), id, auth.system.id).await?;
    let tx = collect_invoice_internal(
        &state,
        auth.system.id,
        invoice,
        req.payment_method,
        &req.idempotency_key,
    )
    .await?;
    Ok(Json(tx.into()))
}

pub async fn cancel_invoice_handler(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthenticatedSystem>,
    Path(id): Path<Uuid>,
) -> Result<Json<InvoiceResponse>, AppError> {
    let invoice = cancel_invoice(state.db.pool(), id, auth.system.id).await?;
    let qr = generate_qr_png_base64(&invoice.qr_payload_url)?;
    Ok(Json(invoice.to_response(qr)))
}
