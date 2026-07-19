use std::collections::HashMap;

use axum::{
    extract::{Query, State},
    http::{header, HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    Extension, Json,
};
use chrono::{DateTime, Utc};

use crate::api::routes::AuthenticatedSystem;
use crate::db::reports::{
    invoice_report_summary, list_invoices_detail, transaction_report_summary, wallet_report_rows,
};
use crate::db::queries;
use crate::error::AppError;
use crate::models::{ReportSummary, StatusSummary, Transaction};
use crate::AppState;

const MAX_REPORT_ROWS: i64 = 10_000;

#[derive(serde::Deserialize)]
pub struct ReportQuery {
    pub from: DateTime<Utc>,
    pub to: DateTime<Utc>,
    #[serde(default)]
    pub format: Option<String>,
    pub status: Option<String>,
    #[serde(default)]
    pub detail: Option<bool>,
}

pub async fn transactions_report(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthenticatedSystem>,
    Query(q): Query<ReportQuery>,
) -> Result<Response, AppError> {
    validate_range(&q.from, &q.to)?;
    tracing::debug!(
        system_id = %auth.system.id,
        from = %q.from,
        to = %q.to,
        "transactions report"
    );
    let rows = transaction_report_summary(
        state.db.pool(),
        auth.system.id,
        q.from,
        q.to,
        q.status.as_deref(),
    )
    .await?;
    let summary = build_summary(q.from, q.to, rows);

    if q.format.as_deref() == Some("csv") && q.detail.unwrap_or(false) {
        let txs = queries::list_transactions_by_system(
            state.db.pool(),
            auth.system.id,
            None,
            MAX_REPORT_ROWS + 1,
        )
        .await?;
        if txs.len() as i64 > MAX_REPORT_ROWS {
            return Err(AppError::PayloadTooLarge(format!(
                "report exceeds {MAX_REPORT_ROWS} rows; narrow date range"
            )));
        }
        return Ok(csv_transactions(txs));
    }

    Ok(Json(summary).into_response())
}

pub async fn wallets_report(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthenticatedSystem>,
    Query(q): Query<ReportQuery>,
) -> Result<Response, AppError> {
    validate_range(&q.from, &q.to)?;
    let rows = wallet_report_rows(state.db.pool(), auth.system.id, q.from, q.to).await?;

    if q.format.as_deref() == Some("csv") {
        let mut wtr = csv::Writer::from_writer(vec![]);
        wtr.write_record([
            "country",
            "currency",
            "current_balance",
            "period_deposits",
            "period_payouts",
            "net_change",
        ])
        .map_err(|e| AppError::Internal(e.to_string()))?;
        for (wallet, deposits, payouts) in &rows {
            wtr.write_record([
                wallet.country.clone(),
                wallet.currency.clone(),
                wallet.balance.to_string(),
                deposits.to_string(),
                payouts.to_string(),
                (deposits - payouts).to_string(),
            ])
            .map_err(|e| AppError::Internal(e.to_string()))?;
        }
        let data = String::from_utf8(wtr.into_inner().map_err(|e| AppError::Internal(e.to_string()))?)
            .map_err(|e| AppError::Internal(e.to_string()))?;
        return Ok(csv_response("wallets-report.csv", data));
    }

    let payload: Vec<_> = rows
        .into_iter()
        .map(|(w, deposits, payouts)| {
            serde_json::json!({
                "country": w.country,
                "currency": w.currency,
                "current_balance": w.balance,
                "period_deposits": deposits,
                "period_payouts": payouts,
                "net_change": deposits - payouts,
            })
        })
        .collect();

    Ok(Json(serde_json::json!({ "from": q.from, "to": q.to, "wallets": payload })).into_response())
}

pub async fn invoices_report(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthenticatedSystem>,
    Query(q): Query<ReportQuery>,
) -> Result<Response, AppError> {
    validate_range(&q.from, &q.to)?;
    let rows = invoice_report_summary(
        state.db.pool(),
        auth.system.id,
        q.from,
        q.to,
        q.status.as_deref(),
    )
    .await?;
    let summary = build_summary(q.from, q.to, rows);

    if q.format.as_deref() == Some("csv") {
        let invoices = list_invoices_detail(
            state.db.pool(),
            auth.system.id,
            q.from,
            q.to,
            MAX_REPORT_ROWS + 1,
        )
        .await?;
        if invoices.len() as i64 > MAX_REPORT_ROWS {
            return Err(AppError::PayloadTooLarge(format!(
                "report exceeds {MAX_REPORT_ROWS} rows"
            )));
        }
        let mut wtr = csv::Writer::from_writer(vec![]);
        wtr.write_record([
            "reference",
            "amount",
            "currency",
            "country",
            "status",
            "created_at",
            "paid_at",
        ])
        .map_err(|e| AppError::Internal(e.to_string()))?;
        for inv in invoices {
            wtr.write_record([
                inv.reference,
                inv.amount.to_string(),
                inv.currency,
                inv.country,
                inv.status,
                inv.created_at.to_rfc3339(),
                inv.paid_at.map(|t| t.to_rfc3339()).unwrap_or_default(),
            ])
            .map_err(|e| AppError::Internal(e.to_string()))?;
        }
        let data = String::from_utf8(wtr.into_inner().map_err(|e| AppError::Internal(e.to_string()))?)
            .map_err(|e| AppError::Internal(e.to_string()))?;
        return Ok(csv_response("invoices-report.csv", data));
    }

    Ok(Json(summary).into_response())
}

fn build_summary(from: DateTime<Utc>, to: DateTime<Utc>, rows: Vec<(String, i64, i64)>) -> ReportSummary {
    let mut by_status = HashMap::new();
    let mut total_count = 0i64;
    let mut total_amount = 0i64;
    for (status, count, amount) in rows {
        total_count += count;
        total_amount += amount;
        by_status.insert(
            status.clone(),
            StatusSummary {
                count,
                amount,
            },
        );
    }
    ReportSummary {
        from,
        to,
        total_count,
        total_amount,
        by_status,
    }
}

fn validate_range(from: &DateTime<Utc>, to: &DateTime<Utc>) -> Result<(), AppError> {
    if from > to {
        return Err(AppError::Validation("from must be before to".into()));
    }
    Ok(())
}

fn csv_transactions(txs: Vec<Transaction>) -> Response {
    let mut wtr = csv::Writer::from_writer(vec![]);
    let _ = wtr.write_record([
        "id",
        "external_id",
        "amount",
        "currency",
        "country",
        "status",
        "direction",
        "created_at",
    ]);
    for tx in txs {
        let _ = wtr.write_record([
            tx.id.to_string(),
            tx.external_id,
            tx.amount.to_string(),
            tx.currency,
            tx.country,
            tx.status,
            tx.direction,
            tx.created_at.to_rfc3339(),
        ]);
    }
    let data = String::from_utf8(wtr.into_inner().unwrap_or_default()).unwrap_or_default();
    csv_response("transactions-report.csv", data)
}

fn csv_response(filename: &str, data: String) -> Response {
    let mut headers = HeaderMap::new();
    headers.insert(header::CONTENT_TYPE, "text/csv".parse().unwrap());
    headers.insert(
        header::CONTENT_DISPOSITION,
        format!("attachment; filename=\"{filename}\"")
            .parse()
            .unwrap(),
    );
    (StatusCode::OK, headers, data).into_response()
}
