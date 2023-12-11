use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use sea_orm::*;
use serde::Serialize;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::{
    models::{prelude::Titles, titles::Model as Title},
    utils::build_resp::{build_err_resp, build_resp},
    AppState,
};

use super::{ApiResponse, ErrorResponseBody};

#[derive(Serialize, ToSchema)]
pub struct TitlesResponseBody {
    /// A list of all fetched titles.
    pub data: Vec<Title>,
}

#[derive(Serialize, ToSchema)]
pub struct TitleResponseBody {
    /// The requested title.
    pub data: Title,
}

#[utoipa::path(get, path = "/api/titles", responses(
    (status = 200, description = "Fetch all titles successful.", body = TitlesResponse),
    (status = 500, description = "Internal server error.", body = ErrorResponse)
))]
pub async fn get_titles(
    State(data): State<Arc<AppState>>,
) -> Result<impl IntoResponse, (StatusCode, Json<ApiResponse<ErrorResponseBody>>)> {
    let titles: Vec<Title> = Titles::find().all(&data.db).await.map_err(|e| {
        build_err_resp(
            StatusCode::INTERNAL_SERVER_ERROR,
            String::from("An internal server error has occurred."),
            format!("Database error: {}", e),
        )
    })?;

    Ok(build_resp(
        StatusCode::OK,
        String::from("Fetching all titles successful."),
        Some(TitlesResponseBody { data: titles }),
    ))
}

#[utoipa::path(get, path = "/api/title/{title_id}", responses(
    (status = 200, description = "Fetch title successful.", body = TitleResponse),
    (status = 204, description = "Fetch title successful, but one was not found.", body = TitleResponse),
    (status = 500, description = "Internal server error.", body = ErrorResponse)
))]
pub async fn get_title(
    State(data): State<Arc<AppState>>,
    Path(title_id): Path<Uuid>,
) -> Result<impl IntoResponse, (StatusCode, Json<ApiResponse<ErrorResponseBody>>)> {
    let title: Option<Title> = Titles::find_by_id(title_id)
        .one(&data.db)
        .await
        .map_err(|e| {
            build_err_resp(
                StatusCode::INTERNAL_SERVER_ERROR,
                String::from("An internal server error has occurred."),
                format!("Database error: {}", e),
            )
        })?;

    let resp = match title {
        Some(t) => build_resp(
            StatusCode::OK,
            format!("Fetch title with id {} successful.", title_id),
            Some(TitleResponseBody { data: t }),
        ),
        None => build_resp(
            StatusCode::NO_CONTENT,
            format!(
                "The server could not find any titles matching the id {}.",
                title_id
            ),
            None::<TitleResponseBody>,
        ),
    };

    Ok(resp)
}
