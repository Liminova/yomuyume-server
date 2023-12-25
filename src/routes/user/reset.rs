use super::{build_err_resp, sendmail};
use crate::{
    models::{
        auth::{TokenClaims, TokenClaimsPurpose},
        prelude::*,
    },
    routes::{ApiResponse, ErrorResponseBody},
    AppState,
};
use argon2::{password_hash::SaltString, Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Extension, Json,
};
use rand_core::OsRng;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ResetRequestBody {
    pub password: String,
}

/// Send a request to change the password when the user has forgotten it.
///
/// The user will receive an email with a token to confirm the modification.
#[utoipa::path(get, path = "/api/user/reset/{email}", responses(
    (status = 200, description = "Token sent to user's email."),
    (status = 500, description = "Internal server error.", body = ErrorResponse),
    (status = 409, description = "A conflict has occurred.", body = ErrorResponse),
))]
pub async fn get_reset(
    State(data): State<Arc<AppState>>,
    Path(email): Path<String>,
) -> Result<impl IntoResponse, (StatusCode, Json<ApiResponse<ErrorResponseBody>>)> {
    if data.env.smtp_host.is_none() {
        return Err(build_err_resp(
            StatusCode::INTERNAL_SERVER_ERROR,
            String::from("An internal server error has occurred."),
            String::from("SMTP is not configured, please contact the server administrator."),
        ));
    }

    if !email_address::EmailAddress::is_valid(&email) {
        return Err(build_err_resp(
            StatusCode::BAD_REQUEST,
            String::from("Server has received a bad request."),
            String::from("Invalid email."),
        ));
    }

    let user = Users::find()
        .filter(users::Column::Email.eq(&email.to_string().to_ascii_lowercase()))
        .one(&data.db)
        .await
        .map_err(|e| {
            build_err_resp(
                StatusCode::INTERNAL_SERVER_ERROR,
                String::from("An internal server error has occurred."),
                format!("Failed to fetch user from database. Database error: {}", e),
            )
        })?
        .ok_or_else(|| {
            build_err_resp(
                StatusCode::CONFLICT,
                String::from("A conflict has occurred."),
                String::from("User not found."),
            )
        })?;

    if !user.is_verified {
        return Err(build_err_resp(
            StatusCode::CONFLICT,
            String::from("A conflict has occurred."),
            String::from("User is not verified."),
        ));
    }

    let now = chrono::Utc::now();
    let token_claims = TokenClaims {
        sub: user.id.clone(),
        iat: now.timestamp() as usize,
        exp: (now + chrono::Duration::hours(1)).timestamp() as usize,
        purpose: Some(TokenClaimsPurpose::ResetPassword),
    };
    let token = jsonwebtoken::encode(
        &jsonwebtoken::Header::default(),
        &token_claims,
        &jsonwebtoken::EncodingKey::from_secret(data.env.jwt_secret.as_ref()),
    )
    .map_err(|e| {
        build_err_resp(
            StatusCode::INTERNAL_SERVER_ERROR,
            String::from("An internal server error has occurred."),
            format!("Failed to generate token. JWT error: {}", e),
        )
    })?;

    let email = format!(
        "Hello, {}!\n\n\
        You have requested to reset your password. Please copy the following token into the app to continue:\n\n\
        {}\n\n\
        If you did not request to reset your password, please ignore this email.\n\n\
        Best regards,\n\
        The {} team",
        &user.username,
        token,
        &data.env.app_name,
    );

    match sendmail(
        &data.env,
        &user.username,
        &user.email,
        &format!("{} - Reset your password", &data.env.app_name),
        &email,
    ) {
        Ok(_) => Ok(StatusCode::OK),
        Err(e) => Err(build_err_resp(
            StatusCode::INTERNAL_SERVER_ERROR,
            String::from("An internal server error has occurred."),
            format!("Failed to send email. SMTP error: {}", e),
        )),
    }
}

/// Confirm the password modification.
///
/// The user will make a request with the token received by email.
#[utoipa::path(post, path = "/api/user/reset", responses(
    (status = 200, description = "Password reset successful."),
    (status = 500, description = "Internal server error.", body = ErrorResponse),
    (status = 400, description = "Bad request.", body = ErrorResponse),
))]
pub async fn post_reset(
    State(data): State<Arc<AppState>>,
    Extension(purpose): Extension<TokenClaimsPurpose>,
    Extension(user): Extension<users::Model>,
    Json(query): Json<ResetRequestBody>,
) -> Result<impl IntoResponse, (StatusCode, Json<ApiResponse<ErrorResponseBody>>)> {
    if purpose != TokenClaimsPurpose::ResetPassword {
        return Err(build_err_resp(
            StatusCode::BAD_REQUEST,
            String::from("Server has received a bad request."),
            String::from("Invalid request purpose."),
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
        })?
        .ok_or_else(|| {
            build_err_resp(
                StatusCode::BAD_REQUEST,
                String::from("Server has received a bad request."),
                String::from("Invalid user."),
            )
        })?;

    let is_valid = match PasswordHash::new(&user.password) {
        Ok(parsed_hash) => Argon2::default()
            .verify_password(query.password.as_bytes(), &parsed_hash)
            .map_or(false, |_| true),
        Err(_) => false,
    };
    if !is_valid {
        return Err(build_err_resp(
            StatusCode::BAD_REQUEST,
            String::from("Server has received a bad request."),
            String::from("Invalid password."),
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

    let mut user: users::ActiveModel = user.into();
    user.password = Set(hashed_password);
    user.save(&data.db).await.map_err(|e| {
        build_err_resp(
            StatusCode::INTERNAL_SERVER_ERROR,
            String::from("An internal server error has occurred."),
            format!("Failed to update user. Database error: {}", e),
        )
    })?;

    Ok(StatusCode::OK)
}
