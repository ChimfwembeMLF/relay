use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::error::AppError;
use crate::models::{Invoice, Transaction};

use super::queries::NewTransaction;

pub async fn create_invoice(
    pool: &PgPool,
    system_id: Uuid,
    reference: &str,
    description: Option<&str>,
    amount: i64,
    currency: &str,
    country: &str,
    expires_at: DateTime<Utc>,
    qr_payload_url: &str,
) -> Result<Invoice, AppError> {
    sqlx::query_as::<_, Invoice>(
        r#"
        INSERT INTO invoices (
            system_id, reference, description, amount, currency, country,
            status, expires_at, qr_payload_url
        )
        VALUES ($1, $2, $3, $4, $5, $6, 'open', $7, $8)
        RETURNING id, system_id, reference, description, amount, currency, country,
                  status, expires_at, paid_at, transaction_id, qr_payload_url, created_at, updated_at
        "#,
    )
    .bind(system_id)
    .bind(reference)
    .bind(description)
    .bind(amount)
    .bind(currency)
    .bind(country)
    .bind(expires_at)
    .bind(qr_payload_url)
    .fetch_one(pool)
    .await
    .map_err(AppError::from)
}

pub async fn get_invoice_by_reference_public(pool: &PgPool, reference: &str) -> Result<Invoice, AppError> {
    let invoice = sqlx::query_as::<_, Invoice>(
        r#"
        SELECT id, system_id, reference, description, amount, currency, country,
               status, expires_at, paid_at, transaction_id, qr_payload_url, created_at, updated_at
        FROM invoices WHERE reference = $1
        "#,
    )
    .bind(reference)
    .fetch_optional(pool)
    .await?
    .ok_or(AppError::NotFound)?;

    maybe_expire_invoice(pool, invoice).await
}

pub async fn get_invoice_by_id(pool: &PgPool, id: Uuid, system_id: Uuid) -> Result<Invoice, AppError> {
    let invoice = sqlx::query_as::<_, Invoice>(
        r#"
        SELECT id, system_id, reference, description, amount, currency, country,
               status, expires_at, paid_at, transaction_id, qr_payload_url, created_at, updated_at
        FROM invoices WHERE id = $1 AND system_id = $2
        "#,
    )
    .bind(id)
    .bind(system_id)
    .fetch_optional(pool)
    .await?
    .ok_or(AppError::NotFound)?;

    maybe_expire_invoice(pool, invoice).await
}

pub async fn get_invoice_by_reference(
    pool: &PgPool,
    reference: &str,
    system_id: Uuid,
) -> Result<Invoice, AppError> {
    let invoice = sqlx::query_as::<_, Invoice>(
        r#"
        SELECT id, system_id, reference, description, amount, currency, country,
               status, expires_at, paid_at, transaction_id, qr_payload_url, created_at, updated_at
        FROM invoices WHERE reference = $1 AND system_id = $2
        "#,
    )
    .bind(reference)
    .bind(system_id)
    .fetch_optional(pool)
    .await?
    .ok_or(AppError::NotFound)?;

    maybe_expire_invoice(pool, invoice).await
}

pub async fn list_invoices(
    pool: &PgPool,
    system_id: Uuid,
    status: Option<&str>,
    limit: i64,
) -> Result<Vec<Invoice>, AppError> {
    let invoices = if let Some(st) = status {
        sqlx::query_as::<_, Invoice>(
            r#"
            SELECT id, system_id, reference, description, amount, currency, country,
                   status, expires_at, paid_at, transaction_id, qr_payload_url, created_at, updated_at
            FROM invoices WHERE system_id = $1 AND status = $2
            ORDER BY created_at DESC LIMIT $3
            "#,
        )
        .bind(system_id)
        .bind(st)
        .bind(limit)
        .fetch_all(pool)
        .await?
    } else {
        sqlx::query_as::<_, Invoice>(
            r#"
            SELECT id, system_id, reference, description, amount, currency, country,
                   status, expires_at, paid_at, transaction_id, qr_payload_url, created_at, updated_at
            FROM invoices WHERE system_id = $1
            ORDER BY created_at DESC LIMIT $2
            "#,
        )
        .bind(system_id)
        .bind(limit)
        .fetch_all(pool)
        .await?
    };

    Ok(invoices)
}

async fn maybe_expire_invoice(pool: &PgPool, invoice: Invoice) -> Result<Invoice, AppError> {
    if invoice.status == "open" && invoice.expires_at < Utc::now() {
        sqlx::query("UPDATE invoices SET status = 'expired', updated_at = NOW() WHERE id = $1")
            .bind(invoice.id)
            .execute(pool)
            .await?;
        return get_invoice_row(pool, invoice.id).await;
    }
    Ok(invoice)
}

async fn get_invoice_row(pool: &PgPool, id: Uuid) -> Result<Invoice, AppError> {
    sqlx::query_as::<_, Invoice>(
        r#"
        SELECT id, system_id, reference, description, amount, currency, country,
               status, expires_at, paid_at, transaction_id, qr_payload_url, created_at, updated_at
        FROM invoices WHERE id = $1
        "#,
    )
    .bind(id)
    .fetch_one(pool)
    .await
    .map_err(AppError::from)
}

pub async fn cancel_invoice(pool: &PgPool, id: Uuid, system_id: Uuid) -> Result<Invoice, AppError> {
    let invoice = get_invoice_by_id(pool, id, system_id).await?;
    if invoice.status != "open" {
        return Err(AppError::InvoiceInvalid(format!(
            "cannot cancel invoice in status {}",
            invoice.status
        )));
    }
    sqlx::query("UPDATE invoices SET status = 'cancelled', updated_at = NOW() WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await?;
    get_invoice_row(pool, id).await
}

pub async fn mark_invoice_paid(
    pool: &PgPool,
    invoice_id: Uuid,
    transaction_id: Uuid,
) -> Result<Invoice, AppError> {
    sqlx::query(
        r#"
        UPDATE invoices SET status = 'paid', paid_at = NOW(), transaction_id = $2, updated_at = NOW()
        WHERE id = $1
        "#,
    )
    .bind(invoice_id)
    .bind(transaction_id)
    .execute(pool)
    .await?;
    get_invoice_row(pool, invoice_id).await
}

pub async fn create_deposit_with_credit(
    pool: &PgPool,
    wallet_id: Uuid,
    amount: i64,
    tx: NewTransaction<'_>,
) -> Result<Transaction, AppError> {
    let mut conn = pool.acquire().await?;

    sqlx::query(
        "UPDATE wallets SET balance = balance + $1, updated_at = NOW() WHERE id = $2",
    )
    .bind(amount)
    .bind(wallet_id)
    .execute(&mut *conn)
    .await?;

    let transaction = sqlx::query_as::<_, Transaction>(
        r#"
        INSERT INTO transactions (
            system_id, wallet_id, external_id, idempotency_key, request_hash,
            amount, currency, country, status, gateway, gateway_reference,
            gateway_status, error, invoice_id, direction
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15)
        RETURNING id, system_id, wallet_id, external_id, idempotency_key, request_hash,
                  amount, currency, country, status, gateway, gateway_reference,
                  gateway_status, error, invoice_id, direction, created_at, updated_at
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
    .fetch_one(&mut *conn)
    .await?;

    Ok(transaction)
}
