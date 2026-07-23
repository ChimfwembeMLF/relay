use sqlx::PgPool;
use uuid::Uuid;

use crate::error::AppError;
use crate::models::{PayoutBatch, PayoutBatchLine, Refund};

pub async fn get_batch_by_idempotency(
    pool: &PgPool,
    system_id: Uuid,
    idempotency_key: &str,
) -> Result<Option<PayoutBatch>, AppError> {
    sqlx::query_as::<_, PayoutBatch>(
        r#"
        SELECT id, system_id, idempotency_key, request_hash, status, line_count,
               success_count, failure_count, created_at
        FROM payout_batches
        WHERE system_id = $1 AND idempotency_key = $2
        "#,
    )
    .bind(system_id)
    .bind(idempotency_key)
    .fetch_optional(pool)
    .await
    .map_err(AppError::from)
}

pub async fn get_batch_by_id(
    pool: &PgPool,
    id: Uuid,
    system_id: Uuid,
) -> Result<PayoutBatch, AppError> {
    sqlx::query_as::<_, PayoutBatch>(
        r#"
        SELECT id, system_id, idempotency_key, request_hash, status, line_count,
               success_count, failure_count, created_at
        FROM payout_batches
        WHERE id = $1 AND system_id = $2
        "#,
    )
    .bind(id)
    .bind(system_id)
    .fetch_optional(pool)
    .await?
    .ok_or(AppError::NotFound)
}

pub async fn list_batch_lines(pool: &PgPool, batch_id: Uuid) -> Result<Vec<PayoutBatchLine>, AppError> {
    sqlx::query_as::<_, PayoutBatchLine>(
        r#"
        SELECT id, batch_id, line_index, external_id, amount, currency, country, phone, provider,
               status, error, transaction_id, line_idempotency_key
        FROM payout_batch_lines
        WHERE batch_id = $1
        ORDER BY line_index
        "#,
    )
    .bind(batch_id)
    .fetch_all(pool)
    .await
    .map_err(AppError::from)
}

pub async fn insert_batch(
    pool: &PgPool,
    system_id: Uuid,
    idempotency_key: &str,
    request_hash: &str,
    status: &str,
    line_count: i32,
    success_count: i32,
    failure_count: i32,
) -> Result<PayoutBatch, AppError> {
    sqlx::query_as::<_, PayoutBatch>(
        r#"
        INSERT INTO payout_batches (
            system_id, idempotency_key, request_hash, status, line_count, success_count, failure_count
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        RETURNING id, system_id, idempotency_key, request_hash, status, line_count,
                  success_count, failure_count, created_at
        "#,
    )
    .bind(system_id)
    .bind(idempotency_key)
    .bind(request_hash)
    .bind(status)
    .bind(line_count)
    .bind(success_count)
    .bind(failure_count)
    .fetch_one(pool)
    .await
    .map_err(AppError::from)
}

pub async fn insert_batch_line(
    pool: &PgPool,
    batch_id: Uuid,
    line_index: i32,
    external_id: &str,
    amount: i64,
    currency: &str,
    country: &str,
    phone: &str,
    provider: &str,
    status: &str,
    error: Option<&str>,
    transaction_id: Option<Uuid>,
    line_idempotency_key: &str,
) -> Result<PayoutBatchLine, AppError> {
    sqlx::query_as::<_, PayoutBatchLine>(
        r#"
        INSERT INTO payout_batch_lines (
            batch_id, line_index, external_id, amount, currency, country, phone, provider,
            status, error, transaction_id, line_idempotency_key
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
        RETURNING id, batch_id, line_index, external_id, amount, currency, country, phone, provider,
                  status, error, transaction_id, line_idempotency_key
        "#,
    )
    .bind(batch_id)
    .bind(line_index)
    .bind(external_id)
    .bind(amount)
    .bind(currency)
    .bind(country)
    .bind(phone)
    .bind(provider)
    .bind(status)
    .bind(error)
    .bind(transaction_id)
    .bind(line_idempotency_key)
    .fetch_one(pool)
    .await
    .map_err(AppError::from)
}

pub async fn get_refund_by_idempotency(
    pool: &PgPool,
    system_id: Uuid,
    idempotency_key: &str,
) -> Result<Option<Refund>, AppError> {
    sqlx::query_as::<_, Refund>(
        r#"
        SELECT id, system_id, invoice_id, amount, currency, country, phone, provider,
               idempotency_key, request_hash, status, transaction_id, error, created_at
        FROM refunds
        WHERE system_id = $1 AND idempotency_key = $2
        "#,
    )
    .bind(system_id)
    .bind(idempotency_key)
    .fetch_optional(pool)
    .await
    .map_err(AppError::from)
}

pub async fn insert_refund(
    pool: &PgPool,
    system_id: Uuid,
    invoice_id: Uuid,
    amount: i64,
    currency: &str,
    country: &str,
    phone: &str,
    provider: &str,
    idempotency_key: &str,
    request_hash: &str,
    status: &str,
    transaction_id: Option<Uuid>,
    error: Option<&str>,
) -> Result<Refund, AppError> {
    sqlx::query_as::<_, Refund>(
        r#"
        INSERT INTO refunds (
            system_id, invoice_id, amount, currency, country, phone, provider,
            idempotency_key, request_hash, status, transaction_id, error
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
        RETURNING id, system_id, invoice_id, amount, currency, country, phone, provider,
                  idempotency_key, request_hash, status, transaction_id, error, created_at
        "#,
    )
    .bind(system_id)
    .bind(invoice_id)
    .bind(amount)
    .bind(currency)
    .bind(country)
    .bind(phone)
    .bind(provider)
    .bind(idempotency_key)
    .bind(request_hash)
    .bind(status)
    .bind(transaction_id)
    .bind(error)
    .fetch_one(pool)
    .await
    .map_err(AppError::from)
}

pub async fn update_refund_transaction(
    pool: &PgPool,
    refund_id: Uuid,
    transaction_id: Uuid,
) -> Result<(), AppError> {
    sqlx::query("UPDATE refunds SET transaction_id = $2 WHERE id = $1")
        .bind(refund_id)
        .bind(transaction_id)
        .execute(pool)
        .await?;
    Ok(())
}
