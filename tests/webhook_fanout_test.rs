mod common;

use std::sync::Arc;

use common::setup_test_state;
use payment_relay::auth::{generate_api_key, hash_api_key};
use payment_relay::db::queries::{self, NewTransaction};
use payment_relay::gateway::mock::MockGateway;
use payment_relay::webhook::sender;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn broadcast_payment_webhook_to_all_subscribers() {
    let mock_a = MockServer::start().await;
    let mock_b = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/a"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&mock_a)
        .await;

    Mock::given(method("POST"))
        .and(path("/b"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&mock_b)
        .await;

    let state = setup_test_state(Arc::new(MockGateway::success())).await;

    let suffix: String = uuid::Uuid::new_v4().to_string()[..4].to_uppercase();
    let system_a = queries::create_system(
        state.db.pool(),
        "Merchant A",
        &format!("A{suffix}"),
        &["ZM".to_string()],
        Some(&format!("{}/a", mock_a.uri())),
        &hash_api_key(&generate_api_key()),
    )
    .await
    .unwrap();

    let _system_b = queries::create_system(
        state.db.pool(),
        "Merchant B",
        &format!("B{suffix}"),
        &["ZM".to_string()],
        Some(&format!("{}/b", mock_b.uri())),
        &hash_api_key(&generate_api_key()),
    )
    .await
    .unwrap();

    // No webhook — should be skipped
    let _silent = queries::create_system(
        state.db.pool(),
        "Silent",
        &format!("S{suffix}"),
        &["ZM".to_string()],
        None,
        &hash_api_key(&generate_api_key()),
    )
    .await
    .unwrap();

    let wallet = queries::get_or_create_wallet(state.db.pool(), system_a.id, "ZM", "ZMW")
        .await
        .unwrap();
    queries::seed_wallet_balance(state.db.pool(), wallet.id, 100_000)
        .await
        .unwrap();

    let transaction = queries::create_transaction_with_debit(
        state.db.pool(),
        wallet.id,
        1000,
        NewTransaction {
            system_id: system_a.id,
            wallet_id: wallet.id,
            external_id: "FAN_550e_20260723_OUT1",
            idempotency_key: "fanout-test",
            request_hash: "abc",
            amount: 1000,
            currency: "ZMW",
            country: "ZM",
            status: "completed",
            gateway: "mock",
            gateway_reference: Some("ref-fan"),
            gateway_status: Some("ACCEPTED"),
            error: None,
            invoice_id: None,
            direction: "payout",
        },
    )
    .await
    .unwrap();

    sender::broadcast_payment_webhook(&state, &transaction)
        .await
        .expect("fan-out should succeed");
}
