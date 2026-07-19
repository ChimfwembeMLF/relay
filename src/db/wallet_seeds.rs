use uuid::Uuid;

use crate::error::AppError;
use crate::models::Wallet;

pub async fn create_wallet_with_balance<'e, E>(
    executor: E,
    system_id: Uuid,
    country: &str,
    currency: &str,
    balance: i64,
) -> Result<Wallet, AppError>
where
    E: sqlx::Executor<'e, Database = sqlx::Postgres>,
{
    sqlx::query_as::<_, Wallet>(
        r#"
        INSERT INTO wallets (system_id, country, currency, balance)
        VALUES ($1, $2, $3, $4)
        RETURNING id, system_id, country, currency, balance, created_at, updated_at
        "#,
    )
    .bind(system_id)
    .bind(country)
    .bind(currency)
    .bind(balance)
    .fetch_one(executor)
    .await
    .map_err(AppError::from)
}

pub async fn record_wallet_seed_event<'e, E>(
    executor: E,
    system_id: Uuid,
    wallet_id: Uuid,
    country: &str,
    currency: &str,
    amount: i64,
    source: &str,
) -> Result<(), AppError>
where
    E: sqlx::Executor<'e, Database = sqlx::Postgres>,
{
    sqlx::query(
        r#"
        INSERT INTO wallet_seed_events (system_id, wallet_id, country, currency, amount, source)
        VALUES ($1, $2, $3, $4, $5, $6)
        "#,
    )
    .bind(system_id)
    .bind(wallet_id)
    .bind(country)
    .bind(currency)
    .bind(amount)
    .bind(source)
    .execute(executor)
    .await?;
    Ok(())
}
