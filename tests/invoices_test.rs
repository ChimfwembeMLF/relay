mod common;

use std::sync::Arc;

use common::{json_request, register_test_system, setup_test_state, test_router};
use payment_relay::gateway::mock::MockGateway;
use uuid::Uuid;

#[tokio::test]
async fn invoice_create_collect_and_idempotency() {
    let state = setup_test_state(Arc::new(MockGateway::success())).await;
    let app = test_router(state.clone());

    let (system_id, api_key, _prefix) = register_test_system(&app).await;

    let create_body = r#"{
        "amount": 5000,
        "currency": "ZMW",
        "country": "ZM",
        "description": "Test sale",
        "expires_in_hours": 24
    }"#;

    let (status, resp) =
        json_request(&app, "POST", "/invoices", Some(&api_key), Some(create_body.into())).await;
    assert_eq!(status, axum::http::StatusCode::OK, "create failed: {resp}");

    let invoice: serde_json::Value = serde_json::from_str(&resp).unwrap();
    assert_eq!(invoice["status"], "open");
    assert!(invoice["qr_url"].as_str().unwrap().contains("/pay/"));
    assert!(!invoice["qr_code_png_base64"].as_str().unwrap().is_empty());

    let invoice_id = invoice["id"].as_str().unwrap();
    let reference = invoice["reference"].as_str().unwrap();

    let (status, by_ref) = json_request(
        &app,
        "GET",
        &format!("/invoices/reference/{reference}"),
        Some(&api_key),
        None,
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::OK);
    assert_eq!(serde_json::from_str::<serde_json::Value>(&by_ref).unwrap()["id"], invoice["id"]);

    let idem_key = format!("collect-{}", Uuid::new_v4());
    let collect_body = format!(
        r#"{{
            "idempotency_key": "{idem_key}",
            "payment_method": {{
                "type": "mmo",
                "details": {{
                    "provider": "MTN_MOMO_ZMB",
                    "phoneNumber": "260763456789"
                }}
            }}
        }}"#
    );

    let (status, collect_resp) = json_request(
        &app,
        "POST",
        &format!("/invoices/{invoice_id}/collect"),
        Some(&api_key),
        Some(collect_body.clone()),
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::OK, "collect failed: {collect_resp}");
    assert!(collect_resp.contains("\"status\":\"completed\""));

    let (status, replay) = json_request(
        &app,
        "POST",
        &format!("/invoices/{invoice_id}/collect"),
        Some(&api_key),
        Some(collect_body),
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::OK);
    let first: serde_json::Value = serde_json::from_str(&collect_resp).unwrap();
    let second: serde_json::Value = serde_json::from_str(&replay).unwrap();
    assert_eq!(first["id"], second["id"]);

    let (_, wallets) =
        json_request(&app, "GET", &format!("/wallets/{system_id}"), Some(&api_key), None).await;
    let wallets: serde_json::Value = serde_json::from_str(&wallets).unwrap();
    let zm = wallets
        .as_array()
        .unwrap()
        .iter()
        .find(|w| w["country"] == "ZM")
        .unwrap();
    assert!(zm["balance"].as_i64().unwrap() >= 105_000);
}

#[tokio::test]
async fn cancel_open_invoice() {
    let state = setup_test_state(Arc::new(MockGateway::success())).await;
    let app = test_router(state);

    let (_system_id, api_key, _prefix) = register_test_system(&app).await;

    let create_body = r#"{
        "amount": 1000,
        "currency": "ZMW",
        "country": "ZM"
    }"#;

    let (_, resp) =
        json_request(&app, "POST", "/invoices", Some(&api_key), Some(create_body.into())).await;
    let invoice: serde_json::Value = serde_json::from_str(&resp).unwrap();
    let invoice_id = invoice["id"].as_str().unwrap();

    let (status, cancel_resp) = json_request(
        &app,
        "POST",
        &format!("/invoices/{invoice_id}/cancel"),
        Some(&api_key),
        None,
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::OK);
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&cancel_resp).unwrap()["status"],
        "cancelled"
    );
}
