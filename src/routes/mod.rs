pub mod auth;
pub mod file;
pub mod index;
pub mod middlewares;
pub mod user;
pub mod utils;

pub use self::{auth::*, index::*, user::*, utils::*};
pub use middlewares::auth::auth;

use crate::models::categories::Model as Categories;
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
    // Auth
    LoginResponse = ApiResponse<LoginResponseBody>,

    // User
    CategoriesResponse = ApiResponse<CategoriesResponseBody>,
    TitleResponse = ApiResponse<TitleResponseBody>,
    FilterResponse = ApiResponse<FilterResponseBody>,

    // Utils
    StatusResponse = ApiResponse<StatusResponseBody>,
    TagsMapResponse = ApiResponse<TagsMapResponseBody>,
    ScanningProgressResponse = ApiResponse<ScanningProgressResponseBody>,

    // Other
    ErrorResponse = ApiResponse<ErrorResponseBody>,
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
            description = "login, register, logout."
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
            description = "getting server status, item/category id-name map"
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

        user::delete_bookmark,
        user::delete_favorite,
        user::get_check,
        user::get_delete,
        user::get_reset,
        user::get_verify,
        user::post_delete,
        user::post_modify,
        user::post_reset,
        user::post_verify,
        user::put_bookmark,
        user::put_favorite,

        index::get_categories,
        index::post_filter,
        index::get_title,

        utils::get_status,
        utils::post_status,
        utils::get_tags,
        utils::get_scanning_progress,

        file::get_page,
        file::get_thumbnail,
        file::head_thumbnail,
    ),
    components(schemas(
        // Note: no need to declare non-Body structs, as long as they are declared in the aliases macro.

        // Auth
        LoginRequest,
        LoginResponse,
        LoginResponseBody,
        RegisterRequest,

        // User
        DeleteRequest,
        ModifyRequest,
        ResetRequest,

        // Index
        Categories,
        CategoriesResponse,
        CategoriesResponseBody,
        TitleResponse,
        TitleResponseBody,
        FilterRequest,
        FilterResponse,
        FilterResponseBody,
        FilterTitleResponseBody,

        // Utils
        StatusRequest,
        StatusResponse,
        StatusResponseBody,
        TagsMapResponse,
        TagsMapResponseBody,
        TitleResponse,
        TitleResponseBody,
        ScanningProgressResponse,
        ScanningProgressResponseBody,

        // Other
        ErrorResponse,
        ErrorResponseBody,
    ))
)]
pub struct ApiDoc;

fn build_resp<T: Serialize>(status: StatusCode, body: T) -> (StatusCode, Json<ApiResponse<T>>) {
    (
        status,
        Json(ApiResponse {
            description: status.canonical_reason().unwrap_or_default().to_string(),
            body: Some(body),
        }),
    )
}

fn build_err_resp<S: AsRef<str>>(
    status: StatusCode,
    body: S,
) -> (StatusCode, Json<ApiResponse<ErrorResponseBody>>) {
    (
        status,
        Json(ApiResponse {
            description: status.canonical_reason().unwrap_or_default().to_string(),
            body: Some(ErrorResponseBody {
                message: body.as_ref().to_string(),
            }),
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
