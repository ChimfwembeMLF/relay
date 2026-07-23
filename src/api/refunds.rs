use axum::{
    extract::{Path, State},
    Extension, Json,
};
use uuid::Uuid;

use crate::api::routes::AuthenticatedSystem;
use crate::auth::hash_request_body;
use crate::db::batches as batch_db;
use crate::db::invoices::{apply_refund_amount, get_invoice_by_id};
use crate::db::queries::{self, NewTransaction};
use crate::error::AppError;
use crate::gateway::GatewayPaymentRequest;
use crate::models::{
    CreateRefundRequest, InvoiceRefundSummary, PaymentMethod, RefundResponse,
};
use crate::webhook::sender;
use crate::AppState;

pub async fn create_refund(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthenticatedSystem>,
    Path(invoice_id): Path<Uuid>,
    Json(req): Json<CreateRefundRequest>,
) -> Result<Json<RefundResponse>, AppError> {
    if req.idempotency_key.trim().is_empty() {
        return Err(AppError::Validation("idempotency_key is required".into()));
    }
    if req.amount <= 0 {
        return Err(AppError::Validation("amount must be positive".into()));
    }

    let invoice = get_invoice_by_id(state.db.pool(), invoice_id, auth.system.id).await?;
    if invoice.status != "paid" {
        return Err(AppError::Validation(format!(
            "invoice is not refundable (status {})",
            invoice.status
        )));
    }
    if req.amount > invoice.remaining_refundable() {
        return Err(AppError::Validation(
            "refund amount exceeds remaining refundable balance".into(),
        ));
    }

    let phone = req
        .phone
        .as_deref()
        .or(invoice.payer_phone.as_deref())
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .ok_or_else(|| AppError::Validation("phone is required for refund".into()))?
        .to_string();
    let provider = req
        .provider
        .as_deref()
        .or(invoice.payer_provider.as_deref())
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .ok_or_else(|| AppError::Validation("provider is required for refund".into()))?
        .to_string();

    if let Err(msg) = crate::catalog::validate_country_currency_provider(
        &invoice.country,
        &invoice.currency,
        &provider,
    ) {
        return Err(AppError::Validation(msg));
    }

    let body = serde_json::to_string(&req)
        .map_err(|e| AppError::Internal(format!("serialize refund: {e}")))?;
    let request_hash = hash_request_body(&body);

    if let Some(existing) = batch_db::get_refund_by_idempotency(
        state.db.pool(),
        auth.system.id,
        &req.idempotency_key,
    )
    .await?
    {
        if existing.request_hash != request_hash {
            return Err(AppError::Conflict(
                "idempotency key reused with different request body".into(),
            ));
        }
        let inv = get_invoice_by_id(state.db.pool(), invoice_id, auth.system.id).await?;
        return Ok(Json(RefundResponse {
            id: existing.id,
            invoice_id: existing.invoice_id,
            amount: existing.amount,
            status: existing.status,
            transaction_id: existing.transaction_id,
            invoice: InvoiceRefundSummary {
                refunded_amount: inv.refunded_amount,
                remaining_refundable: inv.remaining_refundable(),
                fully_refunded: inv.fully_refunded(),
                status: inv.status,
            },
        }));
    }

    let wallet = queries::get_or_create_wallet(
        state.db.pool(),
        auth.system.id,
        &invoice.country,
        &invoice.currency,
    )
    .await?;
    if wallet.balance < req.amount {
        return Err(AppError::InsufficientBalance);
    }

    // Reserve refund capacity first
    apply_refund_amount(state.db.pool(), invoice.id, req.amount).await?;

    let refund = batch_db::insert_refund(
        state.db.pool(),
        auth.system.id,
        invoice.id,
        req.amount,
        &invoice.currency,
        &invoice.country,
        &phone,
        &provider,
        &req.idempotency_key,
        &request_hash,
        "completed",
        None,
        None,
    )
    .await?;

    let payment_method = PaymentMethod {
        method_type: "mmo".into(),
        details: serde_json::json!({ "phone": phone, "provider": provider }),
    };

    let payout_id = Uuid::new_v4();
    let gateway_result = state
        .gateway
        .process_payment(GatewayPaymentRequest {
            payout_id,
            amount: req.amount,
            currency: invoice.currency.clone(),
            payment_method,
        })
        .await;

    let gateway_result = match gateway_result {
        Ok(r) if r.success => r,
        Ok(r) => {
            // roll back refunded_amount
            let _ = sqlx::query(
                "UPDATE invoices SET refunded_amount = refunded_amount - $2 WHERE id = $1",
            )
            .bind(invoice.id)
            .bind(req.amount)
            .execute(state.db.pool())
            .await;
            return Err(AppError::Gateway(
                r.error.unwrap_or_else(|| "refund payout failed".into()),
            ));
        }
        Err(e) => {
            let _ = sqlx::query(
                "UPDATE invoices SET refunded_amount = refunded_amount - $2 WHERE id = $1",
            )
            .bind(invoice.id)
            .bind(req.amount)
            .execute(state.db.pool())
            .await;
            return Err(e);
        }
    };

    let external_id = format!("REFUND_{}", invoice.reference);
    let tx = queries::create_transaction_with_debit(
        state.db.pool(),
        wallet.id,
        req.amount,
        NewTransaction {
            system_id: auth.system.id,
            wallet_id: wallet.id,
            external_id: &external_id,
            idempotency_key: &req.idempotency_key,
            request_hash: &request_hash,
            amount: req.amount,
            currency: &invoice.currency,
            country: &invoice.country,
            status: "completed",
            gateway: &state.config.fallback_gateway,
            gateway_reference: gateway_result.reference.as_deref(),
            gateway_status: Some(&gateway_result.status),
            error: None,
            invoice_id: Some(invoice.id),
            direction: "payout",
            batch_id: None,
            refund_id: Some(refund.id),
        },
    )
    .await?;

    batch_db::update_refund_transaction(state.db.pool(), refund.id, tx.id).await?;

    let webhook_state = state.clone();
    let tx_clone = tx.clone();
    tokio::spawn(async move {
        if let Err(e) = sender::broadcast_payment_webhook(&webhook_state, &tx_clone).await {
            tracing::error!(payment_id = %tx_clone.id, error = %e, "webhook broadcast failed");
        }
    });

    let inv = get_invoice_by_id(state.db.pool(), invoice_id, auth.system.id).await?;

    Ok(Json(RefundResponse {
        id: refund.id,
        invoice_id: invoice.id,
        amount: req.amount,
        status: "completed".into(),
        transaction_id: Some(tx.id),
        invoice: InvoiceRefundSummary {
            refunded_amount: inv.refunded_amount,
            remaining_refundable: inv.remaining_refundable(),
            fully_refunded: inv.fully_refunded(),
            status: inv.status,
        },
    }))
}
