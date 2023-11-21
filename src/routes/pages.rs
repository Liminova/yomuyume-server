use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::{models::page::Page, AppState};

use super::{ApiResponse, ErrorResponseBody};

#[derive(Serialize, ToSchema)]
pub struct PagesResponseBody {
    pub data: Vec<Page>,
}

#[derive(Serialize, ToSchema)]
pub struct PageResponseBody {
    pub data: Page,
}

#[utoipa::path(get, path = "/api/pages", responses(
    (status = 200, description = "Fetch all pages successful.", body = PagesResponse),
    (status = 500, description = "Internal server error.", body = ErrorResponse)
))]
pub async fn get_pages(
    State(data): State<Arc<AppState>>,
) -> Result<impl IntoResponse, (StatusCode, Json<ApiResponse<ErrorResponseBody>>)> {
    let pages = sqlx::query_as!(
        Page,
        r#"SELECT id as "id: uuid::Uuid", title_id as "title_id: uuid::Uuid", path, hash, width as "width: std::primitive::u32", height as "height: std::primitive::u32" FROM pages"#
    )
    .fetch_all(&data.sqlite)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse {
                description: String::from("An internal server error has occurred."),
                body: Some(ErrorResponseBody {
                    message: format!("Database error: {}", e)
                })
            })
        )
    })?;

    Ok((
        StatusCode::OK,
        Json(ApiResponse {
            description: String::from("Fetching all pages successful."),
            body: Some(PagesResponseBody { data: pages }),
        }),
    ))
}

#[utoipa::path(get, path = "/api/page/{page_id}", responses(
    (status = 200, description = "Fetch page successful.", body = PageResponse),
    (status = 204, description = "Fetch page successful, but one was not found.", body = PageResponse),
    (status = 500, description = "Internal server error.", body = ErrorResponse)
))]
pub async fn get_page(
    State(data): State<Arc<AppState>>,
    Path(page_id): Path<Uuid>,
) -> Result<impl IntoResponse, (StatusCode, Json<ApiResponse<ErrorResponseBody>>)> {
    let page = sqlx::query_as!(
        Page,
        r#"SELECT id as "id: uuid::Uuid", title_id as "title_id: uuid::Uuid", path, hash, width as "width: std::primitive::u32", height as "height: std::primitive::u32" FROM pages WHERE id = $1"#,
        page_id
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
                }),
            }),
        )
    })?;

    match page {
        Some(p) => Ok((
            StatusCode::OK,
            Json(ApiResponse {
                description: format!("Fetch page with id {} successful.", page_id),
                body: Some(PageResponseBody { data: p }),
            }),
        )),
        None => Ok((
            StatusCode::NO_CONTENT,
            Json(ApiResponse {
                description: format!(
                    "The server could not find any pages matching the id {}.",
                    page_id
                ),
                body: None,
            }),
        )),
    }
}

#[utoipa::path(get, path = "/api/pages/by_title_id/{title_id}", responses(
    (status = 200, description = "Fetch all pages for title successful.", body = PagesResponse),
    (status = 500, description = "Internal server error.", body = ErrorResponse)
))]
pub async fn get_pages_by_title_id(
    State(data): State<Arc<AppState>>,
    Path(title_id): Path<Uuid>,
) -> Result<impl IntoResponse, (StatusCode, Json<ApiResponse<ErrorResponseBody>>)> {
    let pages = sqlx::query_as!(
        Page,
        r#"SELECT id as "id: uuid::Uuid", title_id as "title_id: uuid::Uuid", path, hash, width as "width: std::primitive::u32", height as "height: std::primitive::u32" FROM pages WHERE title_id = $1"#,
        title_id
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
            description: format!("Fetch all pages for title id {} successful.", title_id),
            body: Some(PagesResponseBody { data: pages }),
        }),
    ))
}
