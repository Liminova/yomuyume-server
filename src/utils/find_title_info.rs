use axum::http::StatusCode;
use sea_orm::{ColumnTrait, Condition, DatabaseConnection, EntityTrait, QueryFilter};

use crate::models::progresses;

use super::build_resp::build_err_resp;

pub async fn find_page_count(db: &DatabaseConnection, title_id: &str) -> u32 {
    let pages = crate::models::pages::Entity::find()
        .filter(crate::models::pages::Column::TitleId.contains(title_id))
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
    let mut condition = Condition::all();

    condition = condition.add(progresses::Column::TitleId.eq(title_id));
    condition = condition.add(progresses::Column::UserId.eq(user_id));

    let progresses = progresses::Entity::find()
        .filter(condition)
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
    let favorites = crate::models::favorites::Entity::find()
        .filter(crate::models::favorites::Column::TitleId.contains(title_id))
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
