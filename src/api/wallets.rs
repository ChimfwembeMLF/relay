use axum::{
    extract::{Path, State},
    Extension, Json,
};
use uuid::Uuid;

use crate::api::routes::{require_system_access, AuthenticatedSystem};
use crate::db::queries;
use crate::error::AppError;
use crate::models::Wallet;
use crate::AppState;

pub async fn list_wallets(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthenticatedSystem>,
    Path(system_id): Path<Uuid>,
) -> Result<Json<Vec<Wallet>>, AppError> {
    require_system_access(&auth, system_id)?;
    let wallets = queries::list_wallets_by_system(state.db.pool(), system_id).await?;
    Ok(Json(wallets))
}
