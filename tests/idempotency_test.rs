mod common;

use std::sync::Arc;

use common::{json_request, register_test_system, seed_wallet, setup_test_state, test_router};
use payment_relay::gateway::mock::MockGateway;
use uuid::Uuid;

#[tokio::test]
async fn idempotency_replay_returns_same_result() {
    let state = setup_test_state(Arc::new(MockGateway::success())).await;
    let app = test_router(state.clone());

    let (system_id, api_key, prefix) = register_test_system(&app).await;
    let system_uuid = Uuid::parse_str(&system_id).unwrap();
    seed_wallet(&state, system_uuid, "ZM", "ZMW", 100_000).await;

    let idempotency_key = format!("idem-{}", Uuid::new_v4());
    let body = format!(
        r#"{{
            "system_id": "{system_id}",
            "external_id": "{prefix}_550e_20260718_IDEM01",
            "amount": 1500,
            "currency": "ZMW",
            "country": "ZM",
            "idempotency_key": "{idempotency_key}",
            "payment_method": {{
                "type": "mmo",
                "details": {{ "provider": "MTN_MOMO_ZMB", "phoneNumber": "260763456789" }}
            }}
        }}"#
    );

    let (status1, resp1) =
        json_request(&app, "POST", "/payments", Some(&api_key), Some(body.clone())).await;
    assert_eq!(status1, axum::http::StatusCode::OK);

    let (status2, resp2) = json_request(&app, "POST", "/payments", Some(&api_key), Some(body)).await;
    assert_eq!(status2, axum::http::StatusCode::OK);
    assert_eq!(resp1, resp2);

    let conflict_body = format!(
        r#"{{
            "system_id": "{system_id}",
            "external_id": "{prefix}_550e_20260718_IDEM01",
            "amount": 9999,
            "currency": "ZMW",
            "country": "ZM",
            "idempotency_key": "{idempotency_key}",
            "payment_method": {{
                "type": "mmo",
                "details": {{ "provider": "MTN_MOMO_ZMB", "phoneNumber": "260763456789" }}
            }}
        }}"#
    );
    let (status3, _) =
        json_request(&app, "POST", "/payments", Some(&api_key), Some(conflict_body)).await;
    assert_eq!(status3, axum::http::StatusCode::CONFLICT);
}
