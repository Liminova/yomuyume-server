use axum::{http::StatusCode, response::IntoResponse};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Deserialize, Serialize, ToSchema)]
pub struct CheckResponseBody {
    success: bool,
}

#[utoipa::path(post, path = "/api/user/check", responses(
    (status = 200, description = "Cookies valid.", body = CheckResponse),
))]
pub async fn get_check() -> impl IntoResponse {
    StatusCode::OK
}
