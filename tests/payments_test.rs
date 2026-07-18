mod common;

use std::sync::Arc;

use common::{json_request, register_test_system, seed_wallet, setup_test_state, test_router};
use payment_relay::gateway::mock::MockGateway;
use uuid::Uuid;

#[tokio::test]
async fn process_payment_happy_path() {
    let state = setup_test_state(Arc::new(MockGateway::success())).await;
    let app = test_router(state.clone());

    let (system_id, api_key, prefix) = register_test_system(&app).await;
    let system_uuid = Uuid::parse_str(&system_id).unwrap();
    seed_wallet(&state, system_uuid, "ZM", "ZMW", 100_000).await;

    let body = format!(
        r#"{{
            "system_id": "{system_id}",
            "external_id": "{prefix}_550e_20260718_ABC123",
            "amount": 1500,
            "currency": "ZMW",
            "country": "ZM",
            "idempotency_key": "pay-{}",
            "payment_method": {{
                "type": "mmo",
                "details": {{
                    "provider": "MTN_MOMO_ZMB",
                    "phoneNumber": "260763456789"
                }}
            }}
        }}"#,
        Uuid::new_v4()
    );

    let (status, resp) = json_request(&app, "POST", "/payments", Some(&api_key), Some(body)).await;
    assert_eq!(status, axum::http::StatusCode::OK, "payment failed: {resp}");
    assert!(resp.contains("\"status\":\"completed\""));
    assert!(resp.contains("gateway_reference"));
}
