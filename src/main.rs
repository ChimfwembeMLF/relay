use std::sync::Arc;

use axum::Router;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use payment_relay::api;
use payment_relay::config::Config;
use payment_relay::gateway::pawapay::PawapayGateway;
use payment_relay::gateway::PaymentGateway;
use payment_relay::AppState;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();

    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
            "payment_relay=info,tower_http=info".into()
        }))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let config = Config::from_env()?;
    let gateway: Arc<dyn PaymentGateway> = Arc::new(PawapayGateway::new(
        config.pawapay_base_url.clone(),
        config.pawapay_api_token.clone(),
    ));

    let state = AppState::new(config.clone(), gateway).await?;
    let app = Router::new().merge(api::routes::create_router(state));

    let addr = format!("0.0.0.0:{}", config.port);
    tracing::info!("listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
