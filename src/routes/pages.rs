use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use axum_macros::debug_handler;
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

#[derive(Deserialize, Serialize, ToSchema)]
pub struct PageByTitleIdRequest {
    pub title_id: Uuid,
}

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
                description: format!("Fetch category with id {} successful.", page_id),
                body: Some(PageResponseBody { data: p }),
            }),
        )),
        None => Ok((
            StatusCode::NO_CONTENT,
            Json(ApiResponse {
                description: format!(
                    "The server could not find any categories matching the id {}.",
                    page_id
                ),
                body: None,
            }),
        )),
    }
}

#[debug_handler]
pub async fn post_get_pages_by_title_id(
    State(data): State<Arc<AppState>>,
    query: Json<PageByTitleIdRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<ApiResponse<ErrorResponseBody>>)> {
    let pages = sqlx::query_as!(
        Page,
        r#"SELECT id as "id: uuid::Uuid", title_id as "title_id: uuid::Uuid", path, hash, width as "width: std::primitive::u32", height as "height: std::primitive::u32" FROM pages WHERE title_id = $1"#,
        query.title_id
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
            description: format!(
                "Fetch all pages for title id {} successful.",
                query.title_id
            ),
            body: Some(PagesResponseBody { data: pages }),
        }),
    ))
}
