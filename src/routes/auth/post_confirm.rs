use crate::models::prelude::Users;
use crate::{
    models::{auth::TokenClaimsPurpose, users},
    routes::{ApiResponse, ErrorResponseBody},
    utils::build_resp::build_err_resp,
    AppState,
};
use argon2::password_hash::SaltString;
use argon2::{Argon2, PasswordHasher};
use axum::{extract::State, http::StatusCode, response::IntoResponse, Extension, Json};
use rand_core::OsRng;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, Set};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ConfirmRequest {
    pub password: String,
}

#[utoipa::path(post, path = "/api/auth/confirm", responses(
    (status = 200, description = "Password reset successful.", body = ConfirmResponse),
    (status = 500, description = "Internal server error.", body = ErrorResponse),
    (status = 409, description = "A conflict has occurred.", body = ErrorResponse),
    (status = 400, description = "Bad request.", body = ErrorResponse),
))]
pub async fn post_confirm(
    State(data): State<Arc<AppState>>,
    Extension(purpose): Extension<TokenClaimsPurpose>,
    Extension(user): Extension<users::Model>,
    Json(query): Json<ConfirmRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<ApiResponse<ErrorResponseBody>>)> {
    if purpose != TokenClaimsPurpose::ResetPassword {
        return Err(build_err_resp(
            StatusCode::BAD_REQUEST,
            String::from("Server has received a bad request."),
            String::from("This route with POST method is only for resetting password."),
        ));
    }

    if query.password.is_empty() {
        return Err(build_err_resp(
            StatusCode::BAD_REQUEST,
            String::from("Server has received a bad request."),
            String::from("Password cannot be empty."),
        ));
    }

    let user = Users::find()
        .filter(users::Column::Id.eq(user.id))
        .one(&data.db)
        .await
        .map_err(|e| {
            build_err_resp(
                StatusCode::INTERNAL_SERVER_ERROR,
                String::from("An internal server error has occurred."),
                format!("Failed to fetch user from database. Database error: {}", e),
            )
        })?;

    if user.is_none() {
        return Err(build_err_resp(
            StatusCode::BAD_REQUEST,
            String::from("Server has received a bad request."),
            String::from("Invalid user."),
        ));
    }

    let salt = SaltString::generate(&mut OsRng);

    let hashed_password = Argon2::default()
        .hash_password(query.password.as_bytes(), &salt)
        .map_err(|e| {
            build_err_resp(
                StatusCode::INTERNAL_SERVER_ERROR,
                String::from("An internal server error has occurred."),
                format!("Error while hashing password: {}", e),
            )
        })?
        .to_string();

    let mut user: users::ActiveModel = user.unwrap().into();
    user.password = Set(hashed_password);

    Ok(StatusCode::OK)
}
