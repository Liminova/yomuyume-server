use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use sea_orm::*;
use serde::Serialize;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::{
    models::{categories::Model as Category, prelude::Categories},
    utils::build_resp::{build_err_resp, build_resp},
    AppState,
};

use super::{ApiResponse, ErrorResponseBody};

#[derive(Serialize, ToSchema)]
pub struct CategoriesResponseBody {
    /// A list of all categories fetched.
    pub data: Vec<Category>,
}

#[derive(Serialize, ToSchema)]
pub struct CategoryResponseBody {
    /// The requested category.
    pub data: Category,
}

#[utoipa::path(get, path = "/api/categories", responses(
    (status = 200, description = "Fetch all categories successful.", body = CategoriesResponse),
    (status = 500, description = "Internal server error.", body = ErrorResponse)
))]
pub async fn get_categories(
    State(data): State<Arc<AppState>>,
) -> Result<impl IntoResponse, (StatusCode, Json<ApiResponse<ErrorResponseBody>>)> {
    let categories: Vec<Category> = Categories::find().all(&data.db).await.map_err(|e| {
        build_err_resp(
            StatusCode::INTERNAL_SERVER_ERROR,
            String::from("An internal server error has occurred."),
            format!("Database error: {}", e),
        )
    })?;

    Ok(build_resp(
        StatusCode::OK,
        String::from("Fetching all categories successful."),
        Some(CategoriesResponseBody { data: categories }),
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
    let category: Option<Category> = Categories::find_by_id(category_id)
        .one(&data.db)
        .await
        .map_err(|e| {
            build_err_resp(
                StatusCode::INTERNAL_SERVER_ERROR,
                String::from("An internal server error has occurred."),
                format!("Database error: {}", e),
            )
        })?;

    let resp = match category {
        Some(c) => build_resp(
            StatusCode::OK,
            format!("Fetch category with id {} successful.", category_id),
            Some(CategoryResponseBody { data: c }),
        ),
        None => build_resp(
            StatusCode::NO_CONTENT,
            format!(
                "The server could not find any categories matching the id {}.",
                category_id
            ),
            None::<CategoryResponseBody>,
        ),
    };

    Ok(resp)
}
