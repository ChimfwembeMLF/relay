use axum::{
    extract::{Request, State},
    http::{header, Method, StatusCode},
    middleware::{self, Next},
    response::Response,
    routing::{get, patch, post},
    Router,
};
use tower_http::services::{ServeDir, ServeFile};
use uuid::Uuid;

use crate::api::admin;
use crate::api::auth as merchant_auth;
use crate::api::batches;
use crate::api::docs;
use crate::api::invoices;
use crate::api::pay_page;
use crate::api::payments;
use crate::api::refunds;
use crate::api::reports;
use crate::api::systems;
use crate::api::wallets;
use crate::api::webhooks;
use crate::auth::{hash_api_key, verify_api_key};
use crate::db::{queries, system_users};
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
        .route("/auth/login", post(merchant_auth::login))
        .route("/auth/logout", post(merchant_auth::logout))
        .route("/admin/login", post(admin::login))
        .route(
            "/api/pay/:reference",
            get(pay_page::get_pay_api).post(pay_page::submit_pay_api),
        )
        .nest_service("/assets", ServeDir::new("frontend/dist/assets"))
        .route_service("/favicon.svg", ServeFile::new("frontend/dist/favicon.svg"))
        .route_service("/logo-blue.png", ServeFile::new("frontend/dist/logo-blue.png"))
        .route_service("/logo.png", ServeFile::new("frontend/dist/logo.png"))
        .route_service(
            "/register-hero.jpg",
            ServeFile::new("frontend/dist/register-hero.jpg"),
        );

    let public = docs::mount(public);

    let protected = Router::new()
        .route("/payments", post(payments::process_payment))
        .route("/payments/:id", get(payments::get_payment))
        .route("/batches", post(batches::create_batch))
        .route("/batches/:id", get(batches::get_batch))
        .route("/wallets/:system_id", get(wallets::list_wallets))
        .route("/transactions/:system_id", get(payments::list_transactions))
        .route(
            "/invoices",
            post(invoices::create_invoice_handler).get(invoices::list_invoices_handler),
        )
        .route(
            "/invoices/reference/:reference",
            get(invoices::get_invoice_by_reference_handler),
        )
        .route("/invoices/:id/collect", post(invoices::collect_invoice))
        .route("/invoices/:id/cancel", post(invoices::cancel_invoice_handler))
        .route("/invoices/:id/refund", post(refunds::create_refund))
        .route("/reports/transactions", get(reports::transactions_report))
        .route("/reports/wallets", get(reports::wallets_report))
        .route("/reports/invoices", get(reports::invoices_report))
        .route(
            "/webhook-endpoints",
            get(webhooks::list_webhooks).post(webhooks::create_webhook),
        )
        .route(
            "/webhook-endpoints/:id",
            patch(webhooks::update_webhook).delete(webhooks::delete_webhook),
        )
        .route_layer(middleware::from_fn_with_state(state.clone(), auth_middleware));

    let admin_routes = Router::new()
        .route("/admin/me", get(admin::me))
        .route("/admin/logout", post(admin::logout))
        .route("/admin/systems", get(admin::list_systems))
        .route("/admin/systems/:id", get(admin::get_system))
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            admin::admin_middleware,
        ));

    Router::new()
        .merge(public)
        .merge(protected)
        .merge(admin_routes)
        .fallback(get(pay_page::serve_spa))
        .with_state(state)
}

/// Browser navigations (refresh / open URL) send `Accept: text/html…`.
/// API clients send `Accept: application/json`. SPA paths like `/payments` and
/// `/invoices` share URLs with the API — without this, auth middleware returns
/// 401 before the SPA fallback can run.
pub(crate) fn is_browser_document_request(req: &Request) -> bool {
    if req.method() != Method::GET {
        return false;
    }
    let Some(accept) = req.headers().get(header::ACCEPT).and_then(|v| v.to_str().ok()) else {
        return false;
    };
    let html = accept.find("text/html");
    let json = accept.find("application/json");
    match (html, json) {
        (Some(h), Some(j)) => h < j,
        (Some(_), None) => true,
        _ => false,
    }
}

async fn auth_middleware(
    State(state): State<AppState>,
    mut req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    if is_browser_document_request(&req) {
        return Ok(pay_page::serve_spa().await);
    }

    let system = if let Some(api_key) = req
        .headers()
        .get("X-API-Key")
        .and_then(|v| v.to_str().ok())
        .map(str::trim)
        .filter(|s| !s.is_empty())
    {
        let hash = hash_api_key(api_key);
        queries::get_system_by_api_key_hash(state.db.pool(), &hash)
            .await
            .map_err(|_| StatusCode::UNAUTHORIZED)?
    } else if let Some(token) = session_token_from_request(&req) {
        system_users::authenticate_session(state.db.pool(), &token)
            .await
            .map_err(|_| StatusCode::UNAUTHORIZED)?
    } else {
        return Err(StatusCode::UNAUTHORIZED);
    };

    req.extensions_mut().insert(AuthenticatedSystem { system });
    Ok(next.run(req).await)
}

fn session_token_from_request(req: &Request) -> Option<String> {
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
