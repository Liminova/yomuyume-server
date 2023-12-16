use std::sync::Arc;

use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use sea_orm::{ColumnTrait, Condition, DatabaseConnection, EntityTrait, QueryFilter};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::models::{progresses, titles, titles_tags};
use crate::{
    utils::build_resp::{build_err_resp, build_resp},
    AppState,
};

use super::super::{ApiResponse, ErrorResponseBody};
use crate::utils::find_title_info::*;

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct FilterRequest {
    /// Keywords to search for (search in title, description, author, tags)
    keywords: Vec<String>,
    /// Categories to filter by
    category_ids: Vec<String>,
    /// Tags to filter by
    tag_ids: Vec<i32>,
    /// Only include listed fields in response
    fields: Vec<String>,
    /// Maximum number of results to return
    limit: u32,
}

impl FilterRequest {
    pub fn set_str(&self, field: &str, value: String) -> String {
        if self.fields.is_empty() || self.fields.contains(&String::from(field)) {
            return value;
        }
        String::from("")
    }
    pub fn set_option_str(&self, field: &str, value: Option<String>) -> String {
        match value {
            Some(value) => {
                if self.fields.is_empty() || self.fields.contains(&String::from(field)) {
                    return value;
                }
                String::from("")
            }
            None => String::from(""),
        }
    }
    pub fn set_vec_i32(&self, field: &str, value: Vec<i32>) -> Vec<i32> {
        if self.fields.is_empty() || self.fields.contains(&String::from(field)) {
            return value;
        }
        vec![]
    }
    pub fn set_u32(&self, field: &str, value: u32) -> u32 {
        if self.fields.is_empty() || self.fields.contains(&String::from(field)) {
            return value;
        }
        0
    }
}

#[derive(Serialize, ToSchema)]
pub struct FilterTitleResponseBody {
    id: String,
    title: String,
    author: String,
    categories_id: String,
    thumbnail_id: String,
    release_date: Option<String>,
    date_added: String,
    date_updated: String,
    favorite_count: Option<u32>,
    page_count: u32,
    page_read: Option<u32>,
}

#[derive(Serialize, ToSchema)]
pub struct FilterResponseBody {
    pub data: Vec<FilterTitleResponseBody>,
}

#[utoipa::path(post, path = "/api/index/filter", responses(
    (status = 200, description = "Fetch all items successful.", body = FilterResponse),
    (status = 204, description = "Fetch all items successful, but none were found.", body = FilterResponse),
    (status = 500, description = "Internal server error.", body = ErrorResponse)
))]
pub async fn post_filter(
    State(data): State<Arc<AppState>>,
    Extension(user): Extension<crate::models::users::Model>,
    Json(query): Json<FilterRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<ApiResponse<ErrorResponseBody>>)> {
    let keywords = query.keywords.to_vec();
    let category_ids = query.category_ids.to_vec();
    let tag_ids: Vec<i32> = query.tag_ids.to_vec();

    if keywords.is_empty() && category_ids.is_empty() && tag_ids.is_empty() {
        return Ok(build_resp(
            StatusCode::NO_CONTENT,
            String::from("Fetching all items successful, but none were found."),
            Some(FilterResponseBody { data: vec![] }),
        ));
    }

    let mut condition = Condition::any();

    if !category_ids.is_empty() {
        for category_id in category_ids {
            condition = condition.add(titles::Column::CategoryId.eq(category_id));
        }
    }

    if !keywords.is_empty() {
        for keyword in keywords {
            condition = condition
                .add(titles::Column::Title.contains(keyword.clone().to_lowercase()))
                .add(titles::Column::Author.contains(keyword.clone().to_lowercase()))
                .add(titles::Column::Description.contains(keyword.clone().to_lowercase()));
        }
    }

    if !tag_ids.is_empty() {
        for tag_id in tag_ids {
            let titles_tags = titles_tags::Entity::find()
                .filter(titles_tags::Column::TagId.eq(tag_id))
                .all(&data.db)
                .await
                .map_err(|e| {
                    build_err_resp(
                        StatusCode::INTERNAL_SERVER_ERROR,
                        String::from("An internal server error has occurred."),
                        format!("Database error: {}", e),
                    )
                })?;

            for titles_tag in titles_tags {
                condition = condition.add(titles::Column::Id.eq(titles_tag.title_id));
            }
        }
    }

    let found_titles = titles::Entity::find()
        .filter(condition)
        .all(&data.db)
        .await
        .map_err(|e| {
            build_err_resp(
                StatusCode::INTERNAL_SERVER_ERROR,
                String::from("An internal server error has occurred."),
                format!("Database error: {}", e),
            )
        })?;

    let mut resp_data: Vec<FilterTitleResponseBody> = vec![];

    for title in found_titles {
        let tag_ids = find_title_tag_ids(&data.db, title.id.clone()).await;
        let page_count = find_page_count(&data.db, title.id.clone()).await;
        let favorite = find_favorite_count(&data.db, title.id.clone()).await;
        let page_read = find_page_read(&data.db, title.id.clone()).await;
        resp_data.push(FilterTitleResponseBody {
            id: query.set_str("id", title.id),
            title: query.set_str("title", title.title),
            author: query.set_option_str("author", title.author),
            categories_id: query.set_str("categories_id", title.category_id),
            thumbnail_id: query.set_str("thumbnail_id", title.thumbnail_id),
            release_date: query.set_option_str("release_date", title.release_date),
            date_added: query.set_str("date_added", title.date_added),
            date_updated: query.set_str("date_updated", title.date_updated),
            tag_ids: query.set_vec_i32("tag_ids", tag_ids),
            favorite: query.set_u32("favorite", favorite),
            page_count: query.set_u32("page_count", page_count),
            page_read: query.set_u32("page_read", page_read),
        });
    }

    let resp = match resp_data.is_empty() {
        true => build_resp(
            StatusCode::NO_CONTENT,
            String::from("Fetching all items successful, but none were found."),
            Some(FilterResponseBody { data: vec![] }),
        ),
        false => build_resp(
            StatusCode::OK,
            String::from("Fetching all items successful."),
            Some(FilterResponseBody { data: resp_data }),
        ),
    };

    Ok(resp)
}
