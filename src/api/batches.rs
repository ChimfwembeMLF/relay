use axum::{
    extract::{Path, State},
    Extension, Json,
};
use uuid::Uuid;

use crate::api::routes::{require_system_access, AuthenticatedSystem};
use crate::auth::hash_request_body;
use crate::db::batches as batch_db;
use crate::db::queries::{self, NewTransaction};
use crate::error::AppError;
use crate::gateway::GatewayPaymentRequest;
use crate::models::{
    BatchLineRequest, BatchLineResponse, BatchResponse, CreateBatchRequest, PaymentMethod,
    PayoutBatchLine,
};
use crate::webhook::sender;
use crate::AppState;

const MAX_BATCH_LINES: usize = 100;

struct StagedLine {
    line_index: i32,
    external_id: String,
    amount: i64,
    currency: String,
    country: String,
    phone: String,
    provider: String,
    status: String,
    error: Option<String>,
    transaction_id: Option<Uuid>,
    line_idempotency_key: String,
}

pub async fn create_batch(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthenticatedSystem>,
    Json(req): Json<CreateBatchRequest>,
) -> Result<Json<BatchResponse>, AppError> {
    require_system_access(&auth, req.system_id)?;

    if req.idempotency_key.trim().is_empty() {
        return Err(AppError::Validation("idempotency_key is required".into()));
    }
    if req.lines.is_empty() {
        return Err(AppError::Validation("lines must not be empty".into()));
    }
    if req.lines.len() > MAX_BATCH_LINES {
        return Err(AppError::Validation(format!(
            "batch may contain at most {MAX_BATCH_LINES} lines"
        )));
    }

    let body = serde_json::to_string(&req)
        .map_err(|e| AppError::Internal(format!("failed to serialize batch: {e}")))?;
    let request_hash = hash_request_body(&body);

    if let Some(existing) =
        batch_db::get_batch_by_idempotency(state.db.pool(), req.system_id, &req.idempotency_key)
            .await?
    {
        if existing.request_hash != request_hash {
            return Err(AppError::Conflict(
                "idempotency key reused with different request body".into(),
            ));
        }
        return Ok(Json(batch_to_response(state.db.pool(), existing).await?));
    }

    let mut success = 0i32;
    let mut failure = 0i32;
    let mut staged: Vec<StagedLine> = Vec::new();

    for (idx, line) in req.lines.iter().enumerate() {
        let line_index = idx as i32;
        let line_key = format!("{}:{line_index}", req.idempotency_key);
        let phone = extract_phone(&line.payment_method);
        let provider = extract_provider(&line.payment_method);
        let external_id = line.external_id.clone().unwrap_or_else(|| {
            format!(
                "{}_{}_{}",
                auth.system.prefix,
                chrono::Utc::now().format("%Y%m%d"),
                &Uuid::new_v4().to_string()[..8]
            )
        });

        if phone.is_empty() || provider.is_empty() {
            failure += 1;
            staged.push(StagedLine {
                line_index,
                external_id,
                amount: line.amount.max(1),
                currency: line.currency.clone(),
                country: line.country.clone(),
                phone,
                provider,
                status: "failed".into(),
                error: Some("phone and provider are required".into()),
                transaction_id: None,
                line_idempotency_key: line_key,
            });
            continue;
        }

        match process_line(
            &state,
            &auth,
            req.system_id,
            line,
            &external_id,
            &line_key,
            &phone,
            &provider,
        )
        .await
        {
            Ok(tx_id) => {
                success += 1;
                staged.push(StagedLine {
                    line_index,
                    external_id,
                    amount: line.amount,
                    currency: line.currency.clone(),
                    country: line.country.clone(),
                    phone,
                    provider,
                    status: "completed".into(),
                    error: None,
                    transaction_id: Some(tx_id),
                    line_idempotency_key: line_key,
                });
            }
            Err(e) => {
                failure += 1;
                staged.push(StagedLine {
                    line_index,
                    external_id,
                    amount: line.amount.max(1),
                    currency: line.currency.clone(),
                    country: line.country.clone(),
                    phone,
                    provider,
                    status: "failed".into(),
                    error: Some(format_line_error(&e)),
                    transaction_id: None,
                    line_idempotency_key: line_key,
                });
            }
        }
    }

    let status = if success == 0 {
        "failed"
    } else if failure == 0 {
        "completed"
    } else {
        "partial"
    };

    let batch = batch_db::insert_batch(
        state.db.pool(),
        req.system_id,
        &req.idempotency_key,
        &request_hash,
        status,
        staged.len() as i32,
        success,
        failure,
    )
    .await?;

    for line in &staged {
        if let Some(tx_id) = line.transaction_id {
            sqlx::query("UPDATE transactions SET batch_id = $1 WHERE id = $2")
                .bind(batch.id)
                .bind(tx_id)
                .execute(state.db.pool())
                .await?;
        }
        batch_db::insert_batch_line(
            state.db.pool(),
            batch.id,
            line.line_index,
            &line.external_id,
            line.amount,
            &line.currency,
            &line.country,
            &line.phone,
            &line.provider,
            &line.status,
            line.error.as_deref(),
            line.transaction_id,
            &line.line_idempotency_key,
        )
        .await?;
    }

    Ok(Json(BatchResponse {
        id: batch.id,
        status: batch.status,
        success_count: batch.success_count,
        failure_count: batch.failure_count,
        lines: staged
            .into_iter()
            .map(|l| BatchLineResponse {
                line_index: l.line_index,
                status: l.status,
                transaction_id: l.transaction_id,
                error: l.error,
                external_id: l.external_id,
            })
            .collect(),
    }))
}

fn format_line_error(e: &AppError) -> String {
    match e {
        AppError::InsufficientBalance => "insufficient_balance".into(),
        AppError::CountryNotEnabled => "country_not_enabled".into(),
        AppError::Validation(m) => m.clone(),
        AppError::Gateway(m) => m.clone(),
        other => other.to_string(),
    }
}

async fn process_line(
    state: &AppState,
    auth: &AuthenticatedSystem,
    system_id: Uuid,
    line: &BatchLineRequest,
    external_id: &str,
    line_key: &str,
    phone: &str,
    provider: &str,
) -> Result<Uuid, AppError> {
    if line.amount <= 0 {
        return Err(AppError::Validation("amount must be positive".into()));
    }
    if !auth.system.enabled_countries.contains(&line.country) {
        return Err(AppError::CountryNotEnabled);
    }
    if let Err(msg) =
        crate::catalog::validate_country_currency_provider(&line.country, &line.currency, provider)
    {
        return Err(AppError::Validation(msg));
    }

    if let Some(existing) =
        queries::get_transaction_by_idempotency(state.db.pool(), system_id, line_key).await?
    {
        return Ok(existing.id);
    }

    let wallet = queries::get_or_create_wallet(
        state.db.pool(),
        system_id,
        &line.country,
        &line.currency,
    )
    .await?;

    if wallet.balance < line.amount {
        return Err(AppError::InsufficientBalance);
    }

    let payment_method = PaymentMethod {
        method_type: "mmo".into(),
        details: serde_json::json!({ "phone": phone, "provider": provider }),
    };

    let payout_id = Uuid::new_v4();
    let gateway_result = state
        .gateway
        .process_payment(GatewayPaymentRequest {
            payout_id,
            amount: line.amount,
            currency: line.currency.clone(),
            payment_method,
        })
        .await?;

    if !gateway_result.success {
        return Err(AppError::Gateway(
            gateway_result
                .error
                .unwrap_or_else(|| "payout failed".into()),
        ));
    }

    let request_hash = hash_request_body(line_key);
    let tx = queries::create_transaction_with_debit(
        state.db.pool(),
        wallet.id,
        line.amount,
        NewTransaction {
            system_id,
            wallet_id: wallet.id,
            external_id,
            idempotency_key: line_key,
            request_hash: &request_hash,
            amount: line.amount,
            currency: &line.currency,
            country: &line.country,
            status: "completed",
            gateway: &state.config.fallback_gateway,
            gateway_reference: gateway_result.reference.as_deref(),
            gateway_status: Some(&gateway_result.status),
            error: None,
            invoice_id: None,
            direction: "payout",
            batch_id: None,
            refund_id: None,
        },
    )
    .await?;

    let webhook_state = state.clone();
    let tx_clone = tx.clone();
    tokio::spawn(async move {
        if let Err(e) = sender::broadcast_payment_webhook(&webhook_state, &tx_clone).await {
            tracing::error!(payment_id = %tx_clone.id, error = %e, "webhook broadcast failed");
        }
    });

    Ok(tx.id)
}

fn extract_phone(pm: &PaymentMethod) -> String {
    pm.details
        .get("phone")
        .or_else(|| pm.details.get("phoneNumber"))
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .trim()
        .to_string()
}

fn extract_provider(pm: &PaymentMethod) -> String {
    pm.details
        .get("provider")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .trim()
        .to_string()
}

async fn batch_to_response(
    pool: &sqlx::PgPool,
    batch: crate::models::PayoutBatch,
) -> Result<BatchResponse, AppError> {
    let lines = batch_db::list_batch_lines(pool, batch.id).await?;
    Ok(BatchResponse {
        id: batch.id,
        status: batch.status,
        success_count: batch.success_count,
        failure_count: batch.failure_count,
        lines: lines.into_iter().map(line_to_response).collect(),
    })
}

fn line_to_response(l: PayoutBatchLine) -> BatchLineResponse {
    BatchLineResponse {
        line_index: l.line_index,
        status: l.status,
        transaction_id: l.transaction_id,
        error: l.error,
        external_id: l.external_id,
    }
}

pub async fn get_batch(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthenticatedSystem>,
    Path(id): Path<Uuid>,
) -> Result<Json<BatchResponse>, AppError> {
    let batch = batch_db::get_batch_by_id(state.db.pool(), id, auth.system.id).await?;
    Ok(Json(batch_to_response(state.db.pool(), batch).await?))
}
