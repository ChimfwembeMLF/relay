use axum::{
    extract::{Path, Query, State},
    Extension, Json,
};
use serde::Deserialize;
use uuid::Uuid;

use crate::api::routes::{require_system_access, AuthenticatedSystem};
use crate::auth::hash_request_body;
use crate::db::queries::{self, NewTransaction};
use crate::error::AppError;
use crate::gateway::GatewayPaymentRequest;
use crate::models::{
    external_id_format_valid, ProcessPaymentRequest, ProcessPaymentResponse, Transaction,
};
use crate::webhook::sender;
use crate::AppState;

#[derive(Deserialize)]
pub struct TransactionListQuery {
    pub external_id: Option<String>,
    pub limit: Option<i64>,
}

pub async fn process_payment(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthenticatedSystem>,
    Json(req): Json<ProcessPaymentRequest>,
) -> Result<Json<ProcessPaymentResponse>, AppError> {
    require_system_access(&auth, req.system_id)?;
    validate_payment_request(&req)?;

    if !auth.system.enabled_countries.contains(&req.country) {
        return Err(AppError::CountryNotEnabled);
    }

    if !external_id_format_valid(&auth.system.prefix, &req.external_id) {
        tracing::warn!(
            system_id = %req.system_id,
            external_id = %req.external_id,
            "external_id does not match recommended format"
        );
    }

    let body = serde_json::to_string(&req)
        .map_err(|e| AppError::Internal(format!("failed to serialize request: {e}")))?;
    let request_hash = hash_request_body(&body);

    if let Some(existing) =
        queries::get_transaction_by_idempotency(state.db.pool(), req.system_id, &req.idempotency_key)
            .await?
    {
        if existing.request_hash != request_hash {
            return Err(AppError::Conflict(
                "idempotency key reused with different request body".into(),
            ));
        }
        return Ok(Json(existing.into()));
    }

    let wallet = queries::get_or_create_wallet(
        state.db.pool(),
        req.system_id,
        &req.country,
        &req.currency,
    )
    .await?;

    if wallet.balance < req.amount {
        return Err(AppError::InsufficientBalance);
    }

    let payout_id = Uuid::new_v4();
    let gateway_result = state
        .gateway
        .process_payment(GatewayPaymentRequest {
            payout_id,
            amount: req.amount,
            currency: req.currency.clone(),
            payment_method: req.payment_method.clone(),
        })
        .await?;

    let status = if gateway_result.success {
        "completed"
    } else {
        "failed"
    };

    let transaction = if gateway_result.success {
        queries::create_transaction_with_debit(
            state.db.pool(),
            wallet.id,
            req.amount,
            NewTransaction {
                system_id: req.system_id,
                wallet_id: wallet.id,
                external_id: &req.external_id,
                idempotency_key: &req.idempotency_key,
                request_hash: &request_hash,
                amount: req.amount,
                currency: &req.currency,
                country: &req.country,
                status,
                gateway: &state.config.fallback_gateway,
                gateway_reference: gateway_result.reference.as_deref(),
                gateway_status: Some(&gateway_result.status),
                error: gateway_result.error.as_deref(),
                invoice_id: None,
                direction: "payout",
                batch_id: None,
                refund_id: None,
            },
        )
        .await?
    } else {
        let tx = NewTransaction {
            system_id: req.system_id,
            wallet_id: wallet.id,
            external_id: &req.external_id,
            idempotency_key: &req.idempotency_key,
            request_hash: &request_hash,
            amount: req.amount,
            currency: &req.currency,
            country: &req.country,
            status,
            gateway: &state.config.fallback_gateway,
            gateway_reference: gateway_result.reference.as_deref(),
            gateway_status: Some(&gateway_result.status),
            error: gateway_result.error.as_deref(),
            invoice_id: None,
            direction: "payout",
            batch_id: None,
            refund_id: None,
        };
        insert_failed_transaction(state.db.pool(), &tx).await?
    };

    tracing::info!(
        system_id = %req.system_id,
        external_id = %req.external_id,
        idempotency_key = %req.idempotency_key,
        payment_id = %transaction.id,
        status = %transaction.status,
        "payment processed"
    );

    if transaction.status == "completed" || transaction.status == "failed" {
        let webhook_state = state.clone();
        let tx = transaction.clone();
        tokio::spawn(async move {
            if let Err(e) = sender::broadcast_payment_webhook(&webhook_state, &tx).await {
                tracing::error!(payment_id = %tx.id, error = %e, "webhook broadcast failed");
            }
        });
    }

    Ok(Json(transaction.into()))
}

async fn insert_failed_transaction(
    pool: &sqlx::PgPool,
    tx: &NewTransaction<'_>,
) -> Result<Transaction, AppError> {
    sqlx::query_as::<_, Transaction>(
        r#"
        INSERT INTO transactions (
            system_id, wallet_id, external_id, idempotency_key, request_hash,
            amount, currency, country, status, gateway, gateway_reference,
            gateway_status, error, invoice_id, direction, batch_id, refund_id
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17)
        RETURNING id, system_id, wallet_id, external_id, idempotency_key, request_hash,
                  amount, currency, country, status, gateway, gateway_reference,
                  gateway_status, error, invoice_id, direction, batch_id, refund_id, created_at, updated_at
        "#,
    )
    .bind(tx.system_id)
    .bind(tx.wallet_id)
    .bind(tx.external_id)
    .bind(tx.idempotency_key)
    .bind(tx.request_hash)
    .bind(tx.amount)
    .bind(tx.currency)
    .bind(tx.country)
    .bind(tx.status)
    .bind(tx.gateway)
    .bind(tx.gateway_reference)
    .bind(tx.gateway_status)
    .bind(tx.error)
    .bind(tx.invoice_id)
    .bind(tx.direction)
    .bind(tx.batch_id)
    .bind(tx.refund_id)
    .fetch_one(pool)
    .await
    .map_err(AppError::from)
}

pub async fn get_payment(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthenticatedSystem>,
    Path(id): Path<Uuid>,
) -> Result<Json<ProcessPaymentResponse>, AppError> {
    let transaction = queries::get_transaction_by_id(state.db.pool(), id, auth.system.id).await?;
    Ok(Json(transaction.into()))
}

pub async fn list_transactions(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthenticatedSystem>,
    Path(system_id): Path<Uuid>,
    Query(query): Query<TransactionListQuery>,
) -> Result<Json<Vec<Transaction>>, AppError> {
    require_system_access(&auth, system_id)?;
    let limit = query.limit.unwrap_or(50).min(100);
    let transactions = queries::list_transactions_by_system(
        state.db.pool(),
        system_id,
        query.external_id.as_deref(),
        limit,
    )
    .await?;
    Ok(Json(transactions))
}

fn validate_payment_request(req: &ProcessPaymentRequest) -> Result<(), AppError> {
    if req.amount <= 0 {
        return Err(AppError::Validation("amount must be positive".into()));
    }
    if req.external_id.trim().is_empty() {
        return Err(AppError::Validation("external_id is required".into()));
    }
    if req.idempotency_key.trim().is_empty() {
        return Err(AppError::Validation("idempotency_key is required".into()));
    }
    if req.currency.len() != 3 || !req.currency.chars().all(|c| c.is_ascii_uppercase()) {
        return Err(AppError::Validation("currency must be a 3-letter ISO code".into()));
    }
    if req.country.len() != 2 || !req.country.chars().all(|c| c.is_ascii_uppercase()) {
        return Err(AppError::Validation("country must be a 2-letter ISO code".into()));
    }
    let provider = req
        .payment_method
        .details
        .get("provider")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    if let Err(msg) =
        crate::catalog::validate_country_currency_provider(&req.country, &req.currency, provider)
    {
        return Err(AppError::Validation(msg));
    }
    Ok(())
}
