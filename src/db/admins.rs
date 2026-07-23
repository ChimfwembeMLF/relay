use chrono::{DateTime, Duration, Utc};
use rand::Rng;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};
use uuid::Uuid;

use crate::auth::{hash_api_key, hash_password, verify_password};
use crate::error::AppError;

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct AdminUser {
    pub id: Uuid,
    pub username: String,
    pub password_hash: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct AdminLoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct AdminLoginResponse {
    pub token: String,
    pub username: String,
    pub expires_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct AdminMeResponse {
    pub id: Uuid,
    pub username: String,
}

pub async fn upsert_admin_user(
    pool: &PgPool,
    username: &str,
    password: &str,
) -> Result<AdminUser, AppError> {
    let password_hash = hash_password(password)?;
    let row = sqlx::query_as::<_, AdminUser>(
        r#"
        INSERT INTO admin_users (username, password_hash)
        VALUES ($1, $2)
        ON CONFLICT (username) DO UPDATE
          SET password_hash = EXCLUDED.password_hash,
              updated_at = NOW()
        RETURNING id, username, password_hash, created_at, updated_at
        "#,
    )
    .bind(username)
    .bind(&password_hash)
    .fetch_one(pool)
    .await?;
    Ok(row)
}

pub async fn get_admin_by_username(
    pool: &PgPool,
    username: &str,
) -> Result<Option<AdminUser>, AppError> {
    let row = sqlx::query_as::<_, AdminUser>(
        r#"
        SELECT id, username, password_hash, created_at, updated_at
        FROM admin_users
        WHERE username = $1
        "#,
    )
    .bind(username)
    .fetch_optional(pool)
    .await?;
    Ok(row)
}

pub async fn create_session(
    pool: &PgPool,
    admin_user_id: Uuid,
) -> Result<(String, DateTime<Utc>), AppError> {
    let token_bytes: [u8; 32] = rand::thread_rng().gen();
    let token = format!("adm_{}", hex::encode(token_bytes));
    let token_hash = hash_api_key(&token);
    let expires_at = Utc::now() + Duration::days(7);

    sqlx::query(
        r#"
        INSERT INTO admin_sessions (admin_user_id, token_hash, expires_at)
        VALUES ($1, $2, $3)
        "#,
    )
    .bind(admin_user_id)
    .bind(&token_hash)
    .bind(expires_at)
    .execute(pool)
    .await?;

    Ok((token, expires_at))
}

pub async fn authenticate_session(
    pool: &PgPool,
    token: &str,
) -> Result<AdminUser, AppError> {
    let token_hash = hash_api_key(token);
    let row = sqlx::query_as::<_, AdminUser>(
        r#"
        SELECT u.id, u.username, u.password_hash, u.created_at, u.updated_at
        FROM admin_sessions s
        JOIN admin_users u ON u.id = s.admin_user_id
        WHERE s.token_hash = $1
          AND s.expires_at > NOW()
        "#,
    )
    .bind(&token_hash)
    .fetch_optional(pool)
    .await?
    .ok_or(AppError::Unauthorized)?;
    Ok(row)
}

pub async fn delete_session(pool: &PgPool, token: &str) -> Result<(), AppError> {
    let token_hash = hash_api_key(token);
    sqlx::query(r#"DELETE FROM admin_sessions WHERE token_hash = $1"#)
        .bind(&token_hash)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn login(
    pool: &PgPool,
    username: &str,
    password: &str,
) -> Result<AdminLoginResponse, AppError> {
    let user = get_admin_by_username(pool, username.trim())
        .await?
        .ok_or(AppError::Unauthorized)?;

    if !verify_password(password, &user.password_hash)? {
        return Err(AppError::Unauthorized);
    }

    let (token, expires_at) = create_session(pool, user.id).await?;
    Ok(AdminLoginResponse {
        token,
        username: user.username,
        expires_at,
    })
}
