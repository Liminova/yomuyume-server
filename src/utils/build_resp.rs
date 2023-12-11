use axum::{http::StatusCode, Json};
use serde::Serialize;

use crate::routes::{ApiResponse, ErrorResponseBody};

pub fn build_resp<T: Serialize>(
    status: StatusCode,
    description: String,
    body: T,
) -> (StatusCode, Json<ApiResponse<T>>) {
    (
        status,
        Json(ApiResponse {
            description,
            body: Some(body),
        }),
    )
}

pub fn build_err_resp(
    status: StatusCode,
    description: String,
    body: String,
) -> (StatusCode, Json<ApiResponse<ErrorResponseBody>>) {
    (
        status,
        Json(ApiResponse {
            description,
            body: Some(ErrorResponseBody { message: body }),
        }),
    )
}
