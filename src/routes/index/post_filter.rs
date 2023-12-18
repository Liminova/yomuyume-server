use super::super::{ApiResponse, ErrorResponseBody};
use crate::models::{titles, titles_tags};
use crate::{
    models::users,
    utils::{build_err_resp, build_resp, find_favorite_count, find_page_count, find_page_read},
    AppState,
};
use axum::{extract::State, http::StatusCode, response::IntoResponse, Extension, Json};
use sea_orm::{ColumnTrait, Condition, EntityTrait, QueryFilter, QuerySelect, QueryTrait};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use utoipa::ToSchema;

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct FilterRequest {
    /// Keywords to search for (search in title, description, author, tags)
    keywords: Option<Vec<String>>,
    /// Categories to filter by
    category_ids: Option<Vec<String>>,
    /// Tags to filter by
    tag_ids: Option<Vec<i32>>,
    /// Maximum number of results to return
    limit: Option<u32>,
}

#[derive(Serialize, ToSchema)]
pub struct FilterTitleResponseBody {
    id: String,
    title: String,
    author: Option<String>,
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

/// Filtering titles by various parameters.
///
/// And also sorting them by various options.
#[utoipa::path(post, path = "/api/index/filter", responses(
    (status = 200, description = "Fetch all items successful.", body = FilterResponse),
    (status = 204, description = "Fetch all items successful, but none were found.", body = FilterResponse),
    (status = 500, description = "Internal server error.", body = ErrorResponse)
))]
pub async fn post_filter(
    State(data): State<Arc<AppState>>,
    Extension(user): Extension<users::Model>,
    Json(query): Json<FilterRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<ApiResponse<ErrorResponseBody>>)> {
    let keywords = query.keywords;
    let category_ids = query.category_ids;
    let tag_ids = query.tag_ids;
    let limit = query.limit;

    if keywords.is_none() && category_ids.is_none() && tag_ids.is_none() {
        return Ok(build_resp(
            StatusCode::NO_CONTENT,
            String::from("Fetching all items successful, but none were found."),
            Some(FilterResponseBody { data: vec![] }),
        ));
    }

    let mut condition = Condition::any();

    if let Some(category_ids) = category_ids {
        for category_id in category_ids {
            condition = condition.add(titles::Column::CategoryId.eq(category_id));
        }
    }

    if let Some(keywords) = keywords {
        for keyword in keywords {
            condition = condition
                .add(titles::Column::Title.contains(keyword.to_lowercase()))
                .add(titles::Column::Author.contains(keyword.to_lowercase()))
                .add(titles::Column::Description.contains(keyword.to_lowercase()));
        }
    }

    if let Some(tag_ids) = tag_ids {
        for tag_id in tag_ids {
            let title_tag_has_tag_id = titles_tags::Entity::find()
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

            for entity in title_tag_has_tag_id {
                condition = condition.add(titles::Column::Id.eq(entity.title_id));
            }
        }
    }

    let found_titles = titles::Entity::find()
        .apply_if(limit.map(|limit| limit as u64), QuerySelect::limit)
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
        let page_count = find_page_count(&data.db, &title.id).await;
        let favorite_count = find_favorite_count(&data.db, &title.id).await;
        let page_read = find_page_read(&data.db, &title.id, &user.id).await;
        resp_data.push(FilterTitleResponseBody {
            id: title.id,
            title: title.title,
            author: title.author,
            categories_id: title.category_id,
            thumbnail_id: title.thumbnail_id,
            release_date: title.release_date,
            date_added: title.date_added,
            date_updated: title.date_updated,
            favorite_count,
            page_count,
            page_read,
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
