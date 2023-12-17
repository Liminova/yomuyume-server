use serde::{Deserialize, Serialize};
use utoipa::{OpenApi, ToSchema};

use crate::models::{
    categories::Model as Category, pages::Model as Page, titles::Model as Title,
    users::Model as User,
};

use self::{
    auth::{LoginRequest, LoginResponseBody, RegisterRequest},
    index::{
        categories::CategoriesResponseBody,
        filter::{FilterRequest, FilterResponseBody},
        title::TitleResponseBody,
    },
    pages::{PageResponseBody, PagesResponseBody},
    status::{StatusRequest, StatusResponseBody},
    user::CheckResponseBody,
};

pub mod auth;
pub mod index;
pub mod pages;
pub mod status;
pub mod user;

#[derive(Clone, Deserialize, Serialize, ToSchema, Debug)]
pub struct ErrorResponseBody {
    /// The error message.
    pub message: String,
}

#[derive(Clone, Deserialize, Serialize, ToSchema, Debug)]
#[aliases(
    CategoriesResponse = ApiResponse<CategoriesResponseBody>,
    ErrorResponse = ApiResponse<ErrorResponseBody>,
    LoginResponse = ApiResponse<LoginResponseBody>,
    PageResponse = ApiResponse<PageResponseBody>,
    PagesResponse = ApiResponse<PagesResponseBody>,
    StatusResponse = ApiResponse<StatusResponseBody>,
    TitleResponse = ApiResponse<TitleResponseBody>,
    FilterResponse = ApiResponse<FilterResponseBody>,
    CheckResponse = ApiResponse<CheckResponseBody>,
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
    info(
        description = "yomuyume's backend documentations.",
        license(name = "MIT or Apache-2.0"),
    ),
    tags(
        (
            name = "auth",
            description = "all the routes related to authentication."
        ),
        (
            name = "status",
            description = "all the routes related to fetching backend status."
        ),
        (
            name = "index",
            description = "all the routes related to fetching index data."
        ),
        (
            name = "user",
            description = "all the routes related to fetching pages data."
        ),
    ),
    paths(
        auth::post_login,
        auth::post_register,
        auth::get_logout,
        user::get_check,
        index::get_categories,
        index::post_filter,
        index::get_title,
        status::get_status,
        status::post_status,
    ),
    components(schemas(
        Category,
        CategoriesResponse,
        CategoriesResponseBody,
        ErrorResponse,
        ErrorResponseBody,
        FilterRequest,
        FilterResponse,
        FilterResponseBody,
        LoginResponse,
        LoginResponseBody,
        LoginRequest,
        Page,
        PageResponse,
        PageResponseBody,
        PagesResponse,
        PagesResponseBody,
        RegisterRequest,
        StatusResponse,
        StatusResponseBody,
        StatusRequest,
        Title,
        TitleResponse,
        TitleResponseBody,
        User,
        CheckResponse,
    ))
)]
pub struct ApiDoc;
