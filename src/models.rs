use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct System {
    pub id: Uuid,
    pub name: String,
    pub prefix: String,
    pub enabled_countries: Vec<String>,
    pub webhook_url: Option<String>,
    #[serde(skip_serializing)]
    pub api_key_hash: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct SystemPublic {
    pub id: Uuid,
    pub name: String,
    pub prefix: String,
    pub enabled_countries: Vec<String>,
    pub webhook_url: Option<String>,
}

impl From<System> for SystemPublic {
    fn from(s: System) -> Self {
        Self {
            id: s.id,
            name: s.name,
            prefix: s.prefix,
            enabled_countries: s.enabled_countries,
            webhook_url: s.webhook_url,
        }
    }
}

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct Wallet {
    pub id: Uuid,
    pub system_id: Uuid,
    pub country: String,
    pub currency: String,
    pub balance: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum TransactionStatus {
    Pending,
    Completed,
    Failed,
}

impl TransactionStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Completed => "completed",
            Self::Failed => "failed",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "pending" => Some(Self::Pending),
            "completed" => Some(Self::Completed),
            "failed" => Some(Self::Failed),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct Transaction {
    pub id: Uuid,
    pub system_id: Uuid,
    pub wallet_id: Uuid,
    pub external_id: String,
    pub idempotency_key: String,
    #[serde(skip_serializing)]
    pub request_hash: String,
    pub amount: i64,
    pub currency: String,
    pub country: String,
    pub status: String,
    pub gateway: String,
    pub gateway_reference: Option<String>,
    pub gateway_status: Option<String>,
    pub error: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentMethod {
    #[serde(rename = "type")]
    pub method_type: String,
    pub details: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessPaymentRequest {
    pub system_id: Uuid,
    pub external_id: String,
    pub amount: i64,
    pub currency: String,
    pub country: String,
    pub payment_method: PaymentMethod,
    pub idempotency_key: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProcessPaymentResponse {
    pub id: Uuid,
    pub external_id: String,
    pub status: String,
    pub gateway_reference: Option<String>,
    pub error: Option<String>,
}

impl From<Transaction> for ProcessPaymentResponse {
    fn from(t: Transaction) -> Self {
        Self {
            id: t.id,
            external_id: t.external_id,
            status: t.status,
            gateway_reference: t.gateway_reference,
            error: t.error,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateSystemRequest {
    pub name: String,
    pub prefix: String,
    pub enabled_countries: Vec<String>,
    pub webhook_url: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CreateSystemResponse {
    pub id: Uuid,
    pub name: String,
    pub prefix: String,
    pub api_key: String,
}

/// Warn-only validation per research.md — logs warning but does not reject.
pub fn validate_external_id_format(prefix: &str, external_id: &str) -> bool {
    let prefix_part = regex_like_prefix(prefix);
    external_id.starts_with(&prefix_part) && external_id.len() >= prefix.len() + 10
}

fn regex_like_prefix(prefix: &str) -> String {
    format!("{prefix}_")
}

pub fn external_id_format_valid(prefix: &str, external_id: &str) -> bool {
    let parts: Vec<&str> = external_id.split('_').collect();
    if parts.len() < 4 {
        return false;
    }
    parts[0] == prefix
        && parts[1].len() == 4
        && parts[2].len() == 8
        && parts[2].chars().all(|c| c.is_ascii_digit())
        && !parts[3].is_empty()
}
