
use axum::{
    Json, extract::{Extension, Query, State,Path}, http::StatusCode,
};

use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use ulid::Ulid;





#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")] // Menerima JSON "income" atau "expense"
pub enum CategoryType {
    Income,
    Expense,
}

// Struct untuk menampung query parameter opsional (?category_type=income)
#[derive(Debug, Deserialize)]
pub struct CategoryFilter {
    pub category_type: Option<CategoryType>,
}

#[derive(Debug, Deserialize)]
pub struct CategoryForm {
    pub name: String,
    pub category_type: CategoryType
}

#[derive(Debug, Deserialize)]
pub struct UpdateCategoryForm {
    pub name: Option<String>,
    pub category_type: Option<CategoryType>,
}


#[derive(Debug, Serialize)]
pub struct CategoryResponse {
    pub id: String,
    pub name: String,
    pub category_type: String,
}


#[derive(Debug, Serialize)]
pub struct CategoryResponseUser {
    pub id: String,
    pub name: String,
    pub category_type: String,
    pub user_id: String,
    pub user_name: String,
}

#[derive(Debug, Serialize)]
pub struct ApiResponse<T> {
    pub message: String,
    pub data: T
}


pub async fn add_category(
    Extension(user_id): Extension<String>,
    State(pool): State<PgPool>,
    Json(payload): Json<CategoryForm>,
) -> Result<(StatusCode, Json<ApiResponse<CategoryResponse>>), (StatusCode, String)> {
    if payload.name.is_empty() {
        return Err((StatusCode::BAD_REQUEST, "Name is required".to_string()));
    }

    
    let category_type_str = match payload.category_type {
        CategoryType::Income => "income",
        CategoryType::Expense => "expense",
    };


    let category_exists = sqlx::query_scalar!(
        "SELECT 1 FROM categories WHERE name = $1 AND type = $2 AND user_id = $3",
        payload.name,
        category_type_str,
        user_id
    )
    .fetch_optional(&pool)
    .await
    .map_err(|_| (
        StatusCode::INTERNAL_SERVER_ERROR,
        "Database error".to_string(),
    )) ?;

    if category_exists.is_some() {
        return Err((StatusCode::BAD_REQUEST, "Category already exists".to_string()));
    }


    let category_id = Ulid::new().to_string();

    sqlx::query!(
            "INSERT INTO categories (id, name, type, user_id) VALUES ($1, $2, $3, $4)",
            category_id,
            payload.name,
            category_type_str,
            user_id
        )
        .execute(&pool)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Database error: {}", e),
            )
        })?;


    Ok((StatusCode::CREATED, Json(ApiResponse {
        message: "Category added successfully".to_string(),
        data: CategoryResponse {
            id: category_id,
            name: payload.name,
            category_type: category_type_str.to_string(),
        },
    })))
}   
        


pub async fn get_categories(
    Extension(user_id): Extension<String>,
    State(pool): State<PgPool>,
    Query(filter): Query<CategoryFilter>
) -> Result<(StatusCode, Json<ApiResponse<Vec<CategoryResponseUser>>>), (StatusCode, String)> {

    let category_type_filter = filter.category_type.as_ref().map(|ct| match ct {
        CategoryType::Income => "income",
        CategoryType::Expense => "expense",
    });
    
    let categories =sqlx::query!(
        "SELECT c.*, u.name as user_name FROM categories c
        LEFT JOIN users u ON c.user_id = u.id WHERE c.user_id = $1 AND ($2::text IS NULL OR c.type = $2)",
        user_id, category_type_filter
    )
    .fetch_all(&pool)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Database error: {}", e),
        )
    })?;

    let category_responses: Vec<CategoryResponseUser> = categories
        .into_iter()
        .map(|row| CategoryResponseUser {
            id: row.id,
            name: row.name,
            category_type: row.r#type,
            user_id: row.user_id,
            user_name: row.user_name,
        })
        .collect();

    Ok((StatusCode::OK, Json(ApiResponse {
        message: "Categories fetched successfully".to_string(),
        data: category_responses,
    })))
}


pub async fn update_category(
    Path(id): Path<String>,
    Extension(user_id): Extension<String>,
    State(pool): State<PgPool>,
    Json(payload): Json<UpdateCategoryForm>,
) -> Result<(StatusCode, Json<ApiResponse<CategoryResponse>>), (StatusCode, String)> {
    

    let category_exists = sqlx::query!(
        "SELECT * FROM categories WHERE id = $1 AND user_id = $2",
        id,
        user_id
    )
    .fetch_optional(&pool)
    .await
    .map_err(|_| (
        StatusCode::INTERNAL_SERVER_ERROR,
        "Database error".to_string(),
    )) ?;

    if category_exists.is_none() {
        return Err((StatusCode::NOT_FOUND, "Category not found".to_string()));
    }

    let mut new_name =category_exists.as_ref().unwrap().name.clone();

    let mut new_category_type_str = category_exists.as_ref().unwrap().r#type.clone();

    if let Some(name) = payload.name {
        new_name = name;
    }

    if let Some(category_type) = payload.category_type {
        new_category_type_str = match category_type {
            CategoryType::Income => "income".to_string(),
            CategoryType::Expense => "expense".to_string(),
        };
    }

    sqlx::query!(
        "UPDATE categories SET name = $1, type = $2 WHERE id = $3 AND user_id = $4",
        new_name,
        new_category_type_str,
        id,
        user_id
    )
    .execute(&pool)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Database error: {}", e),
        )
    })?;

    Ok((StatusCode::OK, Json(ApiResponse {
        message: "Category updated successfully".to_string(),
        data: CategoryResponse {
            id,
            name: new_name,
            category_type: new_category_type_str,
        },
    })))
}


pub async fn delete_category(
    Path(id): Path<String>,
    Extension(user_id): Extension<String>,
    State(pool): State<PgPool>,
) -> Result<(StatusCode, Json<ApiResponse<()>>), (StatusCode, String)> {
    
    let category_exists = sqlx::query!(
        "SELECT * FROM categories WHERE id = $1 AND user_id = $2",
        id,
        user_id
    )
    .fetch_optional(&pool)
    .await
    .map_err(|_| (
        StatusCode::INTERNAL_SERVER_ERROR,
        "Database error".to_string(),
    )) ?;

    if category_exists.is_none() {
        return Err((StatusCode::NOT_FOUND, "Category not found".to_string()));
    }

    sqlx::query!(
        "DELETE FROM categories WHERE id = $1 ",
        id
    )
    .execute(&pool)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Database error: {}", e),
        )
    })?;

    Ok((StatusCode::OK, Json(ApiResponse {
        message: "Category deleted successfully".to_string(),
        data: (),
    })))
}



