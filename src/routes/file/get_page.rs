use std::{fs::File, io::Read, path::PathBuf, sync::Arc};

use axum::{
    extract::{Path, State},
    http::{header, StatusCode},
    response::IntoResponse,
    Json,
};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use zip::ZipArchive;

use crate::{
    models::prelude::*,
    routes::{build_err_resp, ApiResponse, ErrorResponseBody},
    AppState,
};

#[utoipa::path(get, path = "/api/file/page/{page_id}", responses(
    (status = 200, description = "Fetch page successful."),
    (status = 401, description = "Unauthorized", body = utoipa::error::ErrorResponse),
    (status = 404, description = "Page not found", body = utoipa::error::ErrorResponse)
))]
pub async fn get_page(
    State(data): State<Arc<AppState>>,
    Path(page_id): Path<String>,
) -> Result<impl IntoResponse, (StatusCode, Json<ApiResponse<ErrorResponseBody>>)> {
    let page_in_db = Pages::find()
        .filter(pages::Column::Id.contains(page_id))
        .one(&data.db)
        .await
        .map_err(|e| {
            build_err_resp(
                StatusCode::INTERNAL_SERVER_ERROR,
                String::from("An internal server error has occurred."),
                format!("Database error: {}", e),
            )
        })?
        .ok_or_else(|| {
            build_err_resp(
                StatusCode::NOT_FOUND,
                String::from("Page not found."),
                String::from("Page not found."),
            )
        })?;

    let title_in_db = Titles::find()
        .filter(titles::Column::Id.contains(&page_in_db.title_id))
        .one(&data.db)
        .await
        .map_err(|e| {
            build_err_resp(
                StatusCode::INTERNAL_SERVER_ERROR,
                String::from("An internal server error has occurred."),
                format!("Database error: {}", e),
            )
        })?
        .ok_or_else(|| {
            build_err_resp(
                StatusCode::NOT_FOUND,
                String::from("Title not found."),
                String::from("Title not found."),
            )
        })?;

    let mut zip = ZipArchive::new(File::open(title_in_db.path).map_err(|e| {
        build_err_resp(
            StatusCode::INTERNAL_SERVER_ERROR,
            String::from("An internal server error has occurred."),
            format!("File error: {}", e),
        )
    })?)
    .map_err(|e| {
        build_err_resp(
            StatusCode::INTERNAL_SERVER_ERROR,
            String::from("An internal server error has occurred."),
            format!("Zip error: {}", e),
        )
    })?;

    let mut file = zip.by_name(page_in_db.path.as_ref()).map_err(|e| {
        build_err_resp(
            StatusCode::INTERNAL_SERVER_ERROR,
            String::from("An internal server error has occurred."),
            format!("Zip error: {}", e),
        )
    })?;

    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer).map_err(|e| {
        build_err_resp(
            StatusCode::INTERNAL_SERVER_ERROR,
            String::from("An internal server error has occurred."),
            format!("File read error: {}", e),
        )
    })?;

    let mime_type = format!(
        "image/{}",
        PathBuf::from(page_in_db.path)
            .extension()
            .map(|s| s.to_str().unwrap_or(""))
            .unwrap_or("")
            .to_ascii_lowercase()
    );

    Ok((StatusCode::OK, [(header::CONTENT_TYPE, mime_type)], buffer))
}
