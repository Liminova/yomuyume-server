use axum::{extract::Query, response::IntoResponse, Json};
use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

#[derive(Deserialize, Serialize, ToSchema)]
pub struct StatusResponse {
    pub description: String,
    pub server_time: DateTime<Local>,
    pub version: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub echo: Option<String>,
}

#[derive(Deserialize, IntoParams)]
pub struct StatusQuery {
    pub echo: Option<String>,
}

#[utoipa::path(get, path = "/api/status", params(StatusQuery), responses((status = 200, description = "Status check successful.", body = StatusResponse)))]
pub async fn get_status(query: Query<StatusQuery>) -> impl IntoResponse {
    let echo = query.echo.clone();
    Json(StatusResponse {
        description: String::from("Status check successful."),
        server_time: chrono::Local::now(),
        version: String::from("0.1.0"),
        echo,
    })
}

#[utoipa::path(post, path = "/api/status", params(StatusQuery), responses((status = 200, description = "Status check successful.", body = StatusResponse)))]
pub async fn post_status(query: Query<StatusQuery>) -> impl IntoResponse {
    let echo = query.echo.clone();
    Json(StatusResponse {
        description: String::from("Status check successful."),
        server_time: chrono::Local::now(),
        version: String::from("0.1.0"),
        echo,
    })
}
