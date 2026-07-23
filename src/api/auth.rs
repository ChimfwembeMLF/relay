use axum::{
    extract::{Request, State},
    Json,
};

use crate::db::system_users::{self, SystemLoginRequest, SystemLoginResponse};
use crate::error::AppError;
use crate::AppState;

pub async fn login(
    State(state): State<AppState>,
    Json(req): Json<SystemLoginRequest>,
) -> Result<Json<SystemLoginResponse>, AppError> {
    if req.username.trim().is_empty() || req.password.is_empty() {
        return Err(AppError::Validation("username and password are required".into()));
    }
    let result = system_users::login(state.db.pool(), &req.username, &req.password).await?;
    tracing::info!(
        username = %result.username,
        system_id = %result.system_id,
        "system user logged in"
    );
    Ok(Json(result))
}

pub async fn logout(
    State(state): State<AppState>,
    req: Request,
) -> Result<Json<serde_json::Value>, AppError> {
    if let Some(token) = extract_session_token(&req) {
        system_users::delete_session(state.db.pool(), &token).await?;
    }
    Ok(Json(serde_json::json!({ "ok": true })))
}

fn extract_session_token(req: &Request) -> Option<String> {
    if let Some(auth) = req
        .headers()
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
    {
        if let Some(token) = auth.strip_prefix("Bearer ") {
            let token = token.trim();
            if token.starts_with("sess_") {
                return Some(token.to_string());
            }
        }
    }
    req.headers()
        .get("X-Session-Token")
        .and_then(|v| v.to_str().ok())
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
}
