use crate::routes::status::StatusResponse;
use axum::{routing::get, Router};
use routes::status::status;
use std::{env, net::SocketAddr};
use tower_http::trace::TraceLayer;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

mod routes;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    let addr_str = format!(
        "{}:{}",
        env::var("SERVER_ADDRESS").unwrap(),
        env::var("SERVER_PORT").unwrap()
    );

    #[derive(OpenApi)]
    #[openapi(paths(routes::status::status), components(schemas(StatusResponse)))]
    struct ApiDoc;

    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    let app = Router::new()
        .merge(SwaggerUi::new("/swagger").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .route("/api/status", get(status))
        .layer(TraceLayer::new_for_http());

    let addr = addr_str.parse::<SocketAddr>().unwrap();
    tracing::debug!("listening on: {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
