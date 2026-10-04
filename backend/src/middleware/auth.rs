use axum::{
    extract::Request,
    http::{header, StatusCode},
    middleware::Next,
    response::IntoResponse,
};



use jsonwebtoken::{decode, DecodingKey, Validation};
use serde::{Deserialize, Serialize};
use std::env;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: String, // Biasanya berisi user_id
    pub exp: usize,  // Expiration timestamp
}

pub async fn auth(mut req: Request, next: Next) -> Result<impl IntoResponse, (StatusCode, String)> {
    let auth_header = req.headers().get(header::AUTHORIZATION).ok_or((StatusCode::UNAUTHORIZED, "Missing Authorization header".to_string()))?;

    let auth_header_str = auth_header.to_str().map_err(|_| (StatusCode::UNAUTHORIZED, "Invalid Authorization header".to_string()))?;
    let token = auth_header_str.strip_prefix("Bearer ").ok_or((StatusCode::UNAUTHORIZED, "Invalid Authorization header format".to_string()))?;

    let secret = env::var("JWT_SECRET").expect("JWT_SECRET must be set");
    let decoding_key = DecodingKey::from_secret(secret.as_bytes());

let token_data =decode::<Claims>(token, &decoding_key, &Validation::default())
        .map_err(|_| (StatusCode::UNAUTHORIZED, "Invalid token".to_string()))?;

    req.extensions_mut().insert(token_data.claims);


    // Proceed to the next middleware or handler
    Ok(next.run(req).await)
}