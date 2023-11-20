use serde::{Deserialize, Serialize};
use utoipa::{OpenApi, ToSchema};

use crate::models::{category::Category, page::Page, title::Title};

use self::{
    auth::{LoginRequest, LoginResponseBody, RegisterRequest, RegisterResponseBody},
    categories::{CategoriesResponseBody, CategoryResponseBody},
    pages::{PageByTitleIdRequest, PageResponseBody, PagesResponseBody},
    status::{StatusRequest, StatusResponseBody},
    titles::{TitleResponseBody, TitlesResponseBody},
};

pub mod auth;
pub mod categories;
pub mod pages;
pub mod status;
pub mod titles;

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
    paths(
        auth::post_login,
        auth::post_register,
        auth::get_logout,
        categories::get_categories,
        categories::get_category,
        pages::get_pages,
        pages::get_page,
        pages::post_pages_by_title_id,
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
        PageByTitleIdRequest,
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
    ))
)]
pub struct ApiDoc;
