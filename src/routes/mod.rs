use self::{
    auth::{
        post_confirm::ConfirmRequest,
        post_forget::ForgetRequest,
        post_login::{LoginRequest, LoginResponseBody},
        post_register::RegisterRequest,
    },
    index::{
        get_categories::CategoriesResponseBody,
        get_title::TitleResponseBody,
        post_filter::{FilterRequest, FilterResponseBody},
    },
    pages::{PageResponseBody, PagesResponseBody},
    status::{StatusRequest, StatusResponseBody},
    user::CheckResponseBody,
};
use crate::models::{
    categories::Model as Category, pages::Model as Page, titles::Model as Title,
    users::Model as User,
};
use serde::{Deserialize, Serialize};
use utoipa::{OpenApi, ToSchema};

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
        CategoriesResponse,
        CategoriesResponseBody,
        Category,
        CheckResponse,
        ConfirmRequest,
        ErrorResponse,
        ErrorResponseBody,
        FilterRequest,
        FilterResponse,
        FilterResponseBody,
        ForgetRequest,
        LoginRequest,
        LoginResponse,
        LoginResponseBody,
        Page,
        PageResponse,
        PageResponseBody,
        PagesResponse,
        PagesResponseBody,
        RegisterRequest,
        StatusRequest,
        StatusResponse,
        StatusResponseBody,
        Title,
        TitleResponse,
        TitleResponseBody,
        User,
    ))
)]
pub struct ApiDoc;
