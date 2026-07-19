mod common;

use std::sync::Arc;

use common::{form_request, json_request, register_test_system, setup_test_state, test_router};
use payment_relay::gateway::mock::MockGateway;
use payment_relay::pay;
use uuid::Uuid;

#[tokio::test]
async fn pay_page_shows_open_invoice() {
    let state = setup_test_state(Arc::new(MockGateway::success())).await;
    let app = test_router(state);

    let (_system_id, api_key, _prefix) = register_test_system(&app).await;

    let (_, resp) = json_request(
        &app,
        "POST",
        "/invoices",
        Some(&api_key),
        Some(r#"{"amount":5000,"currency":"ZMW","country":"ZM","description":"Sale"}"#.into()),
    )
    .await;
    let invoice: serde_json::Value = serde_json::from_str(&resp).unwrap();
    let reference = invoice["reference"].as_str().unwrap();

    let (status, html) = json_request(
        &app,
        "GET",
        &format!("/pay/{reference}"),
        None,
        None,
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::OK);
    assert!(html.contains("Pay Invoice"));
    assert!(html.contains("ZMW"));
    assert!(html.contains("Pay now"));
    assert!(html.contains(reference));
}

#[tokio::test]
async fn pay_page_collect_and_idempotency() {
    let state = setup_test_state(Arc::new(MockGateway::success())).await;
    let app = test_router(state);

    let (system_id, api_key, _prefix) = register_test_system(&app).await;

    let (_, resp) = json_request(
        &app,
        "POST",
        "/invoices",
        Some(&api_key),
        Some(r#"{"amount":2500,"currency":"ZMW","country":"ZM"}"#.into()),
    )
    .await;
    let invoice: serde_json::Value = serde_json::from_str(&resp).unwrap();
    let reference = invoice["reference"].as_str().unwrap();
    let expires_at = invoice["expires_at"].as_str().unwrap();
    let expires_at = chrono::DateTime::parse_from_rfc3339(expires_at)
        .unwrap()
        .with_timezone(&chrono::Utc);

    let form_token = pay::generate_form_token("test-secret", reference, &expires_at);
    let idem = Uuid::new_v4().to_string();
    let body = format!(
        "phone=260763456789&provider=MTN_MOMO_ZMB&idempotency_key={idem}&form_token={form_token}"
    );

    let (status, html) = form_request(
        &app,
        "POST",
        &format!("/pay/{reference}"),
        Some(body.clone()),
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::OK, "pay failed: {html}");
    assert!(html.contains("Payment successful"));

    let (_, html2) = form_request(
        &app,
        "POST",
        &format!("/pay/{reference}"),
        Some(body),
    )
    .await;
    assert!(html2.contains("Payment successful"));

    let (_, wallets) =
        json_request(&app, "GET", &format!("/wallets/{system_id}"), Some(&api_key), None).await;
    let wallets: serde_json::Value = serde_json::from_str(&wallets).unwrap();
    let zm = wallets
        .as_array()
        .unwrap()
        .iter()
        .find(|w| w["country"] == "ZM")
        .unwrap();
    assert!(zm["balance"].as_i64().unwrap() >= 102_500);
}

#[tokio::test]
async fn pay_page_unknown_reference_is_generic() {
    let state = setup_test_state(Arc::new(MockGateway::success())).await;
    let app = test_router(state);

    let (status, html) = json_request(
        &app,
        "GET",
        "/pay/INV_FAKE_00000000",
        None,
        None,
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::NOT_FOUND);
    assert!(html.contains("Not found"));
    assert!(!html.contains("INV_FAKE"));
}
