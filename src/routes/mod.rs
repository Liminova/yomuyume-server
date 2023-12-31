pub mod auth;
pub mod file;
pub mod index;
pub mod middlewares;
pub mod user;
pub mod utils;

pub use self::{
    auth::{LoginRequest, LoginResponseBody, RegisterRequest},
    index::{
        CategoriesResponseBody, FilterRequest, FilterResponseBody, FilterTitleResponseBody,
        TitleResponseBody,
    },
    user::{DeleteRequestBody, ModifyRequestBody, ResetRequestBody},
    utils::{StatusRequest, StatusResponseBody, TagsMapResponseBody},
};
pub use middlewares::auth::auth;

use crate::models::{
    categories::Model as Category, pages::Model as Page, titles::Model as Title,
    users::Model as User,
};
use argon2::{Argon2, PasswordHash, PasswordVerifier};
use axum::{http::StatusCode, Json};
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
    FilterResponse = ApiResponse<FilterResponseBody>,
    LoginResponse = ApiResponse<LoginResponseBody>,
    StatusResponse = ApiResponse<StatusResponseBody>,
    TagsMapResponse = ApiResponse<TagsMapResponseBody>,
    TitleResponse = ApiResponse<TitleResponseBody>,
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
        ),
        (
            name = "file",
            description = "all the routes related to file fetching."
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
        utils::get_tags,
        file::get_page,
        file::get_thumbnail,
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
        FilterTitleResponseBody,
        LoginRequest,
        LoginResponse,
        LoginResponseBody,
        ModifyRequestBody,
        Page,
        RegisterRequest,
        ResetRequestBody,
        StatusRequest,
        StatusResponse,
        StatusResponseBody,
        TagsMapResponse,
        TagsMapResponseBody,
        Title,
        TitleResponse,
        TitleResponseBody,
        User,
    ))
)]
pub struct ApiDoc;

fn build_resp<T: Serialize>(
    status: StatusCode,
    description: String,
    body: T,
) -> (StatusCode, Json<ApiResponse<T>>) {
    (
        status,
        Json(ApiResponse {
            description,
            body: Some(body),
        }),
    )
}

fn build_err_resp(
    status: StatusCode,
    description: String,
    body: String,
) -> (StatusCode, Json<ApiResponse<ErrorResponseBody>>) {
    (
        status,
        Json(ApiResponse {
            description,
            body: Some(ErrorResponseBody { message: body }),
        }),
    )
}

pub fn check_pass(real: &str, input: &String) -> bool {
    match PasswordHash::new(real) {
        Ok(parsed_hash) => Argon2::default()
            .verify_password(input.as_bytes(), &parsed_hash)
            .map_or(false, |_| true),
        Err(_) => false,
    }
}
