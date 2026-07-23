use chrono::{DateTime, Duration, Utc};
use rand::Rng;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool, Postgres, Transaction};
use uuid::Uuid;

use crate::auth::{hash_api_key, hash_password, verify_password};
use crate::error::AppError;
use crate::models::System;

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct SystemUser {
    pub id: Uuid,
    pub system_id: Uuid,
    pub username: String,
    pub password_hash: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct SystemLoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct SystemLoginResponse {
    pub token: String,
    pub username: String,
    pub expires_at: DateTime<Utc>,
    pub system_id: Uuid,
    pub name: String,
    pub prefix: String,
}

pub async fn create_system_user_in_tx(
    tx: &mut Transaction<'_, Postgres>,
    system_id: Uuid,
    username: &str,
    password: &str,
) -> Result<SystemUser, AppError> {
    let password_hash = hash_password(password)?;
    let row = sqlx::query_as::<_, SystemUser>(
        r#"
        INSERT INTO system_users (system_id, username, password_hash)
        VALUES ($1, $2, $3)
        RETURNING id, system_id, username, password_hash, created_at, updated_at
        "#,
    )
    .bind(system_id)
    .bind(username)
    .bind(&password_hash)
    .fetch_one(&mut **tx)
    .await
    .map_err(|e| {
        if let sqlx::Error::Database(db) = &e {
            if db.constraint().is_some_and(|c| c.contains("username")) {
                return AppError::Conflict("username already taken".into());
            }
        }
        AppError::Database(e)
    })?;
    Ok(row)
}

pub async fn get_user_by_username(
    pool: &PgPool,
    username: &str,
) -> Result<Option<(SystemUser, System)>, AppError> {
    #[derive(FromRow)]
    struct Row {
        // user
        id: Uuid,
        system_id: Uuid,
        username: String,
        password_hash: String,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
        // system
        name: String,
        prefix: String,
        enabled_countries: Vec<String>,
        webhook_url: Option<String>,
        api_key_hash: String,
        system_created_at: DateTime<Utc>,
        system_updated_at: DateTime<Utc>,
    }

    let row = sqlx::query_as::<_, Row>(
        r#"
        SELECT
            u.id,
            u.system_id,
            u.username,
            u.password_hash,
            u.created_at,
            u.updated_at,
            s.name,
            s.prefix,
            s.enabled_countries,
            s.webhook_url,
            s.api_key_hash,
            s.created_at AS system_created_at,
            s.updated_at AS system_updated_at
        FROM system_users u
        JOIN systems s ON s.id = u.system_id
        WHERE u.username = $1
        "#,
    )
    .bind(username)
    .fetch_optional(pool)
    .await?;

    Ok(row.map(|r| {
        (
            SystemUser {
                id: r.id,
                system_id: r.system_id,
                username: r.username,
                password_hash: r.password_hash,
                created_at: r.created_at,
                updated_at: r.updated_at,
            },
            System {
                id: r.system_id,
                name: r.name,
                prefix: r.prefix,
                enabled_countries: r.enabled_countries,
                webhook_url: r.webhook_url,
                api_key_hash: r.api_key_hash,
                created_at: r.system_created_at,
                updated_at: r.system_updated_at,
            },
        )
    }))
}

pub async fn create_session(
    pool: &PgPool,
    system_user_id: Uuid,
) -> Result<(String, DateTime<Utc>), AppError> {
    let token_bytes: [u8; 32] = rand::thread_rng().gen();
    let token = format!("sess_{}", hex::encode(token_bytes));
    let token_hash = hash_api_key(&token);
    let expires_at = Utc::now() + Duration::days(14);

    sqlx::query(
        r#"
        INSERT INTO system_sessions (system_user_id, token_hash, expires_at)
        VALUES ($1, $2, $3)
        "#,
    )
    .bind(system_user_id)
    .bind(&token_hash)
    .bind(expires_at)
    .execute(pool)
    .await?;

    Ok((token, expires_at))
}

pub async fn authenticate_session(pool: &PgPool, token: &str) -> Result<System, AppError> {
    let token_hash = hash_api_key(token);
    let system = sqlx::query_as::<_, System>(
        r#"
        SELECT
            s.id,
            s.name,
            s.prefix,
            s.enabled_countries,
            s.webhook_url,
            s.api_key_hash,
            s.created_at,
            s.updated_at
        FROM system_sessions sess
        JOIN system_users u ON u.id = sess.system_user_id
        JOIN systems s ON s.id = u.system_id
        WHERE sess.token_hash = $1
          AND sess.expires_at > NOW()
        "#,
    )
    .bind(&token_hash)
    .fetch_optional(pool)
    .await?
    .ok_or(AppError::Unauthorized)?;
    Ok(system)
}

pub async fn delete_session(pool: &PgPool, token: &str) -> Result<(), AppError> {
    let token_hash = hash_api_key(token);
    sqlx::query(r#"DELETE FROM system_sessions WHERE token_hash = $1"#)
        .bind(&token_hash)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn login(
    pool: &PgPool,
    username: &str,
    password: &str,
) -> Result<SystemLoginResponse, AppError> {
    let Some((user, system)) = get_user_by_username(pool, username.trim()).await? else {
        return Err(AppError::Unauthorized);
    };

    if !verify_password(password, &user.password_hash)? {
        return Err(AppError::Unauthorized);
    }

    let (token, expires_at) = create_session(pool, user.id).await?;
    Ok(SystemLoginResponse {
        token,
        username: user.username,
        expires_at,
        system_id: system.id,
        name: system.name,
        prefix: system.prefix,
    })
}
