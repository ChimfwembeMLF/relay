pub mod api;
pub mod auth;
pub mod catalog;
pub mod config;
pub mod db;
pub mod error;
pub mod gateway;
pub mod models;
pub mod pay;
pub mod qr;
pub mod queue;
pub mod seed;
pub mod webhook;

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use redis::aio::ConnectionManager;

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
    pub redis: Option<ConnectionManager>,
}

impl AppState {
    pub async fn new(config: Config, gateway: Arc<dyn PaymentGateway>) -> Result<Self, error::AppError> {
        let db = Db::connect(&config.database_url).await?;
        db.migrate().await?;

        let redis = if let Some(url) = &config.redis_url {
            match queue::connect(url).await {
                Ok(conn) => {
                    tracing::info!("connected to Redis for job queues");
                    Some(conn)
                }
                Err(e) => {
                    tracing::error!(error = %e, "REDIS_URL set but connection failed");
                    return Err(e);
                }
            }
        } else {
            tracing::warn!("REDIS_URL not set — webhooks run in-process (no BullMQ)");
            None
        };

        let state = Self {
            db,
            config: config.clone(),
            gateway,
            pay_rate_limiter: PayRateLimiter::new(config.pay_page_rate_limit),
            redis,
        };
        bootstrap_admin(&state).await?;
        Ok(state)
    }
}

/// Ensure a bootstrap admin exists when `ADMIN_USERNAME` + `ADMIN_PASSWORD` are set.
pub async fn bootstrap_admin(state: &AppState) -> Result<(), error::AppError> {
    let (Some(username), Some(password)) = (
        state.config.admin_username.as_deref(),
        state.config.admin_password.as_deref(),
    ) else {
        tracing::warn!("ADMIN_USERNAME/ADMIN_PASSWORD not set — backoffice login disabled until seeded");
        return Ok(());
    };

    let user = db::admins::upsert_admin_user(state.db.pool(), username, password).await?;
    tracing::info!(username = %user.username, "platform admin user ready");
    Ok(())
}
