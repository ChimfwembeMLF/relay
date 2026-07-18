mod common;

use std::sync::Arc;

use common::{json_request, register_test_system, setup_test_state, test_router};
use payment_relay::gateway::mock::MockGateway;

#[tokio::test]
async fn register_system_and_reject_invalid_api_key() {
    let state = setup_test_state(Arc::new(MockGateway::success())).await;
    let app = test_router(state);

    let (system_id, api_key, _prefix) = register_test_system(&app).await;

    let (status, body) = json_request(
        &app,
        "GET",
        &format!("/systems/{system_id}"),
        None,
        None,
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::OK);
    assert!(body.contains("Test System"));

    let (status, _) = json_request(
        &app,
        "GET",
        &format!("/wallets/{system_id}"),
        Some("sk_live_invalid"),
        None,
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::UNAUTHORIZED);

    let (status, _) = json_request(
        &app,
        "GET",
        &format!("/wallets/{system_id}"),
        Some(&api_key),
        None,
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::OK);
}
