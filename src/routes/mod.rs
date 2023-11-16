use serde::{Deserialize, Serialize};
use utoipa::{OpenApi, ToSchema};

use crate::models::category::Category;

use self::{
    auth::{LoginRequest, LoginResponseBody, RegisterRequest, RegisterResponseBody},
    categories::{CategoriesResponseBody, CategoryResponseBody},
    status::{StatusRequest, StatusResponseBody},
};

pub mod auth;
pub mod categories;
pub mod status;

#[derive(Deserialize, Serialize, ToSchema, Debug)]
pub struct ErrorResponseBody {
    /// The error message.
    pub message: String,
}

#[derive(Deserialize, Serialize, ToSchema, Debug)]
#[aliases(
    CategoryResponse = ApiResponse<CategoryResponseBody>,
    CategoriesResponse = ApiResponse<CategoriesResponseBody>,
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
        categories::get_categories,
        categories::get_category,
        status::get_status,
        status::post_status
    ),
    components(schemas(
        Category,
        CategoryResponse,
        CategoryResponseBody,
        CategoriesResponse,
        CategoriesResponseBody,
        ErrorResponse,
        ErrorResponseBody,
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
