use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};
use uuid::Uuid;

use crate::error::AppError;

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct WebhookEndpoint {
    pub id: Uuid,
    pub system_id: Uuid,
    pub url: String,
    pub label: Option<String>,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateWebhookEndpointRequest {
    pub url: String,
    pub label: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateWebhookEndpointRequest {
    pub url: Option<String>,
    pub label: Option<String>,
    pub enabled: Option<bool>,
}

pub async fn list_endpoints(pool: &PgPool, system_id: Uuid) -> Result<Vec<WebhookEndpoint>, AppError> {
    let rows = sqlx::query_as::<_, WebhookEndpoint>(
        r#"
        SELECT id, system_id, url, label, enabled, created_at, updated_at
        FROM webhook_endpoints
        WHERE system_id = $1
        ORDER BY created_at ASC
        "#,
    )
    .bind(system_id)
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

pub async fn create_endpoint(
    pool: &PgPool,
    system_id: Uuid,
    url: &str,
    label: Option<&str>,
) -> Result<WebhookEndpoint, AppError> {
    sqlx::query_as::<_, WebhookEndpoint>(
        r#"
        INSERT INTO webhook_endpoints (system_id, url, label)
        VALUES ($1, $2, $3)
        RETURNING id, system_id, url, label, enabled, created_at, updated_at
        "#,
    )
    .bind(system_id)
    .bind(url)
    .bind(label)
    .fetch_one(pool)
    .await
    .map_err(|e| {
        if let sqlx::Error::Database(db) = &e {
            if db.constraint().is_some_and(|c| c.contains("webhook_endpoints")) {
                return AppError::Conflict("webhook URL already registered for this system".into());
            }
        }
        AppError::Database(e)
    })
}

pub async fn create_endpoint_in_tx(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    system_id: Uuid,
    url: &str,
    label: Option<&str>,
) -> Result<WebhookEndpoint, AppError> {
    let row = sqlx::query_as::<_, WebhookEndpoint>(
        r#"
        INSERT INTO webhook_endpoints (system_id, url, label)
        VALUES ($1, $2, $3)
        ON CONFLICT (system_id, url) DO UPDATE SET updated_at = NOW()
        RETURNING id, system_id, url, label, enabled, created_at, updated_at
        "#,
    )
    .bind(system_id)
    .bind(url)
    .bind(label)
    .fetch_one(&mut **tx)
    .await?;
    Ok(row)
}

pub async fn get_endpoint(
    pool: &PgPool,
    system_id: Uuid,
    id: Uuid,
) -> Result<WebhookEndpoint, AppError> {
    sqlx::query_as::<_, WebhookEndpoint>(
        r#"
        SELECT id, system_id, url, label, enabled, created_at, updated_at
        FROM webhook_endpoints
        WHERE id = $1 AND system_id = $2
        "#,
    )
    .bind(id)
    .bind(system_id)
    .fetch_optional(pool)
    .await?
    .ok_or(AppError::NotFound)
}

pub async fn update_endpoint(
    pool: &PgPool,
    system_id: Uuid,
    id: Uuid,
    url: Option<&str>,
    label: Option<&str>,
    enabled: Option<bool>,
) -> Result<WebhookEndpoint, AppError> {
    let existing = get_endpoint(pool, system_id, id).await?;
    let url = url.unwrap_or(&existing.url);
    let label = label.or(existing.label.as_deref());
    let enabled = enabled.unwrap_or(existing.enabled);

    sqlx::query_as::<_, WebhookEndpoint>(
        r#"
        UPDATE webhook_endpoints
        SET url = $3, label = $4, enabled = $5, updated_at = NOW()
        WHERE id = $1 AND system_id = $2
        RETURNING id, system_id, url, label, enabled, created_at, updated_at
        "#,
    )
    .bind(id)
    .bind(system_id)
    .bind(url)
    .bind(label)
    .bind(enabled)
    .fetch_one(pool)
    .await
    .map_err(AppError::Database)
}

pub async fn delete_endpoint(pool: &PgPool, system_id: Uuid, id: Uuid) -> Result<(), AppError> {
    let result = sqlx::query(
        r#"
        DELETE FROM webhook_endpoints WHERE id = $1 AND system_id = $2
        "#,
    )
    .bind(id)
    .bind(system_id)
    .execute(pool)
    .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound);
    }
    Ok(())
}

/// Distinct enabled webhook URLs across all tenants (for event fan-out).
pub async fn list_enabled_webhook_urls(pool: &PgPool) -> Result<Vec<(Uuid, String)>, AppError> {
    let rows: Vec<(Uuid, String)> = sqlx::query_as(
        r#"
        SELECT system_id, url
        FROM webhook_endpoints
        WHERE enabled = TRUE
        ORDER BY created_at ASC
        "#,
    )
    .fetch_all(pool)
    .await?;
    Ok(rows)
}
