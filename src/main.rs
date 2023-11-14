use axum::{
    middleware,
    routing::{get, post},
    Router,
};
use routes::{auth::*, status::*, *};
use sqlx::{migrate::MigrateDatabase, sqlite::SqlitePoolOptions, Sqlite};
use std::{net::SocketAddr, sync::Arc};
use tower_http::trace::TraceLayer;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::config::Config;

mod config;
mod constants;
mod models;
mod routes;

pub struct AppState {
    sqlite: sqlx::Pool<Sqlite>,
    env: Config,
}

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    let config = Config::init();

    let addr_str = format!("{}:{}", config.server_address, config.server_port);

    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    if !Sqlite::database_exists(&config.database_url)
        .await
        .unwrap_or(false)
    {
        tracing::info!(
            "cannot find database, initializing a new one at: {}",
            &config.database_url
        );
        match Sqlite::create_database(&config.database_url).await {
            Ok(_) => tracing::info!("created new database successfully."),
            Err(e) => {
                tracing::error!("error while creating database: {}", e);
                std::process::exit(1);
            }
        }
    }

    let pool = match SqlitePoolOptions::new()
        .max_connections(10)
        .connect(&config.database_url)
        .await
    {
        Ok(pool) => {
            tracing::info!("connection to sqlite database successful.");
            pool
        }
        Err(err) => {
            tracing::error!("failed to connect to sqlite database: {:?}", err);
            std::process::exit(1);
        }
    };

    let crate_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let migrations = std::path::Path::new(&crate_dir).join("./migrations");

    let migration_results = sqlx::migrate::Migrator::new(migrations)
        .await
        .unwrap()
        .run(&pool)
        .await;

    match migration_results {
        Ok(_) => tracing::info!("migrations successful!"),
        Err(e) => {
            tracing::error!("error during database migration: {}", e);
            std::process::exit(1);
        }
    }

    let app_state = Arc::new(AppState {
        sqlite: pool.clone(),
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
        .layer(TraceLayer::new_for_http())
        .with_state(app_state);

    let addr = addr_str.parse::<SocketAddr>().unwrap();
    tracing::debug!("listening on: {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
