use super::super::build_err_resp;
use crate::routes::ApiResponse;
use crate::routes::ErrorResponseBody;
use crate::{
    models::{auth::TokenClaims, prelude::Users, users::Model as User},
    AppState,
};
use axum::{
    body::Body,
    extract::State,
    http::{header, Request, StatusCode},
    middleware::Next,
    response::IntoResponse,
    Json,
};
use axum_extra::extract::CookieJar;
use jsonwebtoken::{decode, DecodingKey, Validation};
use sea_orm::*;
use std::sync::Arc;

pub async fn auth(
    cookie_jar: CookieJar,
    State(data): State<Arc<AppState>>,
    mut req: Request<Body>,
    next: Next,
) -> Result<impl IntoResponse, (StatusCode, Json<ApiResponse<ErrorResponseBody>>)> {
    let token = cookie_jar
        .get("token")
        .map(|cookie| cookie.value().to_string())
        .or_else(|| {
            req.headers()
                .get(header::AUTHORIZATION)
                .and_then(|auth_header| auth_header.to_str().ok())
                .and_then(|auth_value| {
                    auth_value
                        .strip_prefix("Bearer ")
                        .map(|stripped| stripped.to_owned())
                })
        });

    let token = token.ok_or_else(|| {
        build_err_resp(
            StatusCode::UNAUTHORIZED,
            String::from("You're not authorized."),
            String::from("You're not logged in, please provide a token."),
        )
    })?;

    let claims = decode::<TokenClaims>(
        &token,
        &DecodingKey::from_secret(data.env.jwt_secret.as_ref()),
        &Validation::default(),
    )
    .map_err(|_| {
        build_err_resp(
            StatusCode::UNAUTHORIZED,
            String::from("You're not authorized."),
            String::from("Invalid token."),
        )
    })?
    .claims;

    let user_id = uuid::Uuid::parse_str(&claims.sub).map_err(|_| {
        build_err_resp(
            StatusCode::UNAUTHORIZED,
            String::from("You're not authorized."),
            String::from("Invalid token."),
        )
    })?;

    let user: Option<User> = Users::find_by_id(user_id)
        .one(&data.db)
        .await
        .map_err(|e| {
            build_err_resp(
                StatusCode::INTERNAL_SERVER_ERROR,
                String::from("An internal server error has occurred."),
                format!("Database error: {}", e),
            )
        })?;

    if let Some(user) = user {
        req.extensions_mut().insert(user);
        req.extensions_mut()
            .insert(claims.purpose.unwrap_or_default());
        Ok(next.run(req).await)
    } else {
        Err(build_err_resp(
            StatusCode::UNAUTHORIZED,
            String::from("You're not authorized."),
            String::from("The user belonging to this token no longer exists."),
        ))
    }
}