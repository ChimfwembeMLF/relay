use axum::{
    extract::{Path, State},
    Json,
};
use uuid::Uuid;

use crate::auth::{generate_api_key, hash_api_key};
use crate::db::queries;
use crate::db::system_users;
use crate::error::AppError;
use crate::models::{CreateSystemRequest, CreateSystemResponse, SystemPublic};
use crate::seed::seed_system_wallets_in_tx;
use crate::AppState;

pub async fn create_system(
    State(state): State<AppState>,
    Json(req): Json<CreateSystemRequest>,
) -> Result<Json<CreateSystemResponse>, AppError> {
    validate_create_request(&req)?;

    if queries::prefix_exists(state.db.pool(), &req.prefix).await? {
        return Err(AppError::Conflict(format!("prefix '{}' already exists", req.prefix)));
    }

    // Public register always enables Zambia only (catalog default).
    let enabled_countries = vec![crate::catalog::DEFAULT_REGISTER_COUNTRY.to_string()];

    let api_key = generate_api_key();
    let api_key_hash = hash_api_key(&api_key);
    let username = req.username.trim().to_lowercase();

    let mut tx = state.db.pool().begin().await?;

    let system = queries::create_system_in_tx(
        &mut tx,
        &req.name,
        &req.prefix,
        &enabled_countries,
        req.webhook_url.as_deref(),
        &api_key_hash,
    )
    .await?;

    let user =
        system_users::create_system_user_in_tx(&mut tx, system.id, &username, &req.password)
            .await?;

    let wallets_seeded = seed_system_wallets_in_tx(
        &mut tx,
        system.id,
        &enabled_countries,
        &state.config.wallet_seed_defaults,
        &req.wallet_seeds,
    )
    .await?;

    if let Some(url) = req.webhook_url.as_deref() {
        crate::db::webhook_endpoints::create_endpoint_in_tx(
            &mut tx,
            system.id,
            url,
            Some("Primary"),
        )
        .await?;
    }

    tx.commit().await?;

    let (session_token, _) =
        system_users::create_session(state.db.pool(), user.id).await?;

    tracing::info!(
        system_id = %system.id,
        prefix = %system.prefix,
        username = %username,
        wallets_seeded = wallets_seeded,
        "system registered with seeded wallets"
    );

    Ok(Json(CreateSystemResponse {
        id: system.id,
        name: system.name,
        prefix: system.prefix,
        username,
        api_key,
        session_token,
        wallets_seeded,
    }))
}

pub async fn get_system(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<SystemPublic>, AppError> {
    let system = queries::get_system_by_id(state.db.pool(), id).await?;
    Ok(Json(system.into()))
}

fn validate_create_request(req: &CreateSystemRequest) -> Result<(), AppError> {
    if req.name.trim().is_empty() {
        return Err(AppError::Validation("name is required".into()));
    }
    if req.prefix.len() < 2
        || req.prefix.len() > 8
        || !req
            .prefix
            .chars()
            .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit())
    {
        return Err(AppError::Validation(
            "prefix must be 2-8 uppercase alphanumeric characters".into(),
        ));
    }
    let username = req.username.trim();
    if username.len() < 3
        || username.len() > 64
        || !username
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-' || c == '.')
    {
        return Err(AppError::Validation(
            "username must be 3–64 chars (letters, numbers, _-.)".into(),
        ));
    }
    if req.password.len() < 8 {
        return Err(AppError::Validation(
            "password must be at least 8 characters".into(),
        ));
    }
    if let Some(url) = &req.webhook_url {
        if !url.starts_with("https://") {
            return Err(AppError::Validation("webhook_url must use HTTPS".into()));
        }
    }
    Ok(())
}
