use crate::api::router::create_router;
use crate::application::task_service::TaskService;
use crate::application::user_service::UserService;
use crate::infrastructure::repositories::SqlxUserRepository;
use crate::infrastructure::task_repository::SqlxTaskRepository;
use dotenvy::dotenv;
use sqlx::postgres::PgPoolOptions;
use std::env;
use std::net::SocketAddr;
use std::sync::Arc;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod api;
mod application;
mod domain;
mod infrastructure;

#[tokio::main]
async fn main() {
    dotenv().ok();

    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            env::var("RUST_LOG")
                .unwrap_or_else(|_| "be_rust_task_management=debug,tower_http=debug".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
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
    let task_repository = Box::new(SqlxTaskRepository::new(pool));
    let task_service = Arc::new(TaskService::new(task_repository));

    let app = create_router(user_service, task_service);

    let host = env::var("HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let port = env::var("PORT").unwrap_or_else(|_| "3000".to_string());
    let addr: SocketAddr = format!("{}:{}", host, port)
        .parse()
        .expect("Invalid address");

    tracing::info!("listening on {}", addr);
    tracing::info!("Swagger UI available at http://{}/swagger-ui", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
