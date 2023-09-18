use serde::{Deserialize, Serialize};
use utoipa::{OpenApi, ToSchema};

use self::status::{StatusRequest, StatusResponseBody};

pub mod status;

#[derive(Deserialize, Serialize, ToSchema)]
#[aliases(StatusResponse = ApiResponse<StatusResponseBody>)]
pub struct ApiResponse<T> {
    /// A description of the response status.
    pub description: String,

    #[serde(flatten)]
    pub body: Option<T>,
}

#[derive(OpenApi)]
#[openapi(
    paths(status::get_status, status::post_status),
    components(schemas(StatusResponse, StatusResponseBody, StatusRequest))
)]
pub struct ApiDoc;
