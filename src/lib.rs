pub mod api;
pub mod auth;
pub mod config;
pub mod db;
pub mod error;
pub mod gateway;
pub mod models;
pub mod pay;
pub mod qr;
pub mod seed;
pub mod webhook;

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use crate::config::Config;
use crate::db::Db;
use crate::gateway::PaymentGateway;

#[derive(Clone)]
pub struct PayRateLimiter {
    inner: Arc<Mutex<HashMap<String, Vec<Instant>>>>,
    limit: u32,
    window: Duration,
}

impl PayRateLimiter {
    pub fn new(limit: u32) -> Self {
        Self {
            inner: Arc::new(Mutex::new(HashMap::new())),
            limit,
            window: Duration::from_secs(60),
        }
    }

    pub fn check(&self, key: &str) -> bool {
        let now = Instant::now();
        let mut guard = self.inner.lock().expect("rate limiter lock");
        let entries = guard.entry(key.to_string()).or_default();
        entries.retain(|t| now.duration_since(*t) < self.window);
        if entries.len() as u32 >= self.limit {
            return false;
        }
        entries.push(now);
        true
    }
}

#[derive(Clone)]
pub struct AppState {
    pub db: Db,
    pub config: Config,
    pub gateway: Arc<dyn PaymentGateway>,
    pub pay_rate_limiter: PayRateLimiter,
}

impl AppState {
    pub async fn new(config: Config, gateway: Arc<dyn PaymentGateway>) -> Result<Self, error::AppError> {
        let db = Db::connect(&config.database_url).await?;
        db.migrate().await?;
        Ok(Self {
            db,
            config: config.clone(),
            gateway,
            pay_rate_limiter: PayRateLimiter::new(config.pay_page_rate_limit),
        })
    }
}
