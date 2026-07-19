mod common;

use std::sync::Arc;

use common::{json_request, setup_test_state, test_router};
use payment_relay::gateway::mock::MockGateway;
use uuid::Uuid;

#[tokio::test]
async fn auto_seeds_wallets_on_registration() {
    let state = setup_test_state(Arc::new(MockGateway::success())).await;
    let app = test_router(state);

    let suffix: String = Uuid::new_v4().to_string()[..4].to_uppercase();
    let prefix = format!("S{suffix}");
    let body = format!(
        r#"{{
        "name": "Seed Test",
        "prefix": "{prefix}",
        "enabled_countries": ["ZM", "US"],
        "webhook_url": "https://example.com/webhook"
    }}"#
    );

    let (status, resp) = json_request(&app, "POST", "/systems", None, Some(body)).await;
    assert_eq!(status, axum::http::StatusCode::OK, "register failed: {resp}");

    let parsed: serde_json::Value = serde_json::from_str(&resp).unwrap();
    assert_eq!(parsed["wallets_seeded"].as_u64(), Some(2));

    let system_id = parsed["id"].as_str().unwrap();
    let api_key = parsed["api_key"].as_str().unwrap();

    let (status, wallets) =
        json_request(&app, "GET", &format!("/wallets/{system_id}"), Some(api_key), None).await;
    assert_eq!(status, axum::http::StatusCode::OK);
    let wallets: serde_json::Value = serde_json::from_str(&wallets).unwrap();
    assert_eq!(wallets.as_array().unwrap().len(), 2);

    let zm = wallets
        .as_array()
        .unwrap()
        .iter()
        .find(|w| w["country"] == "ZM")
        .expect("ZM wallet missing");
    assert_eq!(zm["balance"].as_i64(), Some(100_000));
}

#[tokio::test]
async fn wallet_seed_override_takes_precedence() {
    let state = setup_test_state(Arc::new(MockGateway::success())).await;
    let app = test_router(state);

    let suffix: String = Uuid::new_v4().to_string()[..4].to_uppercase();
    let prefix = format!("O{suffix}");
    let body = format!(
        r#"{{
        "name": "Override Test",
        "prefix": "{prefix}",
        "enabled_countries": ["ZM", "US"],
        "wallet_seeds": [
            {{ "country": "ZM", "currency": "ZMW", "amount": 200000 }}
        ]
    }}"#
    );

    let (status, resp) = json_request(&app, "POST", "/systems", None, Some(body)).await;
    assert_eq!(status, axum::http::StatusCode::OK, "register failed: {resp}");

    let parsed: serde_json::Value = serde_json::from_str(&resp).unwrap();
    let system_id = parsed["id"].as_str().unwrap();
    let api_key = parsed["api_key"].as_str().unwrap();

    let (_, wallets) =
        json_request(&app, "GET", &format!("/wallets/{system_id}"), Some(api_key), None).await;
    let wallets: serde_json::Value = serde_json::from_str(&wallets).unwrap();

    let zm = wallets
        .as_array()
        .unwrap()
        .iter()
        .find(|w| w["country"] == "ZM")
        .unwrap();
    assert_eq!(zm["balance"].as_i64(), Some(200_000));

    let us = wallets
        .as_array()
        .unwrap()
        .iter()
        .find(|w| w["country"] == "US")
        .unwrap();
    assert_eq!(us["balance"].as_i64(), Some(10_000));
}

#[tokio::test]
async fn rejects_override_for_disabled_country() {
    let state = setup_test_state(Arc::new(MockGateway::success())).await;
    let app = test_router(state);

    let suffix: String = Uuid::new_v4().to_string()[..4].to_uppercase();
    let prefix = format!("R{suffix}");
    let body = format!(
        r#"{{
        "name": "Reject Test",
        "prefix": "{prefix}",
        "enabled_countries": ["ZM"],
        "wallet_seeds": [
            {{ "country": "US", "currency": "USD", "amount": 5000 }}
        ]
    }}"#
    );

    let (status, _) = json_request(&app, "POST", "/systems", None, Some(body)).await;
    assert_eq!(status, axum::http::StatusCode::BAD_REQUEST);
}
