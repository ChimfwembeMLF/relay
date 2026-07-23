//! Redis job enqueue for BullMQ workers.
//!
//! Jobs are written to Redis lists that the Node BullMQ worker drains into
//! proper BullMQ queues (`webhooks`). Without `REDIS_URL`, callers should
//! fall back to in-process delivery.

use redis::aio::ConnectionManager;
use redis::AsyncCommands;
use serde::Serialize;
use uuid::Uuid;

use crate::error::AppError;

/// Inbox list drained by `workers/` into the BullMQ `webhooks` queue.
pub const WEBHOOKS_INBOX: &str = "relay:inbox:webhooks";

#[derive(Debug, Clone, Serialize)]
pub struct WebhookJob {
    pub job_id: String,
    pub url: String,
    pub body: String,
    pub signature: String,
    pub transaction_id: Uuid,
    pub event_type: String,
}

pub async fn connect(redis_url: &str) -> Result<ConnectionManager, AppError> {
    let client = redis::Client::open(redis_url)
        .map_err(|e| AppError::Config(format!("invalid REDIS_URL: {e}")))?;
    ConnectionManager::new(client)
        .await
        .map_err(|e| AppError::Config(format!("redis connect failed: {e}")))
}

pub async fn enqueue_webhook(
    redis: &mut ConnectionManager,
    job: &WebhookJob,
) -> Result<(), AppError> {
    let payload = serde_json::to_string(job)
        .map_err(|e| AppError::Internal(format!("queue serialize failed: {e}")))?;
    let _: i64 = redis
        .lpush(WEBHOOKS_INBOX, payload)
        .await
        .map_err(|e| AppError::Internal(format!("redis enqueue failed: {e}")))?;
    Ok(())
}
