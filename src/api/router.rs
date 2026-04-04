use crate::api::handlers::{self, login_user, register_user};
use crate::api::task_handlers::{
    self as th, create_task, delete_task, get_all_tasks, get_task_by_id, get_tasks_by_user,
    update_task,
};
use crate::application::task_service::TaskService;
use crate::application::user_service::UserService;
use crate::domain::task::{CreateTask, TaskResponse, UpdateTask};
use crate::domain::user::{CreateUser, LoginUser, UserResponse};
use crate::domain::error::ErrorResponse;
use axum::{
    routing::{get, post},
    Router,
};
use std::sync::Arc;
use tower_http::trace::TraceLayer;
use utoipa::{
    openapi::security::{HttpAuthScheme, HttpBuilder, SecurityScheme},
    Modify, OpenApi,
};
use utoipa_swagger_ui::SwaggerUi;

// ─── Swagger Security Scheme ─────────────────────────────────────────────────

struct BearerSecurityAddon;

impl Modify for BearerSecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        let components = openapi.components.get_or_insert_with(Default::default);
        components.add_security_scheme(
            "bearer_auth",
            SecurityScheme::Http(
                HttpBuilder::new()
                    .scheme(HttpAuthScheme::Bearer)
                    .bearer_format("JWT")
                    .build(),
            ),
        );
    }
}

// ─── OpenAPI Doc ─────────────────────────────────────────────────────────────

#[derive(OpenApi)]
#[openapi(
    paths(
        handlers::register_user,
        handlers::login_user,
        th::create_task,
        th::get_all_tasks,
        th::get_task_by_id,
        th::get_tasks_by_user,
        th::update_task,
        th::delete_task,
    ),
    components(
        schemas(CreateUser, LoginUser, UserResponse, CreateTask, UpdateTask, TaskResponse, ErrorResponse)
    ),
    modifiers(&BearerSecurityAddon),
    tags(
        (name = "auth", description = "Layanan autentikasi dan registrasi user"),
        (name = "tasks", description = "Layanan manajemen tugas (Memerlukan JWT Token)")
    ),
    info(
        title = "Task Management API",
        version = "1.0.0",
        description = "API untuk mengelola tugas dan user menggunakan Axum dan SQLx",
        contact(name = "Rahman", email = "rahman@example.com"),
        license(name = "MIT", url = "https://opensource.org/licenses/MIT")
    )
)]
pub struct ApiDoc;

// ─── Router ───────────────────────────────────────────────────────────────────

pub fn create_router(user_service: Arc<UserService>, task_service: Arc<TaskService>) -> Router {
    let task_routes = Router::new()
        .route("/", post(create_task).get(get_all_tasks))
        .route(
            "/:id",
            get(get_task_by_id).put(update_task).delete(delete_task),
        )
        .route("/user/:id_user", get(get_tasks_by_user))
        .with_state(task_service);

    Router::new()
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .route("/api/auth/register", post(register_user))
        .route("/api/auth/login", post(login_user))
        .with_state(user_service)
        .nest("/api/tasks", task_routes)
        .layer(TraceLayer::new_for_http())
}
