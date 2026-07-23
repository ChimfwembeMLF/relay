mod common;

use std::sync::Arc;

use common::{json_request, register_test_system, seed_wallet, setup_test_state, test_router};
use payment_relay::gateway::mock::MockGateway;
use uuid::Uuid;

#[tokio::test]
async fn batch_partial_success_and_idempotency() {
    let state = setup_test_state(Arc::new(MockGateway::success())).await;
    let app = test_router(state.clone());

    let (system_id, api_key, prefix) = register_test_system(&app).await;
    let system_uuid = Uuid::parse_str(&system_id).unwrap();
    seed_wallet(&state, system_uuid, "ZM", "ZMW", 100_000).await;

    let idem = Uuid::new_v4().to_string();
    let body = serde_json::json!({
        "system_id": system_id,
        "idempotency_key": idem,
        "lines": [
            {
                "amount": 1000,
                "currency": "ZMW",
                "country": "ZM",
                "external_id": format!("{prefix}_20260723_aaa"),
                "payment_method": { "type": "mmo", "details": { "phone": "260763456789", "provider": "MTN_MOMO_ZMB" } }
            },
            {
                "amount": 2000,
                "currency": "ZMW",
                "country": "ZM",
                "payment_method": { "type": "mmo", "details": { "phone": "260763456780", "provider": "AIRTEL_OAPI_ZMB" } }
            },
            {
                "amount": 500,
                "currency": "ZMW",
                "country": "ZM",
                "payment_method": { "type": "mmo", "details": { "phone": "", "provider": "MTN_MOMO_ZMB" } }
            }
        ]
    })
    .to_string();

    let (status, resp) = json_request(&app, "POST", "/batches", Some(&api_key), Some(body.clone())).await;
    assert_eq!(status, axum::http::StatusCode::OK, "batch failed: {resp}");
    let parsed: serde_json::Value = serde_json::from_str(&resp).unwrap();
    assert_eq!(parsed["success_count"], 2);
    assert_eq!(parsed["failure_count"], 1);
    assert_eq!(parsed["status"], "partial");

    let (status2, resp2) =
        json_request(&app, "POST", "/batches", Some(&api_key), Some(body)).await;
    assert_eq!(status2, axum::http::StatusCode::OK);
    let parsed2: serde_json::Value = serde_json::from_str(&resp2).unwrap();
    assert_eq!(parsed2["id"], parsed["id"]);
    assert_eq!(parsed2["success_count"], 2);
}
