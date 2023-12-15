use axum::{extract::Query, http::StatusCode, response::IntoResponse};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

use crate::utils::build_resp::build_resp;

#[derive(Deserialize, Serialize, ToSchema)]
struct CheckResponse {
    success: bool,
}

#[derive(Deserialize, IntoParams, ToSchema)]
pub struct CheckRequest {
    pub echo: Option<String>,
}

#[utoipa::path(post, path = "/api/user/check", responses(
    (status = 200, description = "Cookies valid.", body = CheckResponse),
))]
pub async fn get_check(_: Query<CheckRequest>) -> impl IntoResponse {
    build_resp(
        StatusCode::OK,
        String::from("Cookies valid."),
        Some(CheckResponse { success: true }),
    )
}
