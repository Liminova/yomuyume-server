use super::build_err_resp;
use crate::{
    models::{prelude::Users, users},
    routes::{ApiResponse, ErrorResponseBody},
    AppState,
};
use argon2::{password_hash::SaltString, Argon2, PasswordHasher};
use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use rand_core::OsRng;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use utoipa::ToSchema;

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct RegisterRequest {
    pub username: String,
    pub email: String,
    pub password: String,
}

/// Register a new user.
#[utoipa::path(post, path = "/api/auth/register", responses(
    (status = 200, description = "Registration successful.", body = RegisterResponse),
    (status = 500, description = "Internal server error.", body = ErrorResponse),
    (status = 409, description = "A conflict has occurred.", body = ErrorResponse),
))]
pub async fn post_register(
    State(data): State<Arc<AppState>>,
    query: Json<RegisterRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<ApiResponse<ErrorResponseBody>>)> {
    if !email_address::EmailAddress::is_valid(&query.email) {
        return Err(build_err_resp(
            StatusCode::BAD_REQUEST,
            String::from("Server has received a bad request."),
            String::from("Invalid email."),
        ));
    }

    let email_exists = Users::find()
        .filter(users::Column::Email.eq(&query.email.to_string().to_ascii_lowercase()))
        .one(&data.db)
        .await
        .map_err(|e| {
            build_err_resp(
                StatusCode::INTERNAL_SERVER_ERROR,
                String::from("An internal server error has occurred."),
                format!("Failed to fetch user from database. Database error: {}", e),
            )
        })?;

    if email_exists.is_some() {
        return Err(build_err_resp(
            StatusCode::CONFLICT,
            String::from("A conflict has occurred on the server."),
            String::from("An user with this email already exists."),
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

    user.insert(&data.db).await.map_err(|e| {
        build_err_resp(
            StatusCode::INTERNAL_SERVER_ERROR,
            String::from("An internal server error has occurred."),
            format!("Failed to insert user into database. Database error: {}", e),
        )
    })?;

    Ok(StatusCode::OK)
}
