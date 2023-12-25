use super::build_err_resp;
use crate::{
    models::prelude::*,
    routes::{ApiResponse, ErrorResponseBody},
    AppState,
};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Extension, Json,
};
use sea_orm::{ActiveValue::NotSet, ColumnTrait, Condition, EntityTrait, QueryFilter, Set};
use std::sync::Arc;

#[utoipa::path(put, path = "/api/user/favorite/:id", responses(
    (status = 200, description = "Add favorite successful."),
    (status = 400, description = "Bad request.", body = ErrorResponse),
    (status = 401, description = "Unauthorized.", body = ErrorResponse),
))]
pub async fn put_favorite(
    State(data): State<Arc<AppState>>,
    Extension(user): Extension<users::Model>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, (StatusCode, Json<ApiResponse<ErrorResponseBody>>)> {
    let title = Titles::find_by_id(id)
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
                String::from("Invalid title id."),
            )
        })?;

    let _ = Favorites::find()
        .filter(
            Condition::all()
                .add(favorites::Column::TitleId.contains(&title.id))
                .add(favorites::Column::UserId.contains(&user.id)),
        )
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
                String::from("Title already favorited."),
            )
        })?;

    let active_favorite = favorites::ActiveModel {
        id: NotSet,
        title_id: Set(title.id),
        user_id: Set(user.id),
    };

    Favorites::insert(active_favorite)
        .exec(&data.db)
        .await
        .map_err(|e| {
            build_err_resp(
                StatusCode::INTERNAL_SERVER_ERROR,
                String::from("An internal server error has occurred."),
                format!("Database error: {}", e),
            )
        })?;

    Ok(StatusCode::OK)
}

#[utoipa::path(put, path = "/api/user/bookmark/:id", responses(
    (status = 200, description = "Add bookmark successful."),
    (status = 400, description = "Bad request.", body = ErrorResponse),
    (status = 401, description = "Unauthorized.", body = ErrorResponse),
))]
pub async fn put_bookmark(
    State(data): State<Arc<AppState>>,
    Extension(user): Extension<users::Model>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, (StatusCode, Json<ApiResponse<ErrorResponseBody>>)> {
    let title = Titles::find_by_id(id)
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
                String::from("Invalid title id."),
            )
        })?;

    let _ = Bookmarks::find()
        .filter(
            Condition::all()
                .add(bookmarks::Column::TitleId.contains(&title.id))
                .add(bookmarks::Column::UserId.contains(&user.id)),
        )
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
                String::from("Title already bookmarked."),
            )
        })?;

    let active_bookmark = favorites::ActiveModel {
        id: NotSet,
        title_id: Set(title.id),
        user_id: Set(user.id),
    };

    Favorites::insert(active_bookmark)
        .exec(&data.db)
        .await
        .map_err(|e| {
            build_err_resp(
                StatusCode::INTERNAL_SERVER_ERROR,
                String::from("An internal server error has occurred."),
                format!("Database error: {}", e),
            )
        })?;

    Ok(StatusCode::OK)
}

#[utoipa::path(delete, path = "/api/user/favorite/:id", responses(
    (status = 200, description = "Delete favorite successful."),
    (status = 400, description = "Bad request.", body = ErrorResponse),
    (status = 401, description = "Unauthorized.", body = ErrorResponse),
))]
pub async fn delete_favorite(
    State(data): State<Arc<AppState>>,
    Extension(user): Extension<users::Model>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, (StatusCode, Json<ApiResponse<ErrorResponseBody>>)> {
    let title = Titles::find_by_id(id)
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
                String::from("Invalid title id."),
            )
        })?;

    Favorites::delete_many()
        .filter(
            Condition::all()
                .add(favorites::Column::TitleId.contains(&title.id))
                .add(favorites::Column::UserId.contains(&user.id)),
        )
        .exec(&data.db)
        .await
        .map_err(|e| {
            build_err_resp(
                StatusCode::INTERNAL_SERVER_ERROR,
                String::from("An internal server error has occurred."),
                format!("Database error: {}", e),
            )
        })?;

    Ok(StatusCode::OK)
}

#[utoipa::path(delete, path = "/api/user/bookmark/:id", responses(
    (status = 200, description = "Delete bookmark successful."),
    (status = 400, description = "Bad request.", body = ErrorResponse),
    (status = 401, description = "Unauthorized.", body = ErrorResponse),
))]
pub async fn delete_bookmark(
    State(data): State<Arc<AppState>>,
    Extension(user): Extension<users::Model>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, (StatusCode, Json<ApiResponse<ErrorResponseBody>>)> {
    let title = Titles::find_by_id(id)
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
                String::from("Invalid title id."),
            )
        })?;

    Bookmarks::delete_many()
        .filter(
            Condition::all()
                .add(bookmarks::Column::TitleId.contains(&title.id))
                .add(bookmarks::Column::UserId.contains(&user.id)),
        )
        .exec(&data.db)
        .await
        .map_err(|e| {
            build_err_resp(
                StatusCode::INTERNAL_SERVER_ERROR,
                String::from("An internal server error has occurred."),
                format!("Database error: {}", e),
            )
        })?;

    Ok(StatusCode::OK)
}
