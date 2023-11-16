use serde::{Deserialize, Serialize};
use utoipa::{OpenApi, ToSchema};

use self::{
    auth::{LoginRequest, LoginResponseBody, RegisterRequest, RegisterResponseBody},
    status::{StatusRequest, StatusResponseBody},
};

pub mod auth;
pub mod categories;
pub mod status;

#[derive(Deserialize, Serialize, ToSchema, Debug)]
pub struct ErrorResponseBody {
    pub message: String,
}

#[derive(Deserialize, Serialize, ToSchema, Debug)]
#[aliases(
    ErrorResponse = ApiResponse<ErrorResponseBody>,
    LoginResponse = ApiResponse<LoginResponseBody>,
    RegisterResponse = ApiResponse<RegisterResponseBody>,
    StatusResponse = ApiResponse<StatusResponseBody>
)]
pub struct ApiResponse<T> {
    /// A description of the response status.
    pub description: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(flatten)]
    pub body: Option<T>,
}

#[derive(OpenApi)]
#[openapi(
    paths(
        auth::post_login,
        auth::post_register,
        auth::get_logout,
        status::get_status,
        status::post_status
    ),
    components(schemas(
        LoginResponse,
        LoginResponseBody,
        LoginRequest,
        RegisterResponse,
        RegisterResponseBody,
        RegisterRequest,
        StatusResponse,
        StatusResponseBody,
        StatusRequest
    ))
)]
pub struct ApiDoc;
