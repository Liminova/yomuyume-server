use axum::{http::StatusCode, response::IntoResponse};

/// Check if the cookies are valid.
#[utoipa::path(get, path = "/api/user/check", responses(
    (status = 200, description = "Cookies valid."),
))]
pub async fn get_check() -> impl IntoResponse {
    StatusCode::OK
}
