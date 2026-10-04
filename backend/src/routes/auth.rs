use axum::{routing::post, Router};
use sqlx::PgPool;

use crate::handlers::auth::{register, login};

pub fn routes(pool: PgPool) -> Router {
    Router::new()
        .route("/register", post(register))
        .route("/login", post(login))
        .with_state(pool)
}

