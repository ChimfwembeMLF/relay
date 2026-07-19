use sqlx::{PgPool, Row};
use uuid::Uuid;

use crate::error::AppError;
use crate::models::{System, Transaction, Wallet};

pub async fn prefix_exists(pool: &PgPool, prefix: &str) -> Result<bool, AppError> {
    let row = sqlx::query("SELECT 1 FROM systems WHERE prefix = $1")
        .bind(prefix)
        .fetch_optional(pool)
        .await?;
    Ok(row.is_some())
}

pub async fn create_system(
    pool: &PgPool,
    name: &str,
    prefix: &str,
    enabled_countries: &[String],
    webhook_url: Option<&str>,
    api_key_hash: &str,
) -> Result<System, AppError> {
    let system = sqlx::query_as::<_, System>(
        r#"
        INSERT INTO systems (name, prefix, enabled_countries, webhook_url, api_key_hash)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING id, name, prefix, enabled_countries, webhook_url, api_key_hash, created_at, updated_at
        "#,
    )
    .bind(name)
    .bind(prefix)
    .bind(enabled_countries)
    .bind(webhook_url)
    .bind(api_key_hash)
    .fetch_one(pool)
    .await?;

    Ok(system)
}

pub async fn create_system_in_tx(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    name: &str,
    prefix: &str,
    enabled_countries: &[String],
    webhook_url: Option<&str>,
    api_key_hash: &str,
) -> Result<System, AppError> {
    let system = sqlx::query_as::<_, System>(
        r#"
        INSERT INTO systems (name, prefix, enabled_countries, webhook_url, api_key_hash)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING id, name, prefix, enabled_countries, webhook_url, api_key_hash, created_at, updated_at
        "#,
    )
    .bind(name)
    .bind(prefix)
    .bind(enabled_countries)
    .bind(webhook_url)
    .bind(api_key_hash)
    .fetch_one(&mut **tx)
    .await?;

    Ok(system)
}

pub async fn get_system_by_id(pool: &PgPool, id: Uuid) -> Result<System, AppError> {
    sqlx::query_as::<_, System>(
        r#"
        SELECT id, name, prefix, enabled_countries, webhook_url, api_key_hash, created_at, updated_at
        FROM systems WHERE id = $1
        "#,
    )
    .bind(id)
    .fetch_optional(pool)
    .await?
    .ok_or(AppError::NotFound)
}

pub async fn get_system_by_api_key_hash(pool: &PgPool, api_key_hash: &str) -> Result<System, AppError> {
    sqlx::query_as::<_, System>(
        r#"
        SELECT id, name, prefix, enabled_countries, webhook_url, api_key_hash, created_at, updated_at
        FROM systems WHERE api_key_hash = $1
        "#,
    )
    .bind(api_key_hash)
    .fetch_optional(pool)
    .await?
    .ok_or(AppError::Unauthorized)
}

pub async fn get_or_create_wallet(
    pool: &PgPool,
    system_id: Uuid,
    country: &str,
    currency: &str,
) -> Result<Wallet, AppError> {
    if let Some(wallet) = sqlx::query_as::<_, Wallet>(
        r#"
        SELECT id, system_id, country, currency, balance, created_at, updated_at
        FROM wallets WHERE system_id = $1 AND country = $2 AND currency = $3
        "#,
    )
    .bind(system_id)
    .bind(country)
    .bind(currency)
    .fetch_optional(pool)
    .await?
    {
        return Ok(wallet);
    }

    sqlx::query_as::<_, Wallet>(
        r#"
        INSERT INTO wallets (system_id, country, currency)
        VALUES ($1, $2, $3)
        RETURNING id, system_id, country, currency, balance, created_at, updated_at
        "#,
    )
    .bind(system_id)
    .bind(country)
    .bind(currency)
    .fetch_one(pool)
    .await
    .map_err(AppError::from)
}

pub async fn list_wallets_by_system(pool: &PgPool, system_id: Uuid) -> Result<Vec<Wallet>, AppError> {
    sqlx::query_as::<_, Wallet>(
        r#"
        SELECT id, system_id, country, currency, balance, created_at, updated_at
        FROM wallets WHERE system_id = $1 ORDER BY country, currency
        "#,
    )
    .bind(system_id)
    .fetch_all(pool)
    .await
    .map_err(AppError::from)
}

pub async fn get_transaction_by_idempotency(
    pool: &PgPool,
    system_id: Uuid,
    idempotency_key: &str,
) -> Result<Option<Transaction>, AppError> {
    sqlx::query_as::<_, Transaction>(
        r#"
        SELECT id, system_id, wallet_id, external_id, idempotency_key, request_hash,
               amount, currency, country, status, gateway, gateway_reference,
               gateway_status, error, invoice_id, direction, created_at, updated_at
        FROM transactions
        WHERE system_id = $1 AND idempotency_key = $2
        "#,
    )
    .bind(system_id)
    .bind(idempotency_key)
    .fetch_optional(pool)
    .await
    .map_err(AppError::from)
}

pub async fn get_transaction_by_id(
    pool: &PgPool,
    id: Uuid,
    system_id: Uuid,
) -> Result<Transaction, AppError> {
    sqlx::query_as::<_, Transaction>(
        r#"
        SELECT id, system_id, wallet_id, external_id, idempotency_key, request_hash,
               amount, currency, country, status, gateway, gateway_reference,
               gateway_status, error, invoice_id, direction, created_at, updated_at
        FROM transactions
        WHERE id = $1 AND system_id = $2
        "#,
    )
    .bind(id)
    .bind(system_id)
    .fetch_optional(pool)
    .await?
    .ok_or(AppError::NotFound)
}

pub async fn list_transactions_by_system(
    pool: &PgPool,
    system_id: Uuid,
    external_id: Option<&str>,
    limit: i64,
) -> Result<Vec<Transaction>, AppError> {
    if let Some(ext_id) = external_id {
        sqlx::query_as::<_, Transaction>(
            r#"
            SELECT id, system_id, wallet_id, external_id, idempotency_key, request_hash,
                   amount, currency, country, status, gateway, gateway_reference,
                   gateway_status, error, invoice_id, direction, created_at, updated_at
            FROM transactions
            WHERE system_id = $1 AND external_id = $2
            ORDER BY created_at DESC
            LIMIT $3
            "#,
        )
        .bind(system_id)
        .bind(ext_id)
        .bind(limit)
        .fetch_all(pool)
        .await
        .map_err(AppError::from)
    } else {
        sqlx::query_as::<_, Transaction>(
            r#"
            SELECT id, system_id, wallet_id, external_id, idempotency_key, request_hash,
                   amount, currency, country, status, gateway, gateway_reference,
                   gateway_status, error, invoice_id, direction, created_at, updated_at
            FROM transactions
            WHERE system_id = $1
            ORDER BY created_at DESC
            LIMIT $2
            "#,
        )
        .bind(system_id)
        .bind(limit)
        .fetch_all(pool)
        .await
        .map_err(AppError::from)
    }
}

pub struct NewTransaction<'a> {
    pub system_id: Uuid,
    pub wallet_id: Uuid,
    pub external_id: &'a str,
    pub idempotency_key: &'a str,
    pub request_hash: &'a str,
    pub amount: i64,
    pub currency: &'a str,
    pub country: &'a str,
    pub status: &'a str,
    pub gateway: &'a str,
    pub gateway_reference: Option<&'a str>,
    pub gateway_status: Option<&'a str>,
    pub error: Option<&'a str>,
    pub invoice_id: Option<Uuid>,
    pub direction: &'a str,
}

pub async fn create_transaction_with_debit(
    pool: &PgPool,
    wallet_id: Uuid,
    amount: i64,
    tx: NewTransaction<'_>,
) -> Result<Transaction, AppError> {
    let mut conn = pool.acquire().await?;

    let updated = sqlx::query(
        r#"
        UPDATE wallets SET balance = balance - $1, updated_at = NOW()
        WHERE id = $2 AND balance >= $1
        RETURNING balance
        "#,
    )
    .bind(amount)
    .bind(wallet_id)
    .fetch_optional(&mut *conn)
    .await?;

    if updated.is_none() {
        return Err(AppError::InsufficientBalance);
    }

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

pub async fn update_wallet_balance(pool: &PgPool, wallet_id: Uuid, delta: i64) -> Result<(), AppError> {
    let result = sqlx::query(
        r#"
        UPDATE wallets SET balance = balance + $1, updated_at = NOW()
        WHERE id = $2 AND ($1 >= 0 OR balance + $1 >= 0)
        "#,
    )
    .bind(delta)
    .bind(wallet_id)
    .execute(pool)
    .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::InsufficientBalance);
    }
    Ok(())
}

pub async fn record_webhook_attempt(
    pool: &PgPool,
    transaction_id: Uuid,
    attempt_number: i32,
    url: &str,
    status_code: Option<i32>,
    success: bool,
    error: Option<&str>,
    event_type: &str,
) -> Result<(), AppError> {
    sqlx::query(
        r#"
        INSERT INTO webhook_delivery_attempts
            (transaction_id, attempt_number, url, status_code, success, error, event_type)
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        "#,
    )
    .bind(transaction_id)
    .bind(attempt_number)
    .bind(url)
    .bind(status_code)
    .bind(success)
    .bind(error)
    .bind(event_type)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn seed_wallet_balance(
    pool: &PgPool,
    wallet_id: Uuid,
    balance: i64,
) -> Result<(), AppError> {
    sqlx::query("UPDATE wallets SET balance = $1, updated_at = NOW() WHERE id = $2")
        .bind(balance)
        .bind(wallet_id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn cleanup_test_system(pool: &PgPool, system_id: Uuid) -> Result<(), AppError> {
    sqlx::query("DELETE FROM systems WHERE id = $1")
        .bind(system_id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn system_count_by_prefix(pool: &PgPool, prefix: &str) -> Result<i64, AppError> {
    let row = sqlx::query("SELECT COUNT(*) as cnt FROM systems WHERE prefix = $1")
        .bind(prefix)
        .fetch_one(pool)
        .await?;
    Ok(row.get("cnt"))
}
