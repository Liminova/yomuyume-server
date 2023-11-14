use std::sync::Arc;

use argon2::{password_hash::SaltString, Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use axum::{
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
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::{
    models::{auth::TokenClaims, user::User},
    AppState,
};

use super::ApiResponse;

#[derive(Debug, Serialize, ToSchema)]
pub struct AuthResponseBody {
    pub message: String,
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct RegisterRequest {
    pub username: String,
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct RegisterResponseBody {
    #[serde(flatten)]
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

#[utoipa::path(post, path = "/api/auth/register", responses(
    (status = 200, description = "Registration successful.", body = RegisterResponse),
    (status = 500, description = "Internal server error.", body = AuthResponse),
    (status = 409, description = "A conflict has occurred.", body = AuthResponse),
))]
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

#[utoipa::path(post, path = "/api/auth/login", responses(
    (status = 200, description = "Login successful.", body = LoginResponse),
    (status = 500, description = "Internal server error.", body = AuthResponse),
    (status = 409, description = "A conflict has occurred.", body = AuthResponse),
))]
pub async fn post_login(
    State(data): State<Arc<AppState>>,
    query: Json<LoginRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<ApiResponse<AuthResponseBody>>)> {
    let user = sqlx::query_as!(
        User,
        r#"SELECT id as "id: uuid::Uuid", username, email, password, profile_picture, created_at as "created_at: chrono::DateTime<chrono::Utc>", updated_at as "updated_at: chrono::DateTime<chrono::Utc>" FROM users WHERE username = $1"#,
        query.login
    ).fetch_optional(&data.sqlite).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse {
                description: String::from("An internal server error has occurred."),
                body: Some(AuthResponseBody {
                    message: format!("Failed to fetch user from database. Database error: {}", e),
                })
            })
        )
    })?
    .ok_or_else(|| {
        (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse {
                description: String::from("Server has received a bad request."),
                    body: Some(AuthResponseBody {
                        message: String::from("Invalid username or password."),
                    })
            })
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
                body: Some(AuthResponseBody {
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

    let cookie = Cookie::build("token", token.to_owned())
        .path("/")
        .max_age(time::Duration::minutes(&data.env.jwt_maxage * 60))
        .same_site(SameSite::Lax)
        .http_only(true)
        .finish();

    Ok((
        StatusCode::OK,
        [(header::SET_COOKIE, cookie.to_string())],
        Json(ApiResponse {
            description: String::from("Login successful."),
            body: Some(LoginResponseBody { token }),
        }),
    ))
}

#[utoipa::path(get, path = "/api/auth/login", responses((status = 200, description = "Logout successful.", body = AuthResponse)))]
pub async fn get_logout(
) -> Result<impl IntoResponse, (StatusCode, Json<ApiResponse<AuthResponseBody>>)> {
    let cookie = Cookie::build("token", "")
        .path("/")
        .max_age(time::Duration::hours(-1))
        .same_site(SameSite::Lax)
        .http_only(true)
        .finish();

    Ok((
        StatusCode::OK,
        [(header::SET_COOKIE, cookie.to_string())],
        Json(ApiResponse::<AuthResponseBody> {
            description: String::from("Logout successful."),
            body: None,
        }),
    ))
}
