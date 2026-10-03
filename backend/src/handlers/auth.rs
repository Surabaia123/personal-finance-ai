

use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    Json,
};

use argon2::{
    password_hash::{rand_core::OsRng, PasswordHasher, SaltString},
    Argon2,
};

use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use ulid::Ulid;

#[derive(Debug, Deserialize)]
pub struct RegisterForm {
    pub name: String,
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct RegisterResponse {
    pub user_id: String,
    pub message: String,
}

pub async fn register(
    State(pool): State<PgPool>,
    Json(payload): Json<RegisterForm>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    if payload.name.is_empty() {
        return Err((StatusCode::BAD_REQUEST, "Name is required".to_string()));
    }

    if payload.email.is_empty() {
        return Err((StatusCode::BAD_REQUEST, "Email is required".to_string()));
    }

    if payload.password.is_empty() {
        return Err((StatusCode::BAD_REQUEST, "Password is required".to_string()));
    }

    if payload.password.len() < 8 {
        return Err((StatusCode::BAD_REQUEST, "Password must be at least 8 characters long".to_string()));
    }

    let email_exists = sqlx::query_scalar!(
        "SELECT 1 FROM users WHERE email = $1",
        payload.email
    ) .fetch_optional(&pool)
    .await
    .map_err(|_| (
        StatusCode::INTERNAL_SERVER_ERROR,
        "Database error".to_string(),
    ))?;

if email_exists.is_some() {
        return Err((StatusCode::BAD_REQUEST, "Email already exists".to_string()));
    
}
   
    let salt= SaltString::generate(&mut OsRng);

    let hashed_password = Argon2::default()
        .hash_password(payload.password.as_bytes(), &salt)
        .map_err(|_| (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to hash password".to_string(),
        ))?
        .to_string();

    let user_id = Ulid::new().to_string();

    let result = sqlx::query!(
        "INSERT INTO users (id, name, email, password_hash) VALUES ($1, $2, $3, $4) RETURNING id, name, email, created_at, updated_at",
        user_id,
        payload.name,
        payload.email,
        hashed_password
    )
    .fetch_one(&pool)
    .await
    .map_err(|_| (
        StatusCode::INTERNAL_SERVER_ERROR,
        "Failed to create user".to_string(),
    ))?;

    let response = RegisterResponse {
        user_id: result.id,
        message: "User registered successfully".to_string(),
    };

    Ok((StatusCode::CREATED, Json(response)))
}
