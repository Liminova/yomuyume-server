use crate::{
    models::{bookmarks, favorites, prelude::Titles, titles, users},
    routes::{ApiResponse, ErrorResponseBody},
    utils::build_err_resp,
    AppState,
};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Extension, Json,
};
use sea_orm::{ColumnTrait, Condition, EntityTrait, QueryFilter};
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

    let condition = Condition::all()
        .add(favorites::Column::TitleId.contains(&title.id))
        .add(favorites::Column::UserId.contains(&user.id));

    let _ = favorites::Entity::find()
        .filter(condition)
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

    let active_title: titles::ActiveModel = title.into();
    let active_user: users::ActiveModel = user.into();

    let active_favorite = favorites::ActiveModel {
        title_id: active_title.id,
        user_id: active_user.id,
        ..Default::default()
    };

    favorites::Entity::insert(active_favorite)
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

    let condition = Condition::all()
        .add(bookmarks::Column::TitleId.contains(&title.id))
        .add(bookmarks::Column::UserId.contains(&user.id));

    let _ = bookmarks::Entity::find()
        .filter(condition)
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

    let active_title: titles::ActiveModel = title.into();
    let active_user: users::ActiveModel = user.into();

    let active_bookmark = favorites::ActiveModel {
        title_id: active_title.id,
        user_id: active_user.id,
        ..Default::default()
    };

    favorites::Entity::insert(active_bookmark)
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

    let condition = Condition::all()
        .add(favorites::Column::TitleId.contains(&title.id))
        .add(favorites::Column::UserId.contains(&user.id));

    favorites::Entity::delete_many()
        .filter(condition)
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

    let condition = Condition::all()
        .add(bookmarks::Column::TitleId.contains(&title.id))
        .add(bookmarks::Column::UserId.contains(&user.id));

    bookmarks::Entity::delete_many()
        .filter(condition)
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
