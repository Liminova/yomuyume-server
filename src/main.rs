use axum::{
    routing::{get, post},
    Router,
};
use routes::{auth::*, status::*, *};
use std::{env, net::SocketAddr};
use tower_http::trace::TraceLayer;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

mod constants;
mod routes;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    let addr_str = format!(
        "{}:{}",
        env::var("SERVER_ADDRESS").unwrap(),
        env::var("SERVER_PORT").unwrap()
    );

    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    let app = Router::new()
        .merge(SwaggerUi::new("/swagger").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .route("/api/status", get(get_status).post(post_status))
        .route("/api/auth", post(auth))
        .layer(TraceLayer::new_for_http());

    let addr = addr_str.parse::<SocketAddr>().unwrap();
    tracing::debug!("listening on: {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
