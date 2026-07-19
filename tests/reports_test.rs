mod common;

use std::sync::Arc;

use chrono::{Duration, Utc};
use common::{json_request, register_test_system, setup_test_state, test_router};
use payment_relay::gateway::mock::MockGateway;
use uuid::Uuid;

#[tokio::test]
async fn transaction_and_invoice_reports_json() {
    let state = setup_test_state(Arc::new(MockGateway::success())).await;
    let app = test_router(state);

    let (system_id, api_key, _prefix) = register_test_system(&app).await;

    let create_body = r#"{
        "amount": 2500,
        "currency": "ZMW",
        "country": "ZM"
    }"#;
    let (_, resp) =
        json_request(&app, "POST", "/invoices", Some(&api_key), Some(create_body.into())).await;
    let invoice: serde_json::Value = serde_json::from_str(&resp).unwrap();
    let invoice_id = invoice["id"].as_str().unwrap();

    let collect_body = format!(
        r#"{{
            "idempotency_key": "report-{}",
            "payment_method": {{
                "type": "mmo",
                "details": {{ "provider": "MTN_MOMO_ZMB", "phoneNumber": "260763456789" }}
            }}
        }}"#,
        Uuid::new_v4()
    );
    json_request(
        &app,
        "POST",
        &format!("/invoices/{invoice_id}/collect"),
        Some(&api_key),
        Some(collect_body),
    )
    .await;

    let from = url_encode_datetime(Utc::now() - Duration::hours(1));
    let to = url_encode_datetime(Utc::now() + Duration::hours(1));

    let (status, tx_report) = json_request(
        &app,
        "GET",
        &format!("/reports/transactions?from={from}&to={to}"),
        Some(&api_key),
        None,
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::OK, "tx report: {tx_report}");
    let tx: serde_json::Value = serde_json::from_str(&tx_report).unwrap();
    assert!(tx["total_count"].as_i64().unwrap() >= 1);

    let (status, inv_report) = json_request(
        &app,
        "GET",
        &format!("/reports/invoices?from={from}&to={to}"),
        Some(&api_key),
        None,
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::OK);
    let inv: serde_json::Value = serde_json::from_str(&inv_report).unwrap();
    assert!(inv["total_count"].as_i64().unwrap() >= 1);

    let (status, wallet_report) = json_request(
        &app,
        "GET",
        &format!("/reports/wallets?from={from}&to={to}"),
        Some(&api_key),
        None,
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::OK);
    let wallets: serde_json::Value = serde_json::from_str(&wallet_report).unwrap();
    assert!(wallets["wallets"].as_array().unwrap().len() >= 1);

    let _ = system_id;
}

#[tokio::test]
async fn invoice_report_csv_has_expected_headers() {
    let state = setup_test_state(Arc::new(MockGateway::success())).await;
    let app = test_router(state);

    let (_system_id, api_key, _prefix) = register_test_system(&app).await;

    let create_body = r#"{"amount": 1000, "currency": "ZMW", "country": "ZM"}"#;
    json_request(
        &app,
        "POST",
        "/invoices",
        Some(&api_key),
        Some(create_body.into()),
    )
    .await;

    let from = url_encode_datetime(Utc::now() - Duration::hours(1));
    let to = url_encode_datetime(Utc::now() + Duration::hours(1));

    let (status, csv) = json_request(
        &app,
        "GET",
        &format!("/reports/invoices?from={from}&to={to}&format=csv"),
        Some(&api_key),
        None,
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::OK);
    assert!(csv.starts_with("reference,amount,currency,country,status,created_at,paid_at"));
}

fn url_encode_datetime(dt: chrono::DateTime<Utc>) -> String {
    dt.to_rfc3339()
        .replace(':', "%3A")
        .replace('+', "%2B")
}
