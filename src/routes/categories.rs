use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::Serialize;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::{models::category::Category, AppState};

use super::{ApiResponse, ErrorResponseBody};

#[derive(Serialize, ToSchema)]
pub struct CategoriesResponseBody {
    /// The list of all categories
    pub data: Vec<Category>,
}

#[derive(Serialize, ToSchema)]
pub struct CategoryResponseBody {
    #[serde(flatten)]
    pub data: Category,
}

#[utoipa::path(get, path = "/api/categories", responses(
    (status = 200, description = "Fetch all categories successful.", body = CategoriesResponse),
    (status = 500, description = "Internal server error.", body = ErrorResponse)
))]
pub async fn get_categories(
    State(data): State<Arc<AppState>>,
) -> Result<impl IntoResponse, (StatusCode, Json<ApiResponse<ErrorResponseBody>>)> {
    let categories = sqlx::query_as!(
        Category,
        r#"SELECT id as "id: uuid::Uuid", name, description FROM categories"#
    )
    .fetch_all(&data.sqlite)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse {
                description: String::from("An internal error has occurred."),
                body: Some(ErrorResponseBody {
                    message: format!("Database error: {}", e),
                }),
            }),
        )
    })?;

    Ok((
        StatusCode::OK,
        Json(ApiResponse {
            description: String::from("Fetching all categories successful."),
            body: Some(CategoriesResponseBody { data: categories }),
        }),
    ))
}

#[utoipa::path(get, path = "/api/category/{category_id}", responses(
    (status = 200, description = "Fetch category successful.", body = CategoryResponse),
    (status = 204, description = "Fetch category successful, but one was not found.", body = CategoryResponse),
    (status = 500, description = "Internal server error.", body = ErrorResponse)
))]
pub async fn get_category(
    State(data): State<Arc<AppState>>,
    Path(category_id): Path<Uuid>,
) -> Result<impl IntoResponse, (StatusCode, Json<ApiResponse<ErrorResponseBody>>)> {
    let category = sqlx::query_as!(
        Category,
        r#"SELECT id as "id: uuid::Uuid", name, description FROM categories WHERE id = $1"#,
        category_id
    )
    .fetch_optional(&data.sqlite)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse {
                description: String::from("An internal error has occurred."),
                body: Some(ErrorResponseBody {
                    message: format!("Database error: {}", e),
                }),
            }),
        )
    })?;

    match category {
        Some(c) => Ok((
            StatusCode::OK,
            Json(ApiResponse {
                description: format!("Fetch category with id {} successful.", category_id),
                body: Some(CategoryResponseBody { data: c }),
            }),
        )),
        None => Ok((
            StatusCode::NO_CONTENT,
            Json(ApiResponse {
                description: format!(
                    "The server could not find any categories matching the id {}.",
                    category_id
                ),
                body: None,
            }),
        )),
    }
}
