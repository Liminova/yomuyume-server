use super::{build_err_resp, check_pass};
use crate::{
    models::prelude::*,
    routes::{ApiResponse, ErrorResponseBody},
    AppState,
};
use axum::{extract::State, http::StatusCode, response::IntoResponse, Extension, Json};
use sea_orm::{ActiveModelTrait, Set};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use utoipa::ToSchema;

#[derive(Deserialize, Serialize, Debug, ToSchema)]
pub struct ModifyRequestBody {
    pub username: Option<String>,
    pub email: Option<String>,
    pub password: Option<String>,
    pub new_password: Option<String>,
}

#[utoipa::path(post, path = "/api/user/modify", responses(
    (status = 200, description = "Modify user successful."),
    (status = 400, description = "Bad request.", body = ErrorResponse),
    (status = 401, description = "Unauthorized.", body = ErrorResponse),
    (status = 500, description = "Internal server error.", body = ErrorResponse)
))]
pub async fn post_modify(
    State(data): State<Arc<AppState>>,
    Extension(user): Extension<users::Model>,
    Json(body): Json<ModifyRequestBody>,
) -> Result<impl IntoResponse, (StatusCode, Json<ApiResponse<ErrorResponseBody>>)> {
    let password_in_db = user.password.clone();
    let is_verified = user.is_verified;

    let mut active_user: users::ActiveModel = user.into();

    if let Some(username) = body.username {
        active_user.username = Set(username);
    }

    if let Some(email) = body.email {
        active_user.email = Set(email);
        active_user.is_verified = Set(false);
    }

    if let Some(password) = body.password {
        if !is_verified {
            return Err(build_err_resp(
                StatusCode::UNAUTHORIZED,
                "User is not verified, cannot change password.",
            ));
        }
        if !check_pass(&password_in_db, &password) {
            return Err(build_err_resp(StatusCode::BAD_REQUEST, "Invalid password."));
        }
    }

    if let Some(new_password) = body.new_password {
        active_user.password = Set(new_password);
    }

    active_user.save(&data.db).await.map_err(|e| {
        build_err_resp(
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to update user. Database error: {}", e),
        )
    })?;

    Ok(StatusCode::OK)
}
