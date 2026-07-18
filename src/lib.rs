pub mod api;
pub mod auth;
pub mod config;
pub mod db;
pub mod error;
pub mod gateway;
pub mod models;
pub mod webhook;

use std::sync::Arc;

use crate::config::Config;
use crate::db::Db;
use crate::gateway::PaymentGateway;

#[derive(Clone)]
pub struct AppState {
    pub db: Db,
    pub config: Config,
    pub gateway: Arc<dyn PaymentGateway>,
}

impl AppState {
    pub async fn new(config: Config, gateway: Arc<dyn PaymentGateway>) -> Result<Self, error::AppError> {
        let db = Db::connect(&config.database_url).await?;
        db.migrate().await?;
        Ok(Self { db, config, gateway })
    }
}
