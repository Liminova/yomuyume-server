use super::super::{ApiResponse, ErrorResponseBody};
use crate::{
    models::{categories::Model as Category, prelude::Categories},
    utils::{build_err_resp, build_resp},
    AppState,
};
use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use sea_orm::*;
use serde::Serialize;
use std::sync::Arc;
use utoipa::ToSchema;

#[derive(Serialize, ToSchema)]
pub struct CategoriesResponseBody {
    /// A list of all categories fetched.
    pub data: Vec<Category>,
}

/// Get all categories to be displayed on the library page.
#[utoipa::path(get, path = "/api/index/categories", responses(
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
