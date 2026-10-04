use axum::{routing::{post, put}, Router};
use sqlx::PgPool;

use crate::handlers::category::{add_category,get_categories, update_category, delete_category};
pub fn routes(pool: PgPool) -> Router {
    Router::new()
       .route("/", post(add_category).get(get_categories))
      .route("/{id}", put(update_category).delete(delete_category))
        .with_state(pool)
}

