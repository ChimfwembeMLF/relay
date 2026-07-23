mod common;

use std::sync::Arc;

use common::{json_request, register_test_system, seed_wallet, setup_test_state, test_router};
use payment_relay::gateway::mock::MockGateway;
use uuid::Uuid;

#[tokio::test]
async fn invoice_partial_refund_and_guards() {
    let state = setup_test_state(Arc::new(MockGateway::success())).await;
    let app = test_router(state.clone());

    let (system_id, api_key, _prefix) = register_test_system(&app).await;
    let system_uuid = Uuid::parse_str(&system_id).unwrap();
    seed_wallet(&state, system_uuid, "ZM", "ZMW", 50_000).await;

    let (_, inv_resp) = json_request(
        &app,
        "POST",
        "/invoices",
        Some(&api_key),
        Some(r#"{"amount":5000,"currency":"ZMW","country":"ZM"}"#.into()),
    )
    .await;
    let invoice: serde_json::Value = serde_json::from_str(&inv_resp).unwrap();
    let invoice_id = invoice["id"].as_str().unwrap();

    let collect_body = serde_json::json!({
        "payment_method": {
            "type": "mmo",
            "details": { "phone": "260763456789", "provider": "MTN_MOMO_ZMB" }
        },
        "idempotency_key": Uuid::new_v4().to_string()
    })
    .to_string();
    let (c_status, c_resp) = json_request(
        &app,
        "POST",
        &format!("/invoices/{invoice_id}/collect"),
        Some(&api_key),
        Some(collect_body),
    )
    .await;
    assert_eq!(c_status, axum::http::StatusCode::OK, "collect failed: {c_resp}");

    let refund_key = Uuid::new_v4().to_string();
    let refund_body = serde_json::json!({
        "amount": 1500,
        "idempotency_key": refund_key
    })
    .to_string();
    let (r_status, r_resp) = json_request(
        &app,
        "POST",
        &format!("/invoices/{invoice_id}/refund"),
        Some(&api_key),
        Some(refund_body.clone()),
    )
    .await;
    assert_eq!(r_status, axum::http::StatusCode::OK, "refund failed: {r_resp}");
    let refund: serde_json::Value = serde_json::from_str(&r_resp).unwrap();
    assert_eq!(refund["invoice"]["refunded_amount"], 1500);
    assert_eq!(refund["invoice"]["remaining_refundable"], 3500);
    assert_eq!(refund["invoice"]["status"], "paid");

    let (r2_status, r2_resp) = json_request(
        &app,
        "POST",
        &format!("/invoices/{invoice_id}/refund"),
        Some(&api_key),
        Some(refund_body),
    )
    .await;
    assert_eq!(r2_status, axum::http::StatusCode::OK);
    let refund2: serde_json::Value = serde_json::from_str(&r2_resp).unwrap();
    assert_eq!(refund2["id"], refund["id"]);

    let over = serde_json::json!({
        "amount": 4000,
        "idempotency_key": Uuid::new_v4().to_string()
    })
    .to_string();
    let (o_status, _) = json_request(
        &app,
        "POST",
        &format!("/invoices/{invoice_id}/refund"),
        Some(&api_key),
        Some(over),
    )
    .await;
    assert_eq!(o_status, axum::http::StatusCode::BAD_REQUEST);
}
