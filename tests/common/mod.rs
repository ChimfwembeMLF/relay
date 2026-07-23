use std::sync::Arc;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use axum::Router;
use payment_relay::api::routes::create_router;
use payment_relay::config::Config;
use payment_relay::gateway::PaymentGateway;
use payment_relay::AppState;
use tower::ServiceExt;

pub async fn setup_test_state(gateway: Arc<dyn PaymentGateway>) -> AppState {
    dotenvy::dotenv().ok();
    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set for integration tests");
    let config = Config {
        database_url,
        redis_url: None,
        port: 8080,
        pawapay_api_token: String::new(),
        pawapay_base_url: "https://api.sandbox.pawapay.io".into(),
        webhook_signing_secret: "test-secret".into(),
        fallback_gateway: "mock".into(),
        invoice_pay_base_url: "http://localhost:8080".into(),
        wallet_seed_defaults: payment_relay::config::Config::from_env()
            .map(|c| c.wallet_seed_defaults)
            .unwrap_or_default(),
        pay_page_rate_limit: 100,
        admin_username: Some("admin".into()),
        admin_password: Some("admin-test-password".into()),
    };
    AppState::new(config, gateway).await.expect("failed to create test state")
}

pub fn test_router(state: AppState) -> Router {
    create_router(state)
}

pub async fn json_request(
    app: &Router,
    method: &str,
    uri: &str,
    api_key: Option<&str>,
    body: Option<String>,
) -> (StatusCode, String) {
    let mut builder = Request::builder().method(method).uri(uri);
    builder = builder.header("Content-Type", "application/json");
    if let Some(key) = api_key {
        builder = builder.header("X-API-Key", key);
    }
    let request = builder.body(Body::from(body.unwrap_or_default())).unwrap();
    let response = app.clone().oneshot(request).await.unwrap();
    let status = response.status();
    let bytes = http_body_util::BodyExt::collect(response.into_body())
        .await
        .unwrap()
        .to_bytes();
    (status, String::from_utf8_lossy(&bytes).to_string())
}

pub async fn form_request(
    app: &Router,
    method: &str,
    uri: &str,
    body: Option<String>,
) -> (StatusCode, String) {
    let builder = Request::builder()
        .method(method)
        .uri(uri)
        .header("Content-Type", "application/x-www-form-urlencoded");
    let request = builder.body(Body::from(body.unwrap_or_default())).unwrap();
    let response = app.clone().oneshot(request).await.unwrap();
    let status = response.status();
    let bytes = http_body_util::BodyExt::collect(response.into_body())
        .await
        .unwrap()
        .to_bytes();
    (status, String::from_utf8_lossy(&bytes).to_string())
}

pub async fn register_test_system(app: &Router) -> (String, String, String) {
    let suffix: String = uuid::Uuid::new_v4().to_string()[..4].to_uppercase();
    let prefix = format!("T{suffix}");
    let username = format!("user_{suffix}").to_lowercase();
    let body = format!(
        r#"{{
        "name": "Test System",
        "prefix": "{prefix}",
        "username": "{username}",
        "password": "testpass123",
        "enabled_countries": ["ZM"],
        "webhook_url": "https://example.com/webhook"
    }}"#
    );
    let (status, resp) = json_request(app, "POST", "/systems", None, Some(body)).await;
    assert_eq!(status, StatusCode::OK, "register failed: {resp}");
    let parsed: serde_json::Value = serde_json::from_str(&resp).unwrap();
    (
        parsed["id"].as_str().unwrap().to_string(),
        parsed["api_key"].as_str().unwrap().to_string(),
        parsed["prefix"].as_str().unwrap().to_string(),
    )
}

pub async fn seed_wallet(state: &AppState, system_id: uuid::Uuid, country: &str, currency: &str, balance: i64) {
    let wallet = payment_relay::db::queries::get_or_create_wallet(
        state.db.pool(),
        system_id,
        country,
        currency,
    )
    .await
    .unwrap();
    payment_relay::db::queries::seed_wallet_balance(state.db.pool(), wallet.id, balance)
        .await
        .unwrap();
}
