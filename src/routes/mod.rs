pub mod auth;
pub mod index;
pub mod pages;
pub mod user;
pub mod utils;

pub use self::{
    auth::{LoginRequest, LoginResponseBody, RegisterRequest},
    index::{CategoriesResponseBody, FilterRequest, FilterResponseBody, TitleResponseBody},
    pages::{PageResponseBody, PagesResponseBody},
    user::{DeleteRequestBody, ModifyRequestBody, ResetRequestBody},
    utils::{StatusRequest, StatusResponseBody},
};
use crate::models::{
    categories::Model as Category, pages::Model as Page, titles::Model as Title,
    users::Model as User,
};
use serde::{Deserialize, Serialize};
use utoipa::{OpenApi, ToSchema};

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
            description = "Login, register, logout."
        ),
        (
            name = "index",
            description = "all the routes related to fetching index data."
        ),
        (
            name = "user",
            description = "all the routes related to user."
        ),
        (
            name = "utils",
            description = "Getting server status, item/category id-name map"
        )
    ),
    paths(
        auth::post_login,
        auth::post_register,
        auth::get_logout,
        user::get_check,
        user::post_modify,
        user::get_delete,
        user::post_delete,
        user::get_reset,
        user::post_reset,
        user::get_verify,
        user::post_verify,
        user::delete_bookmark,
        user::delete_favorite,
        user::put_bookmark,
        user::put_favorite,
        index::get_categories,
        index::post_filter,
        index::get_title,
        utils::get_status,
        utils::post_status,
    ),
    components(schemas(
        CategoriesResponse,
        CategoriesResponseBody,
        Category,
        DeleteRequestBody,
        ErrorResponse,
        ErrorResponseBody,
        FilterRequest,
        FilterResponse,
        FilterResponseBody,
        LoginRequest,
        LoginResponse,
        LoginResponseBody,
        ModifyRequestBody,
        Page,
        PageResponse,
        PageResponseBody,
        PagesResponse,
        PagesResponseBody,
        RegisterRequest,
        ResetRequestBody,
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
