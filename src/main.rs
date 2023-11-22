use axum::{
    middleware,
    routing::{get, post},
    Router,
};
use routes::{auth::*, categories::*, pages::*, status::*, titles::*, *};
use sea_orm::{ConnectionTrait, Database, DatabaseConnection, DbBackend, DbErr};
use sea_orm_migration::prelude::*;
use std::{net::SocketAddr, sync::Arc};
use tower_http::trace::TraceLayer;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::{config::Config, migrator::Migrator};

mod config;
mod constants;
mod migrator;
mod models;
mod routes;

pub struct AppState {
    db: DatabaseConnection,
    env: Config,
}

#[tokio::main]
async fn main() -> Result<(), DbErr> {
    dotenvy::dotenv().ok();
    let config = Config::init();

    let addr_str = format!("{}:{}", config.server_address, config.server_port);

    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    let db = Database::connect(config.database_url).await?;
    let db = match db.get_database_backend() {
        DbBackend::Sqlite => db,
        _ => {
            tracing::error!("we don't support other databases outside of sqlite. exiting.");
            std::process::exit(1)
        }
    };

    let schema_manager = SchemaManager::new(&db);
    Migrator::refresh(&db).await?;
    assert!(schema_manager.has_table("users").await?);
    assert!(schema_manager.has_table("categories").await?);
    assert!(schema_manager.has_table("titles").await?);
    assert!(schema_manager.has_table("pages").await?);
    assert!(schema_manager.has_table("tags").await?);
    assert!(schema_manager.has_table("titles_tags").await?);

    let app_state = Arc::new(AppState {
        db,
        env: config.clone(),
    });

    let app = Router::new()
        .merge(SwaggerUi::new("/swagger").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .route("/api/status", get(get_status).post(post_status))
        .route("/api/auth/register", post(post_register))
        .route("/api/auth/login", post(post_login))
        .route(
            "/api/auth/logout",
            get(get_logout).route_layer(middleware::from_fn_with_state(app_state.clone(), auth)),
        )
        .route("/api/categories", get(get_categories))
        .route("/api/category/:category_id", get(get_category))
        .route("/api/titles", get(get_titles))
        .route("/api/title/:title_id", get(get_title))
        .route("/api/pages", get(get_pages))
        .route("/api/page/:page_id", get(get_page))
        .route(
            "/api/pages/by_title_id/:title_id",
            get(get_pages_by_title_id),
        )
        .layer(TraceLayer::new_for_http())
        .with_state(app_state);

    let addr = addr_str.parse::<SocketAddr>().unwrap();
    tracing::debug!("listening on: {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();

    Ok(())
}
