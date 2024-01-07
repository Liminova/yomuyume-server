use super::{build_err_resp, build_resp};
use crate::{
    models::prelude::*,
    routes::{ApiResponse, ErrorResponseBody},
    AppState,
};
use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use sea_orm::EntityTrait;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use utoipa::ToSchema;

#[derive(Serialize, Deserialize, Debug, ToSchema)]
pub struct TagsMapResponseBody {
    pub tags: Vec<(i32, String)>,
}

#[utoipa::path(get, path = "/api/utils/tags", responses(
    (status = 200, description = "Tags map.", body = TagsMapResponse),
    (status = 500, description = "Internal server error.", body = ErrorResponse),
))]
pub async fn get_tags(
    State(data): State<Arc<AppState>>,
) -> Result<impl IntoResponse, (StatusCode, Json<ApiResponse<ErrorResponseBody>>)> {
    let tags = Tags::find().all(&data.db).await.map_err(|_| {
        build_err_resp(
            StatusCode::INTERNAL_SERVER_ERROR,
            String::from("An internal server error has occurred."),
            String::from("Failed to get tags."),
        )
    })?;

    let mut tag_map = Vec::new();
    for tag in tags {
        let tag = Tags::find_by_id(tag.id)
            .one(&data.db)
            .await
            .map_err(|_| {
                build_err_resp(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    String::from("An internal server error has occurred."),
                    String::from("Failed to get categories."),
                )
            })?
            .ok_or_else(|| {
                build_err_resp(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    String::from("An internal server error has occurred."),
                    String::from("Failed to get categories."),
                )
            })?;
        tag_map.push((tag.id, tag.name));
    }

    Ok(build_resp(
        StatusCode::OK,
        String::from("Getting tags map succeeded."),
        TagsMapResponseBody { tags: tag_map },
    ))
}
