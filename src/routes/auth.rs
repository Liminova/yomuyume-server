use std::sync::Arc;

use argon2::{password_hash::SaltString, Argon2, PasswordHasher};
use axum::{
    extract::State,
    http::{header, Request, StatusCode},
    middleware::Next,
    response::IntoResponse,
    Json,
};
use axum_extra::extract::CookieJar;
use jsonwebtoken::{decode, DecodingKey, Validation};
use rand_core::OsRng;
use serde::{Deserialize, Serialize};

use crate::{
    models::{auth::TokenClaims, user::User},
    AppState,
};

use super::ApiResponse;

#[derive(Debug, Serialize)]
pub struct AuthResponseBody {
    pub message: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct RegisterRequest {
    pub username: String,
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct RegisterResponseBody {
    #[serde(flatten)]
    pub user: User,
}

pub async fn auth<B>(
    cookie_jar: CookieJar,
    State(data): State<Arc<AppState>>,
    mut req: Request<B>,
    next: Next<B>,
) -> Result<impl IntoResponse, (StatusCode, Json<ApiResponse<AuthResponseBody>>)> {
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
        (
            StatusCode::UNAUTHORIZED,
            Json(ApiResponse {
                description: String::from("You're not authorized."),
                body: Some(AuthResponseBody {
                    message: String::from("You're not logged in, please provide a token."),
                }),
            }),
        )
    })?;

    let claims = decode::<TokenClaims>(
        &token,
        &DecodingKey::from_secret(data.env.jwt_secret.as_ref()),
        &Validation::default(),
    )
    .map_err(|_| {
        (
            StatusCode::UNAUTHORIZED,
            Json(ApiResponse {
                description: String::from("You're not authorized."),
                body: Some(AuthResponseBody {
                    message: String::from("Invalid token."),
                }),
            }),
        )
    })?
    .claims;

    let user_id = uuid::Uuid::parse_str(&claims.sub).map_err(|_| {
        (
            StatusCode::UNAUTHORIZED,
            Json(ApiResponse {
                description: String::from("You're not authorized."),
                body: Some(AuthResponseBody {
                    message: String::from("Invalid token."),
                }),
            }),
        )
    })?;

    let user = sqlx::query_as!(
        User,
        r#"SELECT id as "id: uuid::Uuid", username, email, password, profile_picture, created_at as "created_at: chrono::DateTime<chrono::Utc>", updated_at as "updated_at: chrono::DateTime<chrono::Utc>" FROM users WHERE id = $1"#,
        user_id
    )
    .fetch_optional(&data.sqlite)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse {
                description: String::from("Failed while fetching user from database."),
                body: Some(AuthResponseBody {
                    message: format!("{}", e),
                }),
            }),
        )
    })?;

    let user = user.ok_or_else(|| {
        (
            StatusCode::UNAUTHORIZED,
            Json(ApiResponse {
                description: String::from("You're not authorized."),
                body: Some(AuthResponseBody {
                    message: String::from("The user belonging to this token no longer exists."),
                }),
            }),
        )
    });

    req.extensions_mut().insert(user);
    Ok(next.run(req).await)
}

#[utoipa::path(post, path = "/api/auth/register", responses((status = 200, description = "Registration successful.", body = RegisterResponse)))]
pub async fn post_register(
    State(data): State<Arc<AppState>>,
    query: Json<RegisterRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<ApiResponse<AuthResponseBody>>)> {
    let email_exists = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM users WHERE email = $1)")
        .bind(query.email.to_owned().to_ascii_lowercase())
        .fetch_one(&data.sqlite)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse {
                    description: String::from("An internal server error has occurred."),
                    body: Some(AuthResponseBody {
                        message: format!(
                            "Failed to fetch user from database. Database error: {}",
                            e
                        ),
                    }),
                }),
            )
        })?;

    if let Some(exists) = email_exists {
        if exists {
            return Err((
                StatusCode::CONFLICT,
                Json(ApiResponse {
                    description: String::from("A conflict has occurred on the server."),
                    body: Some(AuthResponseBody {
                        message: String::from("An user with this email already exists."),
                    }),
                }),
            ));
        }
    }

    let salt = SaltString::generate(&mut OsRng);
    let hashed_password = Argon2::default()
        .hash_password(query.password.as_bytes(), &salt)
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse {
                    description: String::from("An internal server error has occurred."),
                    body: Some(AuthResponseBody {
                        message: format!("Error while hashing password: {}", e),
                    }),
                }),
            )
        })
        .map(|hash| hash.to_string())?;

    let id = uuid::Uuid::new_v4();
    let username = query.username.to_string();
    let email = query.email.to_string().to_ascii_lowercase();
    let created_at = chrono::Utc::now();

    let user = sqlx::query_as!(
        User,
        r#"INSERT INTO users (id, username, email, password, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6) RETURNING id as "id: uuid::Uuid", username, email, password, profile_picture, created_at as "created_at: chrono::DateTime<chrono::Utc>", updated_at as "updated_at: chrono::DateTime<chrono::Utc>""#,
        id,
        username,
        email,
        hashed_password,
        created_at,
        created_at
    ).fetch_one(&data.sqlite)
    .await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse {
                description: String::from("An internal server error has occurred."),
                body: Some(AuthResponseBody {
                    message: format!("Failed to insert user into database. Database error: {}", e)
                }),
            })
        )
    })?;

    Ok((
        StatusCode::OK,
        Json(ApiResponse {
            description: String::from("Registration successful."),
            body: Some(RegisterResponseBody { user }),
        }),
    ))
}
