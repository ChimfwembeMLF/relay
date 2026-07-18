use std::env;

use crate::error::AppError;

#[derive(Clone, Debug)]
pub struct Config {
    pub database_url: String,
    pub port: u16,
    pub pawapay_api_token: String,
    pub pawapay_base_url: String,
    pub webhook_signing_secret: String,
    pub fallback_gateway: String,
}

impl Config {
    pub fn from_env() -> Result<Self, AppError> {
        Ok(Self {
            database_url: env::var("DATABASE_URL")
                .map_err(|_| AppError::Config("DATABASE_URL is required".into()))?,
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
        })
    }
}
