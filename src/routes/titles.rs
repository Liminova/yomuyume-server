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

use crate::{models::title::Title, AppState};

use super::{ApiResponse, ErrorResponseBody};

#[derive(Serialize, ToSchema)]
pub struct TitlesResponseBody {
    pub data: Vec<Title>,
}

#[derive(Serialize, ToSchema)]
pub struct TitleResponseBody {
    #[serde(flatten)]
    pub data: Title,
}

#[utoipa::path(get, path = "/api/titles", responses(
    (status = 200, description = "Fetch all titles successful.", body = TitlesResponse),
    (status = 500, description = "Internal server error.", body = ErrorResponse)
))]
pub async fn get_titles(
    State(data): State<Arc<AppState>>,
) -> Result<impl IntoResponse, (StatusCode, Json<ApiResponse<ErrorResponseBody>>)> {
    let titles = sqlx::query_as!(
        Title,
        r#"SELECT id as "id: uuid::Uuid", title, category_id as "category_id: uuid::Uuid", author, description, release_date as "release_date: chrono::DateTime<chrono::Utc>", is_colored, is_completed, thumbnail FROM titles"#
    )
    .fetch_all(&data.sqlite)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse {
                description: String::from("An internal server error has occurred."),
                body: Some(ErrorResponseBody {
                    message: format!("Database error: {}", e),
                })
            })
        )
    })?;

    Ok((
        StatusCode::OK,
        Json(ApiResponse {
            description: String::from("Fetching all titles successful."),
            body: Some(TitlesResponseBody { data: titles }),
        }),
    ))
}

#[utoipa::path(get, path = "/api/title/{title_id}", responses(
    (status = 200, description = "Fetch title successful.", body = TitleResponse),
    (status = 204, description = "Fetch title successful, but one was not found.", body = TitleResponse),
    (status = 500, description = "Internal server error.", body = ErrorResponse)
))]
pub async fn get_title(
    State(data): State<Arc<AppState>>,
    Path(title_id): Path<Uuid>,
) -> Result<impl IntoResponse, (StatusCode, Json<ApiResponse<ErrorResponseBody>>)> {
    let title = sqlx::query_as!(
        Title,
        r#"SELECT id as "id: uuid::Uuid", title, category_id as "category_id: uuid::Uuid", author, description, release_date as "release_date: chrono::DateTime<chrono::Utc>", is_colored, is_completed, thumbnail FROM titles WHERE id = $1"#,
        title_id
    )
    .fetch_optional(&data.sqlite)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse {
                description: String::from("An internal server error has occurred."),
                body: Some(ErrorResponseBody {
                    message: format!("Database error: {}", e),
                })
            })
        )
    })?;

    match title {
        Some(t) => Ok((
            StatusCode::OK,
            Json(ApiResponse {
                description: format!("Fetch title with id {} successful.", title_id),
                body: Some(TitleResponseBody { data: t }),
            }),
        )),
        None => Ok((
            StatusCode::NO_CONTENT,
            Json(ApiResponse {
                description: format!(
                    "The server could not find any titles matching the id {}.",
                    title_id
                ),
                body: None,
            }),
        )),
    }
}
