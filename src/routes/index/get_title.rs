use super::super::{ApiResponse, ErrorResponseBody};
use crate::{
    models::{
        bookmarks, favorites,
        prelude::{Pages, Thumbnails, Titles},
        progresses, titles_tags, users,
    },
    utils::{build_err_resp, build_resp},
    AppState,
};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Extension, Json,
};
use sea_orm::*;
use serde::Serialize;
use std::sync::Arc;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Serialize, ToSchema, Debug)]
pub struct ResponsePage {
    pub id: String,
    pub title_id: String,
    pub hash: String,
    pub width: u32,
    pub height: u32,
    pub description: Option<String>,
}

#[derive(Serialize, ToSchema, Debug)]
pub struct ResponseThumbnail {
    pub id: String,
    pub hash: String,
    pub width: u32,
    pub height: u32,
}

#[derive(Serialize, ToSchema)]
pub struct TitleResponseBody {
    pub category_id: String,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub release_date: Option<String>,
    pub thumbnail: ResponseThumbnail,
    pub tag_ids: Vec<i32>,
    pub pages: Vec<ResponsePage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub favorites: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_favorite: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_bookmark: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page_read: Option<u32>,
    pub date_added: String,
    pub date_updated: String,
}

/// Get everything about a title.
#[utoipa::path(get, path = "/api/index/title/{title_id}", responses(
    (status = 200, description = "Fetch title successful.", body = TitleResponse),
    (status = 204, description = "Fetch title successful, but one was not found.", body = TitleResponse),
    (status = 500, description = "Internal server error.", body = ErrorResponse)
))]
pub async fn get_title(
    State(data): State<Arc<AppState>>,
    Path(title_id): Path<Uuid>,
    Extension(user): Extension<crate::models::users::Model>,
) -> Result<impl IntoResponse, (StatusCode, Json<ApiResponse<ErrorResponseBody>>)> {
    let title = Titles::find_by_id(title_id)
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
                StatusCode::NO_CONTENT,
                String::from("Bad request."),
                String::from("No title found."),
            )
        })?;

    let thumbnail = Thumbnails::find_by_id(&title.thumbnail_id)
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
                StatusCode::NO_CONTENT,
                String::from("Bad request."),
                String::from("No thumbnail found."),
            )
        })?;

    let pages = Pages::find_by_id(&title.id)
        .all(&data.db)
        .await
        .map_err(|e| {
            build_err_resp(
                StatusCode::INTERNAL_SERVER_ERROR,
                String::from("An internal server error has occurred."),
                format!("Database error: {}", e),
            )
        })?;

    let is_favorite = favorites::Entity::find()
        .having(favorites::Column::UserId.eq(&user.id))
        .having(favorites::Column::TitleId.eq(&title.id))
        .one(&data.db)
        .await
        .map_err(|e| {
            build_err_resp(
                StatusCode::INTERNAL_SERVER_ERROR,
                String::from("An internal server error has occurred."),
                format!("Database error: {}", e),
            )
        })?
        .map(|_| true);

    let is_bookmark = users::Entity::find()
        .having(bookmarks::Column::UserId.eq(&user.id))
        .having(bookmarks::Column::TitleId.eq(&title.id))
        .one(&data.db)
        .await
        .map_err(|e| {
            build_err_resp(
                StatusCode::INTERNAL_SERVER_ERROR,
                String::from("An internal server error has occurred."),
                format!("Database error: {}", e),
            )
        })?
        .map(|_| true);

    let page_read = progresses::Entity::find()
        .having(progresses::Column::UserId.eq(&user.id))
        .having(progresses::Column::TitleId.eq(&title.id))
        .one(&data.db)
        .await
        .map_err(|e| {
            build_err_resp(
                StatusCode::INTERNAL_SERVER_ERROR,
                String::from("An internal server error has occurred."),
                format!("Database error: {}", e),
            )
        })?
        .map(|p| p.page);

    let favorites = match favorites::Entity::find()
        .having(favorites::Column::TitleId.eq(&title.id))
        .count(&data.db)
        .await
        .map_err(|e| {
            build_err_resp(
                StatusCode::INTERNAL_SERVER_ERROR,
                String::from("An internal server error has occurred."),
                format!("Database error: {}", e),
            )
        })? {
        0 => None,
        n => Some(n),
    };

    let tag_ids = titles_tags::Entity::find()
        .having(titles_tags::Column::TitleId.eq(&title.id))
        .all(&data.db)
        .await
        .map_err(|e| {
            build_err_resp(
                StatusCode::INTERNAL_SERVER_ERROR,
                String::from("An internal server error has occurred."),
                format!("Database error: {}", e),
            )
        })?
        .iter()
        .map(|tag| tag.tag_id)
        .collect::<Vec<_>>();

    let title = TitleResponseBody {
        category_id: title.category_id,
        title: title.title,
        author: title.author,
        description: title.description,
        release_date: title.release_date,
        thumbnail: ResponseThumbnail {
            id: thumbnail.id,
            hash: thumbnail.hash,
            width: thumbnail.width,
            height: thumbnail.height,
        },
        tag_ids,
        pages: pages
            .iter()
            .map(|page| ResponsePage {
                id: page.id.clone(),
                title_id: page.title_id.clone(),
                hash: page.hash.clone(),
                width: page.width,
                height: page.height,
                description: page.description.clone(),
            })
            .collect::<Vec<_>>(),
        favorites,
        is_favorite,
        is_bookmark,
        page_read,
        date_added: title.date_added,
        date_updated: title.date_updated,
    };

    Ok(build_resp(
        StatusCode::OK,
        String::from("Fetch title successful."),
        title,
    ))
}
