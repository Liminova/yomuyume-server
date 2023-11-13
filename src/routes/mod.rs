use serde::{Deserialize, Serialize};
use utoipa::{OpenApi, ToSchema};

use self::status::{StatusRequest, StatusResponseBody};

pub mod auth;
pub mod status;

#[derive(Deserialize, Serialize, ToSchema)]
#[aliases(StatusResponse = ApiResponse<StatusResponseBody>)]
pub struct ApiResponse<T> {
    /// A description of the response status.
    pub description: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(flatten)]
    pub body: Option<T>,
}

#[derive(OpenApi)]
#[openapi(
    paths(status::get_status, status::post_status),
    components(schemas(StatusResponse, StatusResponseBody, StatusRequest))
)]
pub struct ApiDoc;
