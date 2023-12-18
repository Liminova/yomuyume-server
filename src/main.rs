use crate::middlewares::auth::auth;
use crate::{config::Config, migrator::Migrator, routes::ApiDoc};
use axum::{
    middleware::from_fn_with_state as apply,
    routing::{get, post},
    Router,
};
use routes::{
    auth::{get_logout, post_login, post_register},
    index::{get_categories, get_title, post_filter},
    pages::{get_page, get_pages, get_pages_by_title_id},
    status::{get_status, post_status},
    user::{
        get_check, get_delete, get_reset, get_verify, post_delete, post_modify, post_reset,
        post_verify,
    },
};
use sea_orm::{ConnectionTrait, Database, DatabaseConnection, DbBackend, DbErr};
use sea_orm_migration::prelude::*;
use std::{net::SocketAddr, sync::Arc};
use tokio::net::TcpListener;
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tracing::info;
use utoipa::OpenApi;
use utoipa_redoc::{Redoc, Servable};
use utoipa_swagger_ui::SwaggerUi;

mod config;
mod constants;
mod middlewares;
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
            get(get_logout).route_layer(apply(app_state.clone(), auth)),
        );

    let user_routes = Router::new()
        .route("/check", get(get_check))
        .route("/reset", post(post_reset))
        .route("/delete", get(get_delete).post(post_delete))
        .route("/verify", get(get_verify).post(post_verify))
        .route("/modify", post(post_modify))
        .layer(apply(app_state.clone(), auth));

    let index_routes = Router::new()
        .route("/filter", post(post_filter))
        .route("/categories", get(get_categories))
        .route("/title/:title_id", get(get_title))
        .layer(apply(app_state.clone(), auth));

    let pages_routes = Router::new()
        .route("/pages", get(get_pages))
        .route("/page/:page_id", get(get_page))
        .route("/pages/by_title_id/:title_id", get(get_pages_by_title_id))
        .layer(apply(app_state.clone(), auth));

    let app = Router::new()
        .nest("/api/auth", auth_routes)
        .nest("/api/index", index_routes)
        .nest("/api/pages", pages_routes)
        .nest("/api/user", user_routes)
        .route("/api/user/reset", get(get_reset))
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
