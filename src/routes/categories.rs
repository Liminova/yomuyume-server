use std::sync::Arc;

use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use serde::Serialize;
use utoipa::ToSchema;

use crate::{models::category::Category, AppState};

use super::{ApiResponse, ErrorResponseBody};

#[derive(Serialize, ToSchema)]
pub struct CategoriesResponseBody {
    /// The list of all categories
    pub data: Vec<Category>,
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
