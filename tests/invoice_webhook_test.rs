mod common;

use std::sync::Arc;

use common::setup_test_state;
use payment_relay::auth::{generate_api_key, hash_api_key};
use payment_relay::db::invoices::{create_invoice, get_invoice_by_reference_public, mark_invoice_paid};
use payment_relay::db::queries::{self, NewTransaction};
use payment_relay::gateway::mock::MockGateway;
use payment_relay::webhook::sender;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn deliver_invoice_paid_webhook() {
    let mock_server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/invoice-hook"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&mock_server)
        .await;

    let state = setup_test_state(Arc::new(MockGateway::success())).await;

    let api_key = generate_api_key();
    let suffix: String = uuid::Uuid::new_v4().to_string()[..4].to_uppercase();
    let prefix = format!("W{suffix}");
    let webhook_url = format!("{}/invoice-hook", mock_server.uri());
    let system = queries::create_system(
        state.db.pool(),
        "Webhook Invoice Test",
        &prefix,
        &["ZM".to_string()],
        Some(&webhook_url),
        &hash_api_key(&api_key),
    )
    .await
    .unwrap();

    let reference = format!("INV_{prefix}_WEBHOOK1");
    let invoice = create_invoice(
        state.db.pool(),
        system.id,
        &reference,
        Some("test"),
        1000,
        "ZMW",
        "ZM",
        chrono::Utc::now() + chrono::Duration::hours(1),
        &format!("http://localhost/pay/{reference}"),
    )
    .await
    .unwrap();

    let wallet = queries::get_or_create_wallet(state.db.pool(), system.id, "ZM", "ZMW")
        .await
        .unwrap();

    queries::seed_wallet_balance(state.db.pool(), wallet.id, 0)
        .await
        .unwrap();

    let tx = NewTransaction {
        system_id: system.id,
        wallet_id: wallet.id,
        external_id: &reference,
        idempotency_key: "inv-wh",
        request_hash: "hash",
        amount: 1000,
        currency: "ZMW",
        country: "ZM",
        status: "completed",
        gateway: "mock",
        gateway_reference: Some("dep-1"),
        gateway_status: Some("ACCEPTED"),
        error: None,
        invoice_id: Some(invoice.id),
        direction: "deposit",
    };
    let transaction = payment_relay::db::invoices::create_deposit_with_credit(
        state.db.pool(),
        wallet.id,
        1000,
        tx,
    )
    .await
    .unwrap();

    mark_invoice_paid(state.db.pool(), invoice.id, transaction.id)
        .await
        .unwrap();
    let invoice = get_invoice_by_reference_public(state.db.pool(), &reference)
        .await
        .unwrap();

    sender::deliver_invoice_webhook(&state, &webhook_url, &invoice, &transaction)
        .await
        .expect("invoice webhook should succeed");

    let row: (String,) = sqlx::query_as(
        "SELECT event_type FROM webhook_delivery_attempts WHERE transaction_id = $1 LIMIT 1",
    )
    .bind(transaction.id)
    .fetch_one(state.db.pool())
    .await
    .unwrap();

    assert_eq!(row.0, "invoice.paid");
}
