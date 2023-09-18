use serde::{Deserialize, Serialize};
use utoipa::{OpenApi, ToSchema};

use self::{
    auth::{AuthRequest, AuthResponseBody},
    status::{StatusRequest, StatusResponseBody},
};

pub mod auth;
pub mod status;

#[derive(Deserialize, Serialize, ToSchema)]
#[aliases(AuthResponse = ApiResponse<AuthResponseBody>, StatusResponse = ApiResponse<StatusResponseBody>)]
pub struct ApiResponse<T> {
    /// A description of the response status.
    pub description: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(flatten)]
    pub body: Option<T>,
}

#[derive(OpenApi)]
#[openapi(
    paths(auth::auth, status::get_status, status::post_status,),
    components(schemas(
        AuthResponse,
        AuthResponseBody,
        AuthRequest,
        StatusResponse,
        StatusResponseBody,
        StatusRequest
    ))
)]
pub struct ApiDoc;
