use axum::{
    extract::{Extension, Path, State},
    Json,
};
use uuid::Uuid;

use crate::api::routes::AuthenticatedSystem;
use crate::db::webhook_endpoints::{
    self, CreateWebhookEndpointRequest, UpdateWebhookEndpointRequest, WebhookEndpoint,
};
use crate::error::AppError;
use crate::AppState;

pub async fn list_webhooks(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthenticatedSystem>,
) -> Result<Json<Vec<WebhookEndpoint>>, AppError> {
    let rows = webhook_endpoints::list_endpoints(state.db.pool(), auth.system.id).await?;
    Ok(Json(rows))
}

pub async fn create_webhook(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthenticatedSystem>,
    Json(req): Json<CreateWebhookEndpointRequest>,
) -> Result<Json<WebhookEndpoint>, AppError> {
    validate_webhook_url(&req.url, allow_insecure(&state))?;
    let row = webhook_endpoints::create_endpoint(
        state.db.pool(),
        auth.system.id,
        req.url.trim(),
        req.label.as_deref().map(str::trim).filter(|s| !s.is_empty()),
    )
    .await?;
    tracing::info!(
        system_id = %auth.system.id,
        endpoint_id = %row.id,
        "webhook endpoint created"
    );
    Ok(Json(row))
}

pub async fn update_webhook(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthenticatedSystem>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateWebhookEndpointRequest>,
) -> Result<Json<WebhookEndpoint>, AppError> {
    if let Some(url) = &req.url {
        validate_webhook_url(url, allow_insecure(&state))?;
    }
    let row = webhook_endpoints::update_endpoint(
        state.db.pool(),
        auth.system.id,
        id,
        req.url.as_deref().map(str::trim),
        req.label.as_deref().map(str::trim),
        req.enabled,
    )
    .await?;
    Ok(Json(row))
}

pub async fn delete_webhook(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthenticatedSystem>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    webhook_endpoints::delete_endpoint(state.db.pool(), auth.system.id, id).await?;
    Ok(Json(serde_json::json!({ "ok": true })))
}

fn allow_insecure(state: &AppState) -> bool {
    state.config.fallback_gateway == "mock"
        || std::env::var("ALLOW_INSECURE_WEBHOOKS").is_ok_and(|v| v == "1" || v == "true")
}

fn validate_webhook_url(url: &str, allow_insecure: bool) -> Result<(), AppError> {
    let url = url.trim();
    if url.is_empty() {
        return Err(AppError::Validation("url is required".into()));
    }
    if url.starts_with("https://") {
        return Ok(());
    }
    if allow_insecure
        && (url.starts_with("http://localhost")
            || url.starts_with("http://127.0.0.1")
            || url.starts_with("http://"))
    {
        return Ok(());
    }
    Err(AppError::Validation(
        "webhook url must use HTTPS (set ALLOW_INSECURE_WEBHOOKS=1 for local http)".into(),
    ))
}
