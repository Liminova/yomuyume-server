use super::{build_err_resp, check_pass, sendmail};
use crate::{
    models::{
        auth::{TokenClaims, TokenClaimsPurpose},
        prelude::*,
    },
    routes::{ApiResponse, ErrorResponseBody},
    AppState,
};
use axum::{extract::State, http::StatusCode, response::IntoResponse, Extension, Json};
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct DeleteRequestBody {
    pub password: String,
}

/// Send a request to delete the user.
///
/// The user will receive an email with a token to confirm the deletion.
#[utoipa::path(get, path = "/api/user/delete", responses(
    (status = 200, description = "Token sent to user's email."),
    (status = 500, description = "Internal server error.", body = ErrorResponse),
    (status = 400, description = "Bad request.", body = ErrorResponse),
))]
pub async fn get_delete(
    State(data): State<Arc<AppState>>,
    Extension(user): Extension<users::Model>,
) -> Result<impl IntoResponse, (StatusCode, Json<ApiResponse<ErrorResponseBody>>)> {
    if data.env.smtp_host.is_none() {
        return Err(build_err_resp(
            StatusCode::INTERNAL_SERVER_ERROR,
            "SMTP is not configured, please contact the server administrator.",
        ));
    }

    let now = chrono::Utc::now();
    let token_claims = TokenClaims {
        sub: user.id,
        iat: now.timestamp() as usize,
        exp: (now + chrono::Duration::hours(1)).timestamp() as usize,
        purpose: Some(TokenClaimsPurpose::DeleteAccount),
    };
    let token = jsonwebtoken::encode(
        &jsonwebtoken::Header::default(),
        &token_claims,
        &jsonwebtoken::EncodingKey::from_secret(data.env.jwt_secret.as_ref()),
    )
    .map_err(|e| {
        build_err_resp(
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to generate token. JWT error: {}", e),
        )
    })?;

    let email = format!(
        "Hello, {}!\n\n\
        // You have requested to delete your account. Please copy the following token into the app to continue:\n\n\
        {}\n\n\
        If you did not request to delete your account, please ignore this email.\n\n\
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
        &format!("{} - Delete your password", &data.env.app_name),
        &email,
    ) {
        Ok(_) => Ok(StatusCode::OK),
        Err(e) => Err(build_err_resp(
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to send email. SMTP error: {}", e),
        )),
    }
}

/// Confirm the deletion of the user.
///
/// The user will make a request with the token received by email.
#[utoipa::path(post, path = "/api/user/delete", responses(
    (status = 200, description = "User deleted."),
    (status = 500, description = "Internal server error.", body = ErrorResponse),
    (status = 400, description = "Bad request.", body = ErrorResponse),
))]
pub async fn post_delete(
    State(data): State<Arc<AppState>>,
    Extension(user): Extension<users::Model>,
    Json(query): Json<DeleteRequestBody>,
) -> Result<impl IntoResponse, (StatusCode, Json<ApiResponse<ErrorResponseBody>>)> {
    if query.password.is_empty() {
        return Err(build_err_resp(
            StatusCode::BAD_REQUEST,
            "Password cannot be empty.",
        ));
    }

    if !check_pass(&user.password, &query.password) {
        return Err(build_err_resp(
            StatusCode::BAD_REQUEST,
            "Invalid username or password.",
        ));
    }

    let user = Users::find()
        .filter(users::Column::Id.eq(user.id))
        .one(&data.db)
        .await
        .map_err(|e| {
            build_err_resp(
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to fetch user from database. Database error: {}", e),
            )
        })?
        .ok_or_else(|| build_err_resp(StatusCode::BAD_REQUEST, "Invalid user."))?;

    let user: users::ActiveModel = user.into();

    user.delete(&data.db).await.map_err(|e| {
        build_err_resp(
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to delete user. Database error: {}", e),
        )
    })?;

    Ok(StatusCode::OK)
}
