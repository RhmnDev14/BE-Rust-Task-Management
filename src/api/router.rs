use crate::api::handlers::{self, login_user, register_user};
use crate::api::s3_handlers::{self as s3h, get_presigned_url, PresignedUrlRequest, PresignedUrlResponse};
use crate::api::task_handlers::{
    self as th, create_task, delete_task, get_all_tasks, get_task_by_id, get_tasks_by_user,
    update_task,
};
use crate::api::group_handlers::{
    self as gh, create_group, delete_group, get_all_groups, get_group_by_id, update_group,
};
use crate::application::task_service::TaskService;
use crate::application::user_service::UserService;
use crate::application::group_service::GroupService;
use crate::infrastructure::s3::S3Client;
use crate::domain::task::{CreateTask, PaginatedResponse, PaginationParams, TaskResponse, UpdateTask};
use crate::domain::group::{CreateGroup, GroupResponse, UpdateGroup};
use crate::domain::user::{ChangePassword, CreateUser, LoginUser, MessageResponse, UpdateUser, UserResponse};
use crate::domain::error::ErrorResponse;
use axum::{
    routing::{get, post, put},
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
        handlers::get_me,
        handlers::update_me,
        handlers::change_password,
        th::create_task,
        th::get_all_tasks,
        th::get_task_by_id,
        th::get_tasks_by_user,
        th::search_tasks,
        th::update_task,
        th::delete_task,
        gh::create_group,
        gh::get_all_groups,
        gh::get_group_by_id,
        gh::update_group,
        gh::delete_group,
        s3h::get_presigned_url,
    ),
    components(
        schemas(
            CreateUser, LoginUser, UpdateUser, ChangePassword, UserResponse, MessageResponse,
            CreateTask, UpdateTask, TaskResponse, PaginationParams, PaginatedResponse<TaskResponse>,
            CreateGroup, UpdateGroup, GroupResponse, PaginatedResponse<GroupResponse>,
            ErrorResponse,
            PresignedUrlRequest, PresignedUrlResponse
        )
    ),
    modifiers(&BearerSecurityAddon),
    tags(
        (name = "auth", description = "Layanan autentikasi dan registrasi user"),
        (name = "tasks", description = "Layanan manajemen tugas (Memerlukan JWT Token)"),
        (name = "groups", description = "Layanan manajemen grup (Memerlukan JWT Token)"),
        (name = "s3", description = "Layanan S3/MinIO untuk file storage")
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

pub fn create_router(
    user_service: Arc<UserService>, 
    task_service: Arc<TaskService>,
    group_service: Arc<GroupService>,
    s3_client: Arc<S3Client>,
) -> Router {
    let task_routes = Router::new()
        .route("/", post(create_task).get(get_all_tasks))
        .route(
            "/:id",
            get(get_task_by_id).put(update_task).delete(delete_task),
        )
        .route("/my", get(get_tasks_by_user))
        .route("/search", get(th::search_tasks))
        .with_state(task_service);

    let group_routes = Router::new()
        .route("/", post(create_group).get(get_all_groups))
        .route(
            "/:id",
            get(get_group_by_id).put(update_group).delete(delete_group),
        )
        .with_state(group_service);

    let s3_routes = Router::new()
        .route("/presigned-url", post(get_presigned_url))
        .with_state(s3_client);

    Router::new()
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .route("/api/auth/register", post(register_user))
        .route("/api/auth/login", post(login_user))
        .route("/api/auth/me", get(handlers::get_me).put(handlers::update_me))
        .route("/api/auth/change-password", put(handlers::change_password))
        .with_state(user_service)
        .nest("/api/tasks", task_routes)
        .nest("/api/groups", group_routes)
        .nest("/api/s3", s3_routes)
        .layer(TraceLayer::new_for_http())
}
