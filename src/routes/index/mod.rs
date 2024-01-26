mod get_categories;
mod get_title;
mod post_filter;

use crate::{
    constants::{blurhash_dimension_cap, ratio_percision},
    models::prelude::*,
    routes::build_err_resp,
};
use axum::http::StatusCode;

use sea_orm::{ColumnTrait, Condition, DatabaseConnection, EntityTrait, QueryFilter};

pub use get_categories::*;
pub use get_title::*;
pub use post_filter::*;

pub async fn find_page_count(db: &DatabaseConnection, title_id: &str) -> i64 {
    let pages = Pages::find()
        .filter(pages::Column::TitleId.contains(title_id))
        .all(db)
        .await
        .map_err(|e| {
            build_err_resp(
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Database error: {}", e),
            )
        })
        .unwrap_or(vec![]);

    match pages.is_empty() {
        true => 0,
        false => pages.len() as i64,
    }
}

pub async fn find_page_read(db: &DatabaseConnection, title_id: &str, user_id: &str) -> Option<i64> {
    let progresses = Progresses::find()
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
                format!("Database error: {}", e),
            )
        })
        .unwrap_or_default();

    match progresses {
        Some(progress) => Some(progress.page),
        None => None,
    }
}

pub async fn find_favorite_count(db: &DatabaseConnection, title_id: &str) -> Option<i64> {
    let favorites = Favorites::find()
        .filter(favorites::Column::TitleId.contains(title_id))
        .all(db)
        .await
        .map_err(|e| {
            build_err_resp(
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Database error: {}", e),
            )
        })
        .unwrap_or(vec![]);

    match favorites.is_empty() {
        true => None,
        false => Some(favorites.len() as i64),
    }
}

fn calculate_dimension(ratio: u32) -> (u32, u32) {
    let max_dimension = blurhash_dimension_cap();
    let ratio = ratio as f32 / ratio_percision() as f32;

    let (width, height) = if ratio >= 1.0 {
        (max_dimension, max_dimension / ratio) // Landscape
    } else {
        (max_dimension * ratio, max_dimension) // Portrait
    };

    (width as u32, height as u32)
}
