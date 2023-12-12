use axum::{
    middleware,
    routing::{get, post},
    Router,
};
use middleware::from_fn_with_state;
use routes::{auth::*, index::*, pages::*, status::*, *};
use sea_orm::{ConnectionTrait, Database, DatabaseConnection, DbBackend, DbErr};
use sea_orm_migration::prelude::*;
use std::{net::SocketAddr, sync::Arc};
use tokio::net::TcpListener;
use tower_http::trace::TraceLayer;
use tracing::info;
use utoipa::OpenApi;
use utoipa_redoc::{Redoc, Servable};
use utoipa_swagger_ui::SwaggerUi;

use crate::{config::Config, migrator::Migrator};

mod config;
mod constants;
mod migrator;
mod models;
mod routes;
mod utils;

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

    let db = Database::connect(&config.database_url).await?;
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
    assert!(schema_manager.has_table("bookmarks").await?);
    assert!(schema_manager.has_table("thumbnails").await?);
    assert!(schema_manager.has_table("favorites").await?);
    assert!(schema_manager.has_table("progresses").await?);

    info!("database migrations complete!");

    let app_state = Arc::new(AppState {
        db,
        env: config.clone(),
    });

    let app = Router::new()
        .merge(SwaggerUi::new("/swagger").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .merge(Redoc::with_url("/redoc", ApiDoc::openapi()))
        .route("/api/status", get(get_status).post(post_status))
        .route("/api/auth/register", post(post_register))
        .route("/api/auth/login", post(post_login))
        .route(
            "/api/auth/logout",
            get(get_logout).route_layer(from_fn_with_state(app_state.clone(), auth)),
        )
        .route(
            "/api/intex/filter",
            post(filter::post_filter).route_layer(from_fn_with_state(app_state.clone(), auth)),
        )
        .route(
            "/api/index/categories",
            get(categories::get_categories)
                .route_layer(from_fn_with_state(app_state.clone(), auth)),
        )
        .route(
            "/api/title/:title_id",
            get(title::get_title).route_layer(from_fn_with_state(app_state.clone(), auth)),
        )
        .route(
            "/api/pages",
            get(get_pages).route_layer(from_fn_with_state(app_state.clone(), auth)),
        )
        .route(
            "/api/page/:page_id",
            get(get_page).route_layer(from_fn_with_state(app_state.clone(), auth)),
        )
        .route(
            "/api/pages/by_title_id/:title_id",
            get(get_pages_by_title_id).route_layer(from_fn_with_state(app_state.clone(), auth)),
        )
        .layer(TraceLayer::new_for_http())
        .with_state(app_state);

    let addr = addr_str.parse::<SocketAddr>().unwrap();
    let listener = TcpListener::bind(&addr).await.unwrap();
    tracing::debug!("listening on: {}", addr);
    let _ = axum::serve(listener, app.into_make_service()).await;

    Ok(())
}
