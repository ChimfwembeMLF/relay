use axum::{
    extract::{Request, State},
    http::StatusCode,
    middleware::{self, Next},
    response::Response,
    routing::{get, post},
    Router,
};
use uuid::Uuid;

use crate::api::invoices;
use crate::api::pay_page;
use crate::api::payments;
use crate::api::reports;
use crate::api::systems;
use crate::api::wallets;
use crate::auth::{hash_api_key, verify_api_key};
use crate::db::queries;
use crate::error::AppError;
use crate::models::System;
use crate::AppState;

#[derive(Clone)]
pub struct AuthenticatedSystem {
    pub system: System,
}

pub fn create_router(state: AppState) -> Router {
    let public = Router::new()
        .route("/systems", post(systems::create_system))
        .route("/systems/:id", get(systems::get_system))
        .route("/pay/:reference", get(pay_page::show_pay_page).post(pay_page::submit_pay_page));

    let protected = Router::new()
        .route("/payments", post(payments::process_payment))
        .route("/payments/:id", get(payments::get_payment))
        .route("/wallets/:system_id", get(wallets::list_wallets))
        .route("/transactions/:system_id", get(payments::list_transactions))
        .route("/invoices", post(invoices::create_invoice_handler).get(invoices::list_invoices_handler))
        .route("/invoices/:reference", get(invoices::get_invoice_by_reference_handler))
        .route("/invoices/:id/collect", post(invoices::collect_invoice))
        .route("/invoices/:id/cancel", post(invoices::cancel_invoice_handler))
        .route("/reports/transactions", get(reports::transactions_report))
        .route("/reports/wallets", get(reports::wallets_report))
        .route("/reports/invoices", get(reports::invoices_report))
        .route_layer(middleware::from_fn_with_state(state.clone(), auth_middleware));

    Router::new()
        .merge(public)
        .merge(protected)
        .with_state(state)
}

async fn auth_middleware(
    State(state): State<AppState>,
    mut req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let api_key = req
        .headers()
        .get("X-API-Key")
        .and_then(|v| v.to_str().ok())
        .ok_or(StatusCode::UNAUTHORIZED)?;

    let hash = hash_api_key(api_key);
    let system = queries::get_system_by_api_key_hash(state.db.pool(), &hash)
        .await
        .map_err(|_| StatusCode::UNAUTHORIZED)?;

    req.extensions_mut().insert(AuthenticatedSystem { system });
    Ok(next.run(req).await)
}

pub fn require_system_access(auth: &AuthenticatedSystem, system_id: Uuid) -> Result<(), AppError> {
    if auth.system.id != system_id {
        return Err(AppError::Forbidden);
    }
    Ok(())
}

#[allow(dead_code)]
pub fn verify_key_for_system(api_key: &str, system: &System) -> bool {
    verify_api_key(api_key, &system.api_key_hash)
}
