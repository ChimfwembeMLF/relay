use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::Path;

use serde::Deserialize;

use crate::error::AppError;

#[derive(Clone, Debug, Deserialize)]
pub struct CountrySeedDefault {
    pub currency: String,
    pub amount: i64,
}

pub type WalletSeedDefaults = HashMap<String, CountrySeedDefault>;

#[derive(Clone, Debug)]
pub struct Config {
    pub database_url: String,
    pub redis_url: Option<String>,
    pub port: u16,
    pub pawapay_api_token: String,
    pub pawapay_base_url: String,
    pub webhook_signing_secret: String,
    pub fallback_gateway: String,
    pub invoice_pay_base_url: String,
    pub wallet_seed_defaults: WalletSeedDefaults,
    pub pay_page_rate_limit: u32,
    /// Bootstrap credentials for the platform backoffice (`POST /admin/login`).
    pub admin_username: Option<String>,
    pub admin_password: Option<String>,
}

impl Config {
    pub fn from_env() -> Result<Self, AppError> {
        let wallet_seed_defaults = load_wallet_seed_defaults()?;

        let redis_url = env::var("REDIS_URL").ok().filter(|s| !s.trim().is_empty());

        Ok(Self {
            database_url: env::var("DATABASE_URL")
                .map_err(|_| AppError::Config("DATABASE_URL is required".into()))?,
            redis_url,
            port: env::var("PORT")
                .unwrap_or_else(|_| "8080".into())
                .parse()
                .map_err(|_| AppError::Config("PORT must be a valid u16".into()))?,
            pawapay_api_token: env::var("PAWAPAY_API_TOKEN").unwrap_or_default(),
            pawapay_base_url: env::var("PAWAPAY_BASE_URL")
                .unwrap_or_else(|_| "https://api.sandbox.pawapay.io".into()),
            webhook_signing_secret: env::var("WEBHOOK_SIGNING_SECRET")
                .unwrap_or_else(|_| "dev-secret-change-me".into()),
            fallback_gateway: env::var("FALLBACK_GATEWAY")
                .unwrap_or_else(|_| "pawapay".into()),
            invoice_pay_base_url: env::var("INVOICE_PAY_BASE_URL")
                .unwrap_or_else(|_| "http://localhost:8080".into()),
            wallet_seed_defaults,
            pay_page_rate_limit: env::var("PAY_PAGE_RATE_LIMIT")
                .unwrap_or_else(|_| "10".into())
                .parse()
                .map_err(|_| AppError::Config("PAY_PAGE_RATE_LIMIT must be a valid u32".into()))?,
            admin_username: env::var("ADMIN_USERNAME").ok().filter(|s| !s.trim().is_empty()),
            admin_password: env::var("ADMIN_PASSWORD").ok().filter(|s| !s.trim().is_empty()),
        })
    }
}

fn load_wallet_seed_defaults() -> Result<WalletSeedDefaults, AppError> {
    if let Ok(json) = env::var("WALLET_SEED_DEFAULTS_JSON") {
        return serde_json::from_str(&json)
            .map_err(|e| AppError::Config(format!("invalid WALLET_SEED_DEFAULTS_JSON: {e}")));
    }

    let path = env::var("WALLET_SEED_DEFAULTS_PATH")
        .unwrap_or_else(|_| "config/wallet_seed_defaults.json".into());

    if Path::new(&path).exists() {
        let content = fs::read_to_string(&path)
            .map_err(|e| AppError::Config(format!("failed to read {path}: {e}")))?;
        return serde_json::from_str(&content)
            .map_err(|e| AppError::Config(format!("invalid wallet seed defaults file: {e}")));
    }

    Ok(HashMap::new())
}
