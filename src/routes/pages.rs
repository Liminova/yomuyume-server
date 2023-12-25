use super::{build_err_resp, build_resp, ApiResponse, ErrorResponseBody};
use crate::{models::prelude::*, AppState};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use sea_orm::*;
use serde::Serialize;
use std::sync::Arc;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Serialize, ToSchema)]
pub struct PagesResponseBody {
    pub data: Vec<pages::Model>,
}

#[derive(Serialize, ToSchema)]
pub struct PageResponseBody {
    pub data: pages::Model,
}

#[utoipa::path(get, path = "/api/pages", responses(
    (status = 200, description = "Fetch all pages successful.", body = PagesResponse),
    (status = 500, description = "Internal server error.", body = ErrorResponse)
))]
pub async fn get_pages(
    State(data): State<Arc<AppState>>,
) -> Result<impl IntoResponse, (StatusCode, Json<ApiResponse<ErrorResponseBody>>)> {
    let pages: Vec<pages::Model> = Pages::find().all(&data.db).await.map_err(|e| {
        build_err_resp(
            StatusCode::INTERNAL_SERVER_ERROR,
            String::from("An internal server error has occurred."),
            format!("Database error: {}", e),
        )
    })?;

    Ok(build_resp(
        StatusCode::OK,
        String::from("Fetching all pages successful."),
        Some(PagesResponseBody { data: pages }),
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
    let page: Option<pages::Model> =
        Pages::find_by_id(page_id)
            .one(&data.db)
            .await
            .map_err(|e| {
                build_err_resp(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    String::from("An internal server error has occurred."),
                    format!("Database error: {}", e),
                )
            })?;

    let resp = match page {
        Some(p) => build_resp(
            StatusCode::OK,
            format!("Fetch page with id {} successful.", page_id),
            Some(PageResponseBody { data: p }),
        ),
        None => build_resp(
            StatusCode::NO_CONTENT,
            format!(
                "The server could not find any pages matching the id {}.",
                page_id
            ),
            None::<PageResponseBody>,
        ),
    };

    Ok(resp)
}

#[utoipa::path(get, path = "/api/pages/by_title_id/{title_id}", responses(
    (status = 200, description = "Fetch all pages for title successful.", body = PagesResponse),
    (status = 500, description = "Internal server error.", body = ErrorResponse)
))]
pub async fn get_pages_by_title_id(
    State(data): State<Arc<AppState>>,
    Path(title_id): Path<Uuid>,
) -> Result<impl IntoResponse, (StatusCode, Json<ApiResponse<ErrorResponseBody>>)> {
    let pages: Vec<pages::Model> = Pages::find()
        .filter(titles::Column::Id.eq(title_id))
        .all(&data.db)
        .await
        .map_err(|e| {
            build_err_resp(
                StatusCode::INTERNAL_SERVER_ERROR,
                String::from("An internal server error has occurred."),
                format!("Database error: {}", e),
            )
        })?;

    Ok(build_resp(
        StatusCode::OK,
        format!("Fetch all pages for title id {} successful.", title_id),
        Some(PagesResponseBody { data: pages }),
    ))
}
