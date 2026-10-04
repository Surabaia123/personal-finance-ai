

use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    Json,
};

use argon2::{
    Argon2, PasswordHash, PasswordVerifier, password_hash::{PasswordHasher, SaltString, rand_core::OsRng},
};

use jsonwebtoken::{encode, EncodingKey, Header};

use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use ulid::Ulid;
use std::env;
use chrono::{Duration, Utc};


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



#[derive(Debug, Deserialize)]
pub struct LoginForm {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub token: String,
    pub message: String,
    pub expiration: usize,
}

#[derive(Debug, Serialize)]
struct Claims {
    pub sub: String,
    pub exp: usize,
}


pub async  fn login(
    State(pool): State<PgPool>,
    Json(payload): Json<LoginForm>,
) -> Result<impl IntoResponse, (StatusCode, String)> {

    let email = payload.email.trim();

    if email.is_empty() {
        return Err((StatusCode::BAD_REQUEST, "Email is required".to_string()));
    }

    if payload.password.is_empty() {
        return Err((StatusCode::BAD_REQUEST, "Password is required".to_string()));
    }
    let user= sqlx::query!(
       
        "SELECT id, name, email, password_hash FROM users WHERE email = $1",
        email
    )
    .fetch_optional(&pool)
    .await
    .map_err(|_| (
        StatusCode::INTERNAL_SERVER_ERROR,
        "Database error".to_string(),
    ))?;

    if user.is_none() {
        return Err((StatusCode::UNAUTHORIZED, "Invalid email or password".to_string()));
    }

    let user = user.unwrap();

    let parsed_hash = PasswordHash::new(&user.password_hash).map_err(|_| (
        StatusCode::INTERNAL_SERVER_ERROR,
        "Failed to parse password hash".to_string(),
    ))?;

    if !Argon2::default().verify_password(
        payload.password.as_bytes(),
        &parsed_hash,
    ).is_ok() {
        return Err((StatusCode::UNAUTHORIZED, "Invalid email or password".to_string()));
    }


    let secret = env::var("JWT_SECRET").expect("JWT_SECRET must be set");
    let expiration = Utc::now() + Duration::minutes(60);    

    let claims = Claims {
        sub: user.id.to_string(),
        exp: expiration.timestamp() as usize,
    };

   

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_ref()),
    )
    .map_err(|_| (
        StatusCode::INTERNAL_SERVER_ERROR,
        "Failed to encode token".to_string(),
    ))?;


    

    let response = LoginResponse {
        token,
        message: "Login successful".to_string(),
        expiration: claims.exp,
    };

    Ok((StatusCode::OK, Json(response)))
}   
