use axum::http::StatusCode;
use axum::{extract::Query, response::IntoResponse, Json};
use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

use crate::constants::version::get_version;
use crate::ApiResponse;

#[derive(Deserialize, Serialize, ToSchema)]
pub struct StatusResponseBody {
    /// Current local server time.
    pub server_time: DateTime<Local>,
    /// Current yomuyume version.
    pub version: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    /// Your test string.
    pub echo: Option<String>,
}

#[derive(Deserialize, IntoParams, ToSchema)]
pub struct StatusRequest {
    ///  A test string to test your request body.
    pub echo: Option<String>,
}

#[utoipa::path(get, path = "/api/status", params(StatusRequest), responses((status = 200, description = "Status check successful.", body = StatusResponse)))]
pub async fn get_status(query: Query<StatusRequest>) -> impl IntoResponse {
    let echo = query.echo.clone();
    let version = get_version();
    Json(ApiResponse {
        description: String::from("Status check successful."),
        body: Some(StatusResponseBody {
            server_time: chrono::Local::now(),
            version,
            echo,
        }),
    })
}

#[utoipa::path(post, path = "/api/status", responses((status = 200, description = "Status check successful.", body = StatusResponse)))]
pub async fn post_status(query: Option<Json<StatusRequest>>) -> impl IntoResponse {
    if let Some(query) = query {
        let echo = query.echo.clone();
        let version = get_version();
        (
            StatusCode::OK,
            Json(ApiResponse {
                description: String::from("Status check successful."),
                body: Some(StatusResponseBody {
                    server_time: chrono::Local::now(),
                    version,
                    echo,
                }),
            }),
        )
    } else {
        let version = get_version();
        (
            StatusCode::OK,
            Json(ApiResponse {
                description: String::from("Status check successful."),
                body: Some(StatusResponseBody {
                    server_time: chrono::Local::now(),
                    version,
                    echo: None,
                }),
            }),
        )
    }
}
