use std::sync::Arc;

use argon2::{password_hash::SaltString, Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use axum::{
    body::Body,
    extract::State,
    http::{header, Request, StatusCode},
    middleware::Next,
    response::IntoResponse,
    Json,
};
use axum_extra::extract::{
    cookie::{Cookie, SameSite},
    CookieJar,
};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use rand_core::OsRng;
use sea_orm::*;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::{
    models::{auth::TokenClaims, prelude::Users, users, users::Model as User},
    AppState,
};

use super::{ApiResponse, ErrorResponseBody};

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct RegisterRequest {
    pub username: String,
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct RegisterResponseBody {
    pub user: User,
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct LoginRequest {
    pub login: String,
    pub password: String,
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct LoginResponseBody {
    pub token: String,
}

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
        (
            StatusCode::UNAUTHORIZED,
            Json(ApiResponse {
                description: String::from("You're not authorized."),
                body: Some(ErrorResponseBody {
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
                body: Some(ErrorResponseBody {
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
                body: Some(ErrorResponseBody {
                    message: String::from("Invalid token."),
                }),
            }),
        )
    })?;

    let user: Option<User> = Users::find_by_id(user_id)
        .one(&data.db)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse {
                    description: String::from("An internal server error has occurred."),
                    body: Some(ErrorResponseBody {
                        message: format!("Database error: {}", e),
                    }),
                }),
            )
        })?;

    let user = user.ok_or_else(|| {
        (
            StatusCode::UNAUTHORIZED,
            Json(ApiResponse {
                description: String::from("You're not authorized."),
                body: Some(ErrorResponseBody {
                    message: String::from("The user belonging to this token no longer exists."),
                }),
            }),
        )
    });

    req.extensions_mut().insert(user);
    Ok(next.run(req).await)
}

#[utoipa::path(post, path = "/api/auth/register", responses(
    (status = 200, description = "Registration successful.", body = RegisterResponse),
    (status = 500, description = "Internal server error.", body = ErrorResponse),
    (status = 409, description = "A conflict has occurred.", body = ErrorResponse),
))]
pub async fn post_register(
    State(data): State<Arc<AppState>>,
    query: Json<RegisterRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<ApiResponse<ErrorResponseBody>>)> {
    let email_exists: Option<User> = Users::find()
        .from_raw_sql(Statement::from_sql_and_values(
            DatabaseBackend::Sqlite,
            r#"SELECT EXISTS(SELECT 1 FROM users WHERE email = $1)"#,
            [query.email.clone().into()],
        ))
        .one(&data.db)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse {
                    description: String::from("An internal server error has occurred."),
                    body: Some(ErrorResponseBody {
                        message: format!(
                            "Failed to fetch user from database. Database error: {}",
                            e
                        ),
                    }),
                }),
            )
        })?;

    if email_exists.is_some() {
        return Err((
            StatusCode::CONFLICT,
            Json(ApiResponse {
                description: String::from("A conflict has occurred on the server."),
                body: Some(ErrorResponseBody {
                    message: String::from("An user with this email already exists."),
                }),
            }),
        ));
    }

    let salt = SaltString::generate(&mut OsRng);
    let hashed_password = Argon2::default()
        .hash_password(query.password.as_bytes(), &salt)
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse {
                    description: String::from("An internal server error has occurred."),
                    body: Some(ErrorResponseBody {
                        message: format!("Error while hashing password: {}", e),
                    }),
                }),
            )
        })
        .map(|hash| hash.to_string())?;

    let id = uuid::Uuid::new_v4().to_string();
    let username = query.username.to_string();
    let email = query.email.to_string().to_ascii_lowercase();
    let created_at = chrono::Utc::now().to_string();

    let user = users::ActiveModel {
        id: Set(id),
        username: Set(username),
        email: Set(email),
        created_at: Set(created_at.clone()),
        updated_at: Set(created_at),
        password: Set(hashed_password),
        ..Default::default()
    };

    let user = user.insert(&data.db).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse {
                description: String::from("An internal server error has occurred."),
                body: Some(ErrorResponseBody {
                    message: format!("Failed to insert user into database. Database error: {}", e),
                }),
            }),
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

#[utoipa::path(post, path = "/api/auth/login", responses(
    (status = 200, description = "Login successful.", body = LoginResponse),
    (status = 500, description = "Internal server error.", body = ErrorResponse),
    (status = 409, description = "A conflict has occurred.", body = ErrorResponse),
))]
pub async fn post_login(
    State(data): State<Arc<AppState>>,
    query: Json<LoginRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<ApiResponse<ErrorResponseBody>>)> {
    let user: User = Users::find()
        .filter(users::Column::Username.eq(&query.login))
        .one(&data.db)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse {
                    description: String::from("An internal server error has occurred."),
                    body: Some(ErrorResponseBody {
                        message: format!("Database error: {}", e),
                    }),
                }),
            )
        })?
        .ok_or_else(|| {
            (
                StatusCode::BAD_REQUEST,
                Json(ApiResponse {
                    description: String::from("Server has received a bad request."),
                    body: Some(ErrorResponseBody {
                        message: String::from("Invalid username or password."),
                    }),
                }),
            )
        })?;

    let is_valid = match PasswordHash::new(&user.password) {
        Ok(parsed_hash) => Argon2::default()
            .verify_password(query.password.as_bytes(), &parsed_hash)
            .map_or(false, |_| true),
        Err(_) => false,
    };

    if !is_valid {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ApiResponse {
                description: String::from("Server has received a bad request"),
                body: Some(ErrorResponseBody {
                    message: String::from("Invalid username or password."),
                }),
            }),
        ));
    }

    let now = chrono::Utc::now();
    let iat = now.timestamp() as usize;
    let exp = (now + chrono::Duration::minutes(60)).timestamp() as usize;
    let claims = TokenClaims {
        sub: user.id.to_string(),
        exp,
        iat,
    };

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(data.env.jwt_secret.as_ref()),
    )
    .unwrap();

    let cookie = Cookie::build(("token", token.to_owned()))
        .path("/")
        .max_age(time::Duration::minutes(&data.env.jwt_maxage * 60))
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

#[utoipa::path(get, path = "/api/auth/logout", responses((status = 200, description = "Logout successful.", body = ErrorResponse)))]
pub async fn get_logout(
) -> Result<impl IntoResponse, (StatusCode, Json<ApiResponse<ErrorResponseBody>>)> {
    let cookie = Cookie::build(("token", ""))
        .path("/")
        .max_age(time::Duration::hours(-1))
        .same_site(SameSite::Lax)
        .http_only(true);

    Ok((
        StatusCode::OK,
        [(header::SET_COOKIE, cookie.to_string())],
        Json(ApiResponse::<ErrorResponseBody> {
            description: String::from("Logout successful."),
            body: None,
        }),
    ))
}
