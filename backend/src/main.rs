mod handlers;
mod middleware;
mod models;
mod routes;

use axum::{
    extract::Extension,
    middleware as axum_middleware, // Memberi alias 'axum_middleware' agar tidak bentrok dengan mod middleware lokal
    routing::get,
    Json, Router,
};
use sqlx::postgres::PgPoolOptions;
use std::env;

use crate::middleware::auth::{auth, Claims};

#[tokio::main]
async fn main() {
    dotenvy::from_filename("../.env").ok();

    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to connect to PostgreSQL");

    println!("Connected to PostgreSQL!");

    let protected_routes= Router::new()
        .route("/me", get(me))
        .nest("/categories", routes::category::routes(pool.clone()))
        .layer(axum_middleware::from_fn(auth));

    let app = Router::new()
        .route("/api/health", get(health))
        .nest("/api/auth", routes::auth::routes(pool.clone()))
        .nest("/api", protected_routes)
        ;


    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080")
        .await
        .expect("Failed to bind port 8080");

    println!("Server running on http://localhost:8080");

    axum::serve(listener, app)
        .await
        .expect("Server error");

   
}
async fn health() -> &'static str {
    "OK"
}


async fn me(Extension(claims): Extension<Claims>) -> Json<Claims> {
    Json(claims)
}