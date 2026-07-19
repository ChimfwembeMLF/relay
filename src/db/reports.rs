use chrono::{DateTime, Utc};
use sqlx::{PgPool, Row};
use uuid::Uuid;

use crate::error::AppError;
use crate::models::{Invoice, Wallet};

pub async fn transaction_report_summary(
    pool: &PgPool,
    system_id: Uuid,
    from: DateTime<Utc>,
    to: DateTime<Utc>,
    status: Option<&str>,
) -> Result<Vec<(String, i64, i64)>, AppError> {
    let rows = if let Some(st) = status {
        sqlx::query(
            r#"
            SELECT status, COUNT(*)::bigint as cnt, COALESCE(SUM(amount), 0)::bigint as total
            FROM transactions
            WHERE system_id = $1 AND created_at >= $2 AND created_at <= $3 AND status = $4
            GROUP BY status
            "#,
        )
        .bind(system_id)
        .bind(from)
        .bind(to)
        .bind(st)
        .fetch_all(pool)
        .await?
    } else {
        sqlx::query(
            r#"
            SELECT status, COUNT(*)::bigint as cnt, COALESCE(SUM(amount), 0)::bigint as total
            FROM transactions
            WHERE system_id = $1 AND created_at >= $2 AND created_at <= $3
            GROUP BY status
            "#,
        )
        .bind(system_id)
        .bind(from)
        .bind(to)
        .fetch_all(pool)
        .await?
    };

    Ok(rows
        .iter()
        .map(|r| {
            (
                r.get::<String, _>("status"),
                r.get::<i64, _>("cnt"),
                r.get::<i64, _>("total"),
            )
        })
        .collect())
}

pub async fn wallet_report_rows(
    pool: &PgPool,
    system_id: Uuid,
    from: DateTime<Utc>,
    to: DateTime<Utc>,
) -> Result<Vec<(Wallet, i64, i64)>, AppError> {
    let wallets = sqlx::query_as::<_, Wallet>(
        "SELECT id, system_id, country, currency, balance, created_at, updated_at FROM wallets WHERE system_id = $1",
    )
    .bind(system_id)
    .fetch_all(pool)
    .await?;

    let mut out = Vec::new();
    for wallet in wallets {
        let deposits: i64 = sqlx::query_scalar(
            r#"
            SELECT COALESCE(SUM(amount), 0)::bigint FROM transactions
            WHERE wallet_id = $1 AND direction = 'deposit' AND status = 'completed'
              AND created_at >= $2 AND created_at <= $3
            "#,
        )
        .bind(wallet.id)
        .bind(from)
        .bind(to)
        .fetch_one(pool)
        .await?;

        let payouts: i64 = sqlx::query_scalar(
            r#"
            SELECT COALESCE(SUM(amount), 0)::bigint FROM transactions
            WHERE wallet_id = $1 AND direction = 'payout' AND status = 'completed'
              AND created_at >= $2 AND created_at <= $3
            "#,
        )
        .bind(wallet.id)
        .bind(from)
        .bind(to)
        .fetch_one(pool)
        .await?;

        out.push((wallet, deposits, payouts));
    }
    Ok(out)
}

pub async fn invoice_report_summary(
    pool: &PgPool,
    system_id: Uuid,
    from: DateTime<Utc>,
    to: DateTime<Utc>,
    status: Option<&str>,
) -> Result<Vec<(String, i64, i64)>, AppError> {
    let rows = if let Some(st) = status {
        sqlx::query(
            r#"
            SELECT status, COUNT(*)::bigint as cnt, COALESCE(SUM(amount), 0)::bigint as total
            FROM invoices
            WHERE system_id = $1 AND created_at >= $2 AND created_at <= $3 AND status = $4
            GROUP BY status
            "#,
        )
        .bind(system_id)
        .bind(from)
        .bind(to)
        .bind(st)
        .fetch_all(pool)
        .await?
    } else {
        sqlx::query(
            r#"
            SELECT status, COUNT(*)::bigint as cnt, COALESCE(SUM(amount), 0)::bigint as total
            FROM invoices
            WHERE system_id = $1 AND created_at >= $2 AND created_at <= $3
            GROUP BY status
            "#,
        )
        .bind(system_id)
        .bind(from)
        .bind(to)
        .fetch_all(pool)
        .await?
    };

    Ok(rows
        .iter()
        .map(|r| {
            (
                r.get::<String, _>("status"),
                r.get::<i64, _>("cnt"),
                r.get::<i64, _>("total"),
            )
        })
        .collect())
}

pub async fn list_invoices_detail(
    pool: &PgPool,
    system_id: Uuid,
    from: DateTime<Utc>,
    to: DateTime<Utc>,
    limit: i64,
) -> Result<Vec<Invoice>, AppError> {
    sqlx::query_as::<_, Invoice>(
        r#"
        SELECT id, system_id, reference, description, amount, currency, country,
               status, expires_at, paid_at, transaction_id, qr_payload_url, created_at, updated_at
        FROM invoices
        WHERE system_id = $1 AND created_at >= $2 AND created_at <= $3
        ORDER BY created_at DESC LIMIT $4
        "#,
    )
    .bind(system_id)
    .bind(from)
    .bind(to)
    .bind(limit)
    .fetch_all(pool)
    .await
    .map_err(AppError::from)
}
