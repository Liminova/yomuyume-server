use axum::{http::StatusCode, response::IntoResponse};

#[utoipa::path(post, path = "/api/user/check", responses(
    (status = 200, description = "Cookies valid."),
))]
pub async fn get_check() -> impl IntoResponse {
    StatusCode::OK
}
