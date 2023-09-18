use axum::{http::StatusCode, response::IntoResponse, Json};
use serde::{Deserialize, Serialize};
use serde_email::Email;
use utoipa::ToSchema;

use super::ApiResponse;

#[derive(Deserialize, Serialize, ToSchema)]
pub struct AuthRequest {
    /// User's email address.
    pub email: String,
    /// OTP code, sent to the given email address.
    pub code: String,
}

#[derive(Deserialize, Serialize, ToSchema)]
pub struct AuthResponseBody {
    /// Authorization token.
    pub token: String,
}

#[utoipa::path(post, path = "/api/auth", responses((status = 200, description = "Authentication successful.", body = AuthResponse), (status = 400, description = "Invalid or missing email.")))]
pub async fn auth(query: Json<AuthRequest>) -> impl IntoResponse {
    let email = Email::from_string(query.email.clone());
    if let Ok(email) = email {
        // TODO: do auth things here, returning magic value for now.
        (
            StatusCode::OK,
            Json(ApiResponse {
                description: String::from("Authentication successful."),
                body: Some(AuthResponseBody {
                    token: String::from("cirno"),
                }),
            }),
        )
    } else {
        (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse {
                description: String::from("Invalid or missing email."),
                body: None,
            }),
        )
    }
}
