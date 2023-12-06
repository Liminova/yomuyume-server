use serde::{Deserialize, Serialize};
use utoipa::{OpenApi, ToSchema};

use crate::models::{
    categories::Model as Category, pages::Model as Page, titles::Model as Title,
    users::Model as User,
};

use self::{
    auth::{LoginRequest, LoginResponseBody, RegisterRequest, RegisterResponseBody},
    categories::{CategoriesResponseBody, CategoryResponseBody},
    pages::{PageResponseBody, PagesResponseBody},
    status::{StatusRequest, StatusResponseBody},
    titles::{TitleResponseBody, TitlesResponseBody},
};

pub mod auth;
pub mod categories;
pub mod pages;
pub mod status;
pub mod titles;

#[derive(Clone, Deserialize, Serialize, ToSchema, Debug)]
pub struct ErrorResponseBody {
    /// The error message.
    pub message: String,
}

#[derive(Clone, Deserialize, Serialize, ToSchema, Debug)]
#[aliases(
    CategoryResponse = ApiResponse<CategoryResponseBody>,
    CategoriesResponse = ApiResponse<CategoriesResponseBody>,
    ErrorResponse = ApiResponse<ErrorResponseBody>,
    LoginResponse = ApiResponse<LoginResponseBody>,
    PageResponse = ApiResponse<PageResponseBody>,
    PagesResponse = ApiResponse<PagesResponseBody>,
    RegisterResponse = ApiResponse<RegisterResponseBody>,
    StatusResponse = ApiResponse<StatusResponseBody>,
    TitleResponse = ApiResponse<TitleResponseBody>,
    TitlesResponse = ApiResponse<TitlesResponseBody>,
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
            name = "categories",
            description = "all the routes related to fetching categories."
        ),
        (
            name = "pages",
            description = "all the routes related to fetching pages."
        ),
        (
            name = "status",
            description = "all the routes related to fetching backend status."
        ),
        (
            name = "titles",
            description = "all the routes related to fetching titles."
        ),
    ),
    paths(
        auth::post_login,
        auth::post_register,
        auth::get_logout,
        categories::get_categories,
        categories::get_category,
        pages::get_pages,
        pages::get_page,
        pages::get_pages_by_title_id,
        status::get_status,
        status::post_status,
        titles::get_titles,
        titles::get_title,
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
        Page,
        PageResponse,
        PageResponseBody,
        PagesResponse,
        PagesResponseBody,
        RegisterResponse,
        RegisterResponseBody,
        RegisterRequest,
        StatusResponse,
        StatusResponseBody,
        StatusRequest,
        Title,
        TitleResponse,
        TitleResponseBody,
        TitlesResponse,
        TitlesResponseBody,
        User,
    ))
)]
pub struct ApiDoc;
