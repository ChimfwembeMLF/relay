mod common;

use std::sync::Arc;

use common::{json_request, register_test_system, setup_test_state, test_router};
use payment_relay::gateway::mock::MockGateway;

#[tokio::test]
async fn pay_api_returns_open_invoice() {
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

    let (status, body) = json_request(
        &app,
        "GET",
        &format!("/api/pay/{reference}"),
        None,
        None,
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::OK);
    let data: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert_eq!(data["status"], "open");
    assert_eq!(data["payable"], true);
    assert!(data["form_token"].is_string());
    assert!(data["amount_display"].as_str().unwrap().contains("ZMW"));
}

#[tokio::test]
async fn pay_api_collect_and_idempotency() {
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

    let (_, api_body) = json_request(
        &app,
        "GET",
        &format!("/api/pay/{reference}"),
        None,
        None,
    )
    .await;
    let api: serde_json::Value = serde_json::from_str(&api_body).unwrap();

    let pay_body = serde_json::json!({
        "phone": "260763456789",
        "provider": "MTN_MOMO_ZMB",
        "idempotency_key": api["idempotency_key"],
        "form_token": api["form_token"],
    })
    .to_string();

    let (status, pay_resp) = json_request(
        &app,
        "POST",
        &format!("/api/pay/{reference}"),
        None,
        Some(pay_body.clone()),
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::OK, "pay failed: {pay_resp}");
    let result: serde_json::Value = serde_json::from_str(&pay_resp).unwrap();
    assert_eq!(result["status"], "paid");

    let (status2, pay_resp2) = json_request(
        &app,
        "POST",
        &format!("/api/pay/{reference}"),
        None,
        Some(pay_body),
    )
    .await;
    assert_eq!(status2, axum::http::StatusCode::OK);
    let result2: serde_json::Value = serde_json::from_str(&pay_resp2).unwrap();
    assert_eq!(result2["status"], "paid");

    let (_, wallets) =
        json_request(&app, "GET", &format!("/wallets/{system_id}"), Some(&api_key), None).await;
    let wallets: serde_json::Value = serde_json::from_str(&wallets).unwrap();
    let zm = wallets
        .as_array()
        .unwrap()
        .iter()
        .find(|w| w["country"] == "ZM")
        .unwrap();
    assert!(zm["balance"].as_i64().unwrap() >= 2_500);
}

#[tokio::test]
async fn pay_api_unknown_reference() {
    let state = setup_test_state(Arc::new(MockGateway::success())).await;
    let app = test_router(state);

    let (status, body) = json_request(
        &app,
        "GET",
        "/api/pay/INV_FAKE_00000000",
        None,
        None,
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::NOT_FOUND);
    let data: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert_eq!(data["error"], "not_found");
}

#[tokio::test]
async fn pay_spa_serves_react_shell() {
    let state = setup_test_state(Arc::new(MockGateway::success())).await;
    let app = test_router(state);

    let (_, api_key, _) = register_test_system(&app).await;
    let (_, resp) = json_request(
        &app,
        "POST",
        "/invoices",
        Some(&api_key),
        Some(r#"{"amount":1000,"currency":"ZMW","country":"ZM"}"#.into()),
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
    assert!(html.contains("id=\"root\""));
    assert!(html.contains("/assets/"));
}

#[tokio::test]
async fn swagger_docs_are_public() {
    let state = setup_test_state(Arc::new(MockGateway::success())).await;
    let app = test_router(state);

    let (status, body) = json_request(&app, "GET", "/swagger-ui/", None, None).await;
    assert_eq!(status, axum::http::StatusCode::OK);
    assert!(body.contains("swagger-ui"));

    let (status, yaml) = json_request(&app, "GET", "/api-docs/openapi.yaml", None, None).await;
    assert_eq!(status, axum::http::StatusCode::OK);
    assert!(yaml.contains("openapi:"));
    assert!(yaml.contains("/payments"));
}
