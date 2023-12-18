use crate::models::users;
use crate::utils::build_resp;
use crate::{
    models::{auth::TokenClaims, prelude::Users},
    routes::{ApiResponse, ErrorResponseBody},
    utils::{build_err_resp, check_pass},
    AppState,
};
use axum::{
    extract::State,
    http::{header, StatusCode},
    response::IntoResponse,
    Json,
};
use axum_extra::extract::cookie::{Cookie, SameSite};
use jsonwebtoken::{encode, EncodingKey, Header};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use utoipa::ToSchema;

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct LoginRequest {
    pub login: String,
    pub password: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct LoginResponseBody {
    pub token: String,
}

/// Login to the server.
#[utoipa::path(post, path = "/api/auth/login", responses(
    (status = 200, description = "Login successful.", body = LoginResponse),
    (status = 500, description = "Internal server error.", body = ErrorResponse),
    (status = 409, description = "A conflict has occurred.", body = ErrorResponse),
    (status = 400, description = "Bad request.", body = ErrorResponse),
))]
pub async fn post_login(
    State(data): State<Arc<AppState>>,
    query: Json<LoginRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<ApiResponse<ErrorResponseBody>>)> {
    let user: users::Model = Users::find()
        .filter(users::Column::Username.eq(&query.login))
        .one(&data.db)
        .await
        .map_err(|e| {
            build_err_resp(
                StatusCode::INTERNAL_SERVER_ERROR,
                String::from("An internal server error has occurred."),
                format!("Database error: {}", e),
            )
        })?
        .ok_or_else(|| {
            build_err_resp(
                StatusCode::BAD_REQUEST,
                String::from("Server has received a bad request."),
                String::from("Invalid username or password."),
            )
        })?;

    if !check_pass(&user.password, &query.password) {
        return Err(build_err_resp(
            StatusCode::BAD_REQUEST,
            String::from("Server has received a bad request."),
            String::from("Invalid username or password."),
        ));
    }

    let now = chrono::Utc::now();
    let iat = now.timestamp() as usize;
    let exp = (now + chrono::Duration::hours(data.env.jwt_maxage_hour)).timestamp() as usize;
    let claims = TokenClaims {
        sub: user.id.to_string(),
        exp,
        iat,
        purpose: None,
    };

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(data.env.jwt_secret.as_ref()),
    )
    .unwrap();

    let cookie = Cookie::build(("token", token.to_owned()))
        .path("/")
        .max_age(time::Duration::minutes(&data.env.jwt_maxage_hour * 60))
        .same_site(SameSite::Lax)
        .http_only(true);

    Ok((
        StatusCode::OK,
        [(header::SET_COOKIE, cookie.to_string())],
        Json(ApiResponse {
            description: String::from("Login successful."),
            body: Some(LoginResponseBody { token }),
        }),
    ))
}
