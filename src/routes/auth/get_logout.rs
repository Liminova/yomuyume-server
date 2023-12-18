use axum::{
    http::{header, StatusCode},
    response::IntoResponse,
};
use axum_extra::extract::cookie::{Cookie, SameSite};

/// Reset all the cookies to log the user out.
#[utoipa::path(get, path = "/api/auth/logout", responses((status = 200, description = "Logout successful.", body = ErrorResponse)))]
pub async fn get_logout() -> impl IntoResponse {
    let cookie = Cookie::build(("token", ""))
        .path("/")
        .max_age(time::Duration::hours(-1))
        .same_site(SameSite::Lax)
        .http_only(true);

    (StatusCode::OK, [(header::SET_COOKIE, cookie.to_string())])
}
