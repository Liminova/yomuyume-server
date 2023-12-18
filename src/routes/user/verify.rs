use crate::{
    models::{
        auth::{TokenClaims, TokenClaimsPurpose},
        users,
    },
    routes::{ApiResponse, ErrorResponseBody},
    utils::{build_err_resp, sendmail},
    AppState,
};
use axum::{extract::State, http::StatusCode, response::IntoResponse, Extension, Json};
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};
use std::sync::Arc;

#[utoipa::path(get, path = "/api/user/verify", responses(
    (status = 200, description = "Verification email sent."),
    (status = 500, description = "Internal server error.", body = ErrorResponse),
))]
pub async fn get_verify(
    State(data): State<Arc<AppState>>,
    Extension(user): Extension<users::Model>,
) -> Result<impl IntoResponse, (StatusCode, Json<ApiResponse<ErrorResponseBody>>)> {
    if data.env.smtp_host.is_none() {
        return Err(build_err_resp(
            StatusCode::INTERNAL_SERVER_ERROR,
            String::from("An internal server error has occurred."),
            String::from("SMTP is not configured, please contact the server administrator."),
        ));
    }

    let now = chrono::Utc::now();
    let token_claims = TokenClaims {
        sub: user.id,
        iat: now.timestamp() as usize,
        exp: (now + chrono::Duration::hours(1)).timestamp() as usize,
        purpose: Some(TokenClaimsPurpose::VerifyRegister),
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

    let body = format!(
        "Hello {},\n\n\
        You have requested to verify your account. \
        Please click copy the following token into the app to continue:\n\n\
        {}\n\n\
        If you did not request this, please ignore this email.\n\n\
        Thanks,\n\
        The {} Team",
        &user.username, token, &data.env.app_name,
    );

    match sendmail(
        &data.env,
        &user.username,
        &user.email,
        &format!("{} - Verify your account", &data.env.app_name),
        &body,
    ) {
        Ok(_) => Ok(StatusCode::OK),
        Err(e) => Err(build_err_resp(
            StatusCode::INTERNAL_SERVER_ERROR,
            String::from("An internal server error has occurred."),
            format!("Failed to send email. SMTP error: {}", e),
        )),
    }
}

/// Verify a user's account with the token sent to their email
/// get sent to their email by this same route using a GET request.
#[utoipa::path(post, path = "/api/user/verify", responses(
    (status = 200, description = "Account verification successful."),
    (status = 500, description = "Internal server error.", body = ErrorResponse),
    (status = 400, description = "Bad request.", body = ErrorResponse),
))]
pub async fn post_verify(
    State(data): State<Arc<AppState>>,
    Extension(purpose): Extension<TokenClaimsPurpose>,
    Extension(user): Extension<users::Model>,
) -> Result<impl IntoResponse, (StatusCode, Json<ApiResponse<ErrorResponseBody>>)> {
    if purpose != TokenClaimsPurpose::VerifyRegister {
        return Err(build_err_resp(
            StatusCode::BAD_REQUEST,
            String::from("Server has received a bad request."),
            String::from("Invalid request purpose."),
        ));
    }

    let user = users::Entity::find()
        .filter(users::Column::Id.eq(&user.id))
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

    let mut user: users::ActiveModel = user.into();
    user.is_verified = Set(true);

    user.save(&data.db).await.map_err(|e| {
        build_err_resp(
            StatusCode::INTERNAL_SERVER_ERROR,
            String::from("An internal server error has occurred."),
            format!("Failed to update user. Database error: {}", e),
        )
    })?;

    Ok(StatusCode::OK)
}
