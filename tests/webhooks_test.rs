mod common;

use std::sync::Arc;

use common::setup_test_state;
use payment_relay::auth::{generate_api_key, hash_api_key};
use payment_relay::db::queries::{self, NewTransaction};
use payment_relay::gateway::mock::MockGateway;
use payment_relay::webhook::sender;
use uuid::Uuid;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn deliver_signed_webhook_on_payment() {
    let mock_server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/hook"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&mock_server)
        .await;

    let state = setup_test_state(Arc::new(MockGateway::success())).await;

    let api_key = generate_api_key();
    let suffix: String = uuid::Uuid::new_v4().to_string()[..4].to_uppercase();
    let prefix = format!("W{suffix}");
    let system = queries::create_system(
        state.db.pool(),
        "Webhook Test",
        &prefix,
        &["ZM".to_string()],
        None,
        &hash_api_key(&api_key),
    )
    .await
    .unwrap();

    let wallet = queries::get_or_create_wallet(state.db.pool(), system.id, "ZM", "ZMW")
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
            system_id: system.id,
            wallet_id: wallet.id,
            external_id: "WHK_550e_20260718_HOOK1",
            idempotency_key: "hook-test",
            request_hash: "abc",
            amount: 1000,
            currency: "ZMW",
            country: "ZM",
            status: "completed",
            gateway: "mock",
            gateway_reference: Some("ref-123"),
            gateway_status: Some("ACCEPTED"),
            error: None,
        },
    )
    .await
    .unwrap();

    let url = format!("{}/hook", mock_server.uri());
    sender::deliver_payment_webhook(&state, &url, &transaction)
        .await
        .expect("webhook delivery should succeed");

    let row: (bool,) = sqlx::query_as(
        "SELECT success FROM webhook_delivery_attempts WHERE transaction_id = $1 LIMIT 1",
    )
    .bind(transaction.id)
    .fetch_one(state.db.pool())
    .await
    .unwrap();

    assert!(row.0);
}
