use axum::{
    routing::get,
    Router,
};
use sqlx::postgres::PgPoolOptions;
use std::env;

mod handlers;
mod models;
mod routes;

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

    let app = Router::new()
        .route("/api/health", get(health))
        .nest("/api/auth", routes::auth::routes(pool));


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