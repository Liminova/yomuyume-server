use std::sync::Arc;

use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use serde::Serialize;
use utoipa::ToSchema;

use crate::{
    routes::{build_resp, ApiResponse, ErrorResponseBody},
    AppState,
};

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct ScanningProgressResponseBody {
    scanning_completed: bool,
    scanning_progress: f64,
}

#[utoipa::path(get, path = "/api/utils/scanning_progress", responses(
    (status = 200, description = "", body = ScanningProgressResponse),
))]
pub async fn get_scanning_progress(
    State(data): State<Arc<AppState>>,
) -> Result<impl IntoResponse, (StatusCode, Json<ApiResponse<ErrorResponseBody>>)> {
    let scanning_complete = data.scanning_complete.lock().await;
    let scanning_progress = data.scanning_progress.lock().await;

    Ok(build_resp(
        StatusCode::OK,
        ScanningProgressResponseBody {
            scanning_completed: *scanning_complete,
            scanning_progress: *scanning_progress,
        },
    ))
}
