use axum::{routing::post, Router};
use sqlx::PgPool;

use crate::handlers::auth::register;

pub fn routes(pool: PgPool) -> Router {
    Router::new()
        .route("/register", post(register))
        .with_state(pool)
}