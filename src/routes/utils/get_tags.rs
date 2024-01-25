use crate::{
    models::prelude::*,
    routes::{build_err_resp, build_resp, ApiResponse, ErrorResponseBody},
    AppState,
};
use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use sea_orm::EntityTrait;
use serde::Serialize;
use std::sync::Arc;
use utoipa::ToSchema;

#[derive(Serialize, Debug, ToSchema)]
pub struct TagsMapResponseBody {
    pub tags: Vec<(u32, String)>,
}

#[utoipa::path(get, path = "/api/utils/tags", responses(
    (status = 200, description = "Tags map.", body = TagsMapResponse),
    (status = 500, description = "Internal server error.", body = ErrorResponse),
))]
pub async fn get_tags(
    State(data): State<Arc<AppState>>,
) -> Result<impl IntoResponse, (StatusCode, Json<ApiResponse<ErrorResponseBody>>)> {
    let tags = Tags::find()
        .all(&data.db)
        .await
        .map_err(|_| build_err_resp(StatusCode::INTERNAL_SERVER_ERROR, "Failed to get tags."))?;

    let tag_map = tags
        .into_iter()
        .map(|tag| (tag.id, tag.name))
        .collect::<Vec<(u32, String)>>();

    Ok(build_resp(
        StatusCode::OK,
        TagsMapResponseBody { tags: tag_map },
    ))
}
