use crate::{
    models::{
        auth::{ConfirmReason, ConfirmTokenClaims},
        prelude::Users,
        users,
    },
    routes::{ApiResponse, ErrorResponseBody},
    utils::build_resp::build_err_resp,
    AppState,
};
use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use serde::Deserialize;
use std::sync::Arc;
use utoipa::ToSchema;

#[derive(Deserialize, ToSchema)]
pub struct ForgetRequest {
    pub email: String,
}

#[utoipa::path(post, path = "/api/auth/forget", responses(
    (status = 200, description = "Code sent."),
    (status = 500, description = "Internal server error.", body = ErrorResponse),
    (status = 409, description = "A conflict has occurred.", body = ErrorResponse),
))]
pub async fn post_forget(
    State(data): State<Arc<AppState>>,
    query: Json<ForgetRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<ApiResponse<ErrorResponseBody>>)> {
    if data.env.smtp_host.is_none() {
        return Err(build_err_resp(
            StatusCode::INTERNAL_SERVER_ERROR,
            String::from("An internal server error has occurred."),
            String::from("SMTP is not configured, please contact the server administrator."),
        ));
    }

    if !email_address::EmailAddress::is_valid(&query.email) {
        return Err(build_err_resp(
            StatusCode::BAD_REQUEST,
            String::from("Server has received a bad request."),
            String::from("Invalid email."),
        ));
    }

    let user = Users::find()
        .filter(users::Column::Email.eq(&query.email.to_string().to_ascii_lowercase()))
        .one(&data.db)
        .await
        .map_err(|e| {
            build_err_resp(
                StatusCode::INTERNAL_SERVER_ERROR,
                String::from("An internal server error has occurred."),
                format!("Failed to fetch user from database. Database error: {}", e),
            )
        })?
        .unwrap();

    let now = chrono::Utc::now();
    let iat = now.timestamp() as usize;
    let exp = (now + chrono::Duration::hours(1)).timestamp() as usize;
    let token_claims = ConfirmTokenClaims {
        sub: user.id.clone(),
        iat,
        exp,
        reason: ConfirmReason::Forget,
    };
    let token = match jsonwebtoken::encode(
        &jsonwebtoken::Header::default(),
        &token_claims,
        &jsonwebtoken::EncodingKey::from_secret(data.env.jwt_secret.as_ref()),
    ) {
        Ok(token) => token,
        Err(e) => {
            return Err(build_err_resp(
                StatusCode::INTERNAL_SERVER_ERROR,
                String::from("An internal server error has occurred."),
                format!("Failed to generate token. JWT error: {}", e),
            ))
        }
    };

    let email = format!(
        "Hello, {}!\n\n\
        You have requested to reset your password. Please copy the following token into the app to reset your password:\n\n\
        {}\n\n\
        If you did not request to reset your password, please ignore this email.\n\n\
        Best regards,\n\
        The {} team",
        &user.username,
        token,
        &data.env.app_name,
    );

    match crate::utils::sendmail::sendmail(
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
