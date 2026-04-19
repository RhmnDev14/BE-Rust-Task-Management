use crate::api::router::create_router;
use crate::application::task_service::TaskService;
use crate::application::user_service::UserService;
use crate::application::group_service::GroupService;
use crate::application::master_service::MasterService;
use crate::repository::user_repository::SqlxUserRepository;
use crate::repository::task_repository::SqlxTaskRepository;
use crate::repository::group_repository::SqlxGroupRepository;
use crate::repository::master_repository::SqlxMasterRepository;
use dotenvy::dotenv;
use sqlx::postgres::{PgConnectOptions, PgPoolOptions};
use sqlx::ConnectOptions;
use tracing::log::LevelFilter;
use std::env;
use std::net::SocketAddr;
use std::sync::Arc;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod api;
mod application;
mod domain;
mod infrastructure;
mod repository;

#[tokio::main]
async fn main() {
    dotenv().ok();

    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            env::var("RUST_LOG")
                .unwrap_or_else(|_| "be_rust_task_management=debug,tower_http=debug,sqlx=info".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let connection_options = database_url
        .parse::<PgConnectOptions>()
        .expect("Invalid DATABASE_URL")
        .log_statements(LevelFilter::Info);

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect_with(connection_options)
        .await
        .expect("Failed to connect to Postgres");

    // Run migrations
    sqlx::migrate!()
        .run(&pool)
        .await
        .expect("Failed to run migrations");

    // User service
    let user_repository = Box::new(SqlxUserRepository::new(pool.clone()));
    let jwt_secret = env::var("JWT_SECRET").expect("JWT_SECRET must be set");
    let user_service = Arc::new(UserService::new(user_repository, jwt_secret));

    // Task service
    let task_repository = Box::new(SqlxTaskRepository::new(pool.clone()));
    let task_service = Arc::new(TaskService::new(task_repository));

    // Group service
    let group_repository = Box::new(SqlxGroupRepository::new(pool.clone()));
    let group_service = Arc::new(GroupService::new(group_repository));

    // Master service
    let master_repository = Box::new(SqlxMasterRepository::new(pool));
    let master_service = Arc::new(MasterService::new(master_repository));

    // S3 client
    let s3_client = Arc::new(crate::infrastructure::s3::S3Client::new().await);

    let app = create_router(user_service, task_service, group_service, master_service, s3_client);

    let host = env::var("HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let port = env::var("PORT").unwrap_or_else(|_| "3000".to_string());
    let addr: SocketAddr = format!("{}:{}", host, port)
        .parse()
        .expect("Invalid address");

    tracing::info!("listening on {}", addr);
    tracing::info!("Swagger UI available at http://{}/swagger-ui", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap();
}

async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    tracing::info!("signal received, starting graceful shutdown");
}
