use super::{
    super::{ApiResponse, ErrorResponseBody},
    build_err_resp, build_resp,
};
use crate::{models::prelude::*, AppState};
use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use sea_orm::*;
use serde::Serialize;
use std::sync::Arc;
use utoipa::ToSchema;

#[derive(Serialize, ToSchema)]
pub struct CategoriesResponseBody {
    /// A list of all categories fetched.
    pub data: Vec<categories::Model>,
}

/// Get all categories to be displayed on the library page.
#[utoipa::path(get, path = "/api/index/categories", responses(
    (status = 200, description = "Fetch all categories successful.", body = CategoriesResponse),
    (status = 500, description = "Internal server error.", body = ErrorResponse)
))]
pub async fn get_categories(
    State(data): State<Arc<AppState>>,
) -> Result<impl IntoResponse, (StatusCode, Json<ApiResponse<ErrorResponseBody>>)> {
    let categories = Categories::find().all(&data.db).await.map_err(|e| {
        build_err_resp(
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("DB error getting categories: {}", e),
        )
    })?;

    Ok(build_resp(
        StatusCode::OK,
        Some(CategoriesResponseBody { data: categories }),
    ))
}
