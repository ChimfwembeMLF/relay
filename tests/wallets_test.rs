mod common;

use std::sync::Arc;

use common::{json_request, register_test_system, seed_wallet, setup_test_state, test_router};
use payment_relay::gateway::mock::MockGateway;
use uuid::Uuid;

#[tokio::test]
async fn list_wallets_for_system() {
    let state = setup_test_state(Arc::new(MockGateway::success())).await;
    let app = test_router(state.clone());

    let (system_id, api_key, _prefix) = register_test_system(&app).await;
    let system_uuid = Uuid::parse_str(&system_id).unwrap();
    seed_wallet(&state, system_uuid, "ZM", "ZMW", 50_000).await;

    let (status, resp) = json_request(
        &app,
        "GET",
        &format!("/wallets/{system_id}"),
        Some(&api_key),
        None,
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::OK);
    assert!(resp.contains("\"balance\":50000"));
    assert!(resp.contains("\"country\":\"ZM\""));
}
