use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::FromRow;
use uuid::Uuid;

use crate::db::admins::{self, AdminLoginRequest, AdminLoginResponse, AdminMeResponse};
use crate::db::queries;
use crate::db::webhook_endpoints::{self, WebhookEndpoint};
use crate::error::AppError;
use crate::models::{SystemPublic, Wallet};
use crate::AppState;

use axum::{
    extract::{Path, Request, State},
    http::StatusCode,
    middleware::Next,
    response::Response,
    Extension, Json,
};

#[derive(Clone)]
pub struct AdminAuth {
    pub user_id: Uuid,
    pub username: String,
}

#[derive(Serialize, FromRow)]
pub struct AdminSystemSummary {
    pub id: Uuid,
    pub name: String,
    pub prefix: String,
    pub enabled_countries: Vec<String>,
    pub webhook_url: Option<String>,
    pub webhook_endpoints: i64,
    pub created_at: DateTime<Utc>,
}

#[derive(Serialize)]
pub struct AdminSystemDetail {
    pub system: SystemPublic,
    pub wallets: Vec<Wallet>,
    pub webhook_endpoints: Vec<WebhookEndpoint>,
}

fn extract_admin_token(req: &Request) -> Option<String> {
    if let Some(auth) = req
        .headers()
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
    {
        if let Some(token) = auth.strip_prefix("Bearer ") {
            let token = token.trim();
            if !token.is_empty() {
                return Some(token.to_string());
            }
        }
    }

    req.headers()
        .get("X-Admin-Token")
        .and_then(|v| v.to_str().ok())
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
}

pub async fn admin_middleware(
    State(state): State<AppState>,
    mut req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let Some(token) = extract_admin_token(&req) else {
        return Err(StatusCode::UNAUTHORIZED);
    };

    let user = admins::authenticate_session(state.db.pool(), &token)
        .await
        .map_err(|_| StatusCode::UNAUTHORIZED)?;

    req.extensions_mut().insert(AdminAuth {
        user_id: user.id,
        username: user.username,
    });
    Ok(next.run(req).await)
}

pub async fn login(
    State(state): State<AppState>,
    Json(req): Json<AdminLoginRequest>,
) -> Result<Json<AdminLoginResponse>, AppError> {
    if req.username.trim().is_empty() || req.password.is_empty() {
        return Err(AppError::Validation("username and password are required".into()));
    }
    let result = admins::login(state.db.pool(), &req.username, &req.password).await?;
    tracing::info!(username = %result.username, "admin logged in");
    Ok(Json(result))
}

pub async fn logout(
    State(state): State<AppState>,
    req: Request,
) -> Result<Json<serde_json::Value>, AppError> {
    if let Some(token) = extract_admin_token(&req) {
        admins::delete_session(state.db.pool(), &token).await?;
    }
    Ok(Json(serde_json::json!({ "ok": true })))
}

pub async fn me(
    Extension(admin): Extension<AdminAuth>,
) -> Result<Json<AdminMeResponse>, AppError> {
    Ok(Json(AdminMeResponse {
        id: admin.user_id,
        username: admin.username,
    }))
}

pub async fn list_systems(
    State(state): State<AppState>,
    Extension(_admin): Extension<AdminAuth>,
) -> Result<Json<Vec<AdminSystemSummary>>, AppError> {
    let rows = sqlx::query_as::<_, AdminSystemSummary>(
        r#"
        SELECT
            s.id,
            s.name,
            s.prefix,
            s.enabled_countries,
            s.webhook_url,
            (SELECT COUNT(*)::bigint FROM webhook_endpoints we WHERE we.system_id = s.id) AS webhook_endpoints,
            s.created_at
        FROM systems s
        ORDER BY s.created_at DESC
        "#,
    )
    .fetch_all(state.db.pool())
    .await?;
    Ok(Json(rows))
}

pub async fn get_system(
    State(state): State<AppState>,
    Extension(_admin): Extension<AdminAuth>,
    Path(id): Path<Uuid>,
) -> Result<Json<AdminSystemDetail>, AppError> {
    let system = queries::get_system_by_id(state.db.pool(), id).await?;
    let wallets = queries::list_wallets_by_system(state.db.pool(), id).await?;
    let webhook_endpoints = webhook_endpoints::list_endpoints(state.db.pool(), id).await?;
    Ok(Json(AdminSystemDetail {
        system: system.into(),
        wallets,
        webhook_endpoints,
    }))
}
