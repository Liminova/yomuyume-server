mod get_categories;
mod get_title;
mod post_filter;

use super::{build_err_resp, build_resp};
use crate::models::{favorites, pages, progresses};
use axum::http::StatusCode;

pub use get_categories::{get_categories, CategoriesResponseBody};
pub use get_title::{get_title, TitleResponseBody};
pub use post_filter::{post_filter, FilterRequest, FilterResponseBody};

pub use get_categories::__path_get_categories;
pub use get_title::__path_get_title;
pub use post_filter::__path_post_filter;
use sea_orm::{ColumnTrait, Condition, DatabaseConnection, EntityTrait, QueryFilter};

pub async fn find_page_count(db: &DatabaseConnection, title_id: &str) -> u32 {
    let pages = pages::Entity::find()
        .filter(pages::Column::TitleId.contains(title_id))
        .all(db)
        .await
        .map_err(|e| {
            build_err_resp(
                StatusCode::INTERNAL_SERVER_ERROR,
                String::from("An internal server error has occurred."),
                format!("Database error: {}", e),
            )
        })
        .unwrap_or(vec![]);

    match pages.is_empty() {
        true => 0,
        false => pages.len() as u32,
    }
}

pub async fn find_page_read(db: &DatabaseConnection, title_id: &str, user_id: &str) -> Option<u32> {
    let progresses = progresses::Entity::find()
        .filter(
            Condition::all()
                .add(progresses::Column::TitleId.eq(title_id))
                .add(progresses::Column::UserId.eq(user_id)),
        )
        .one(db)
        .await
        .map_err(|e| {
            build_err_resp(
                StatusCode::INTERNAL_SERVER_ERROR,
                String::from("An internal server error has occurred."),
                format!("Database error: {}", e),
            )
        })
        .unwrap_or_default();

    match progresses {
        Some(progress) => Some(progress.page),
        None => None,
    }
}

pub async fn find_favorite_count(db: &DatabaseConnection, title_id: &str) -> Option<u32> {
    let favorites = favorites::Entity::find()
        .filter(favorites::Column::TitleId.contains(title_id))
        .all(db)
        .await
        .map_err(|e| {
            build_err_resp(
                StatusCode::INTERNAL_SERVER_ERROR,
                String::from("An internal server error has occurred."),
                format!("Database error: {}", e),
            )
        })
        .unwrap_or(vec![]);

    match favorites.is_empty() {
        true => None,
        false => Some(favorites.len() as u32),
    }
}
