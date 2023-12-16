use std::{net::SocketAddr, sync::Arc};

use axum::{
    middleware::from_fn_with_state,
    Router,
    routing::{get, post},
};
use sea_orm::{ConnectionTrait, Database, DatabaseConnection, DbBackend, DbErr};
use sea_orm_migration::prelude::*;
use tokio::net::TcpListener;
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tracing::info;
use utoipa::OpenApi;
use utoipa_redoc::{Redoc, Servable};
use utoipa_swagger_ui::SwaggerUi;

use routes::{auth::*, index::*, pages::*, status::*, user::*};

use crate::{config::Config, migrator::Migrator, routes::ApiDoc};

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
    Migrator::up(&db, None).await?;
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

    let auth_routes = Router::new()
        .route("/register", post(post_register))
        .route("/login", post(post_login))
        .route(
            "/logout",
            get(get_logout).route_layer(from_fn_with_state(app_state.clone(), auth)),
        );

    let user_routes = Router::new()
        .route("/check", get(get_check))
        .layer(from_fn_with_state(app_state.clone(), auth));

    let index_routes = Router::new()
        .route("/filter", post(filter::post_filter))
        .route("/categories", get(categories::get_categories))
        .route("/title/:title_id", get(title::get_title))
        .layer(from_fn_with_state(app_state.clone(), auth));

    let pages_routes = Router::new()
        .route("/pages", get(get_pages))
        .route("/page/:page_id", get(get_page))
        .route("/pages/by_title_id/:title_id", get(get_pages_by_title_id))
        .layer(from_fn_with_state(app_state.clone(), auth));

    let app = Router::new()
        .nest("/api/auth", auth_routes)
        .nest("/api/index", index_routes)
        .nest("/api/pages", pages_routes)
        .nest("/api/user", user_routes)
        .route("/api/status", get(get_status).post(post_status))
        .merge(SwaggerUi::new("/swagger").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .merge(Redoc::with_url("/redoc", ApiDoc::openapi()))
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive())
        .with_state(app_state);

    let addr = addr_str.parse::<SocketAddr>().unwrap();
    let listener = TcpListener::bind(&addr).await.unwrap();
    tracing::debug!("listening on: {}", addr);
    let _ = axum::serve(listener, app.into_make_service()).await;

    Ok(())
}
