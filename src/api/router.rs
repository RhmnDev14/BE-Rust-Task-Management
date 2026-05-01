use crate::api::handlers::{self, login_user, register_user, get_user_options, update_user_role};
use crate::api::s3_handlers::{self as s3h, get_presigned_url, get_file_view_url, PresignedUrlRequest, PresignedUrlResponse};
use crate::api::task_handlers::{
    self as th, create_task, delete_task, get_all_tasks, get_task_by_id, get_tasks_by_user,
    update_task,
};
use crate::api::group_handlers::{
    self as gh, create_group, delete_group, get_all_groups, get_group_by_id, get_group_members,
    update_group,
};
use crate::api::notification_handlers::{self as nh, get_user_notifications, mark_as_read, get_unread_count};
use crate::api::master_handlers::{self as mh, get_progress_options};
use crate::application::task_service::TaskService;
use crate::application::user_service::UserService;
use crate::application::group_service::GroupService;
use crate::application::master_service::MasterService;
use crate::application::notification_service::NotificationService;
use crate::infrastructure::s3::S3Client;
use crate::domain::task::{CreateTask, PaginatedResponse, PaginationParams, TaskResponse, UpdateTask};
use crate::domain::group::{CreateGroup, GroupMember, GroupResponse, UpdateGroup};
use crate::domain::master::{MenuOption, ProgressOption, RoleOption};
use crate::domain::notification::{MarkAsReadRequest, NotificationResponse, UnreadCountResponse};
use crate::domain::user::{ChangePassword, CreateUser, LoginUser, MessageResponse, UpdateUser, UpdateUserRoleRequest, UserOption, UserResponse};
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
        handlers::get_user_options,
        handlers::update_user_role,
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
        gh::get_group_members,
        gh::update_group,
        gh::delete_group,
        mh::get_progress_options,
        mh::get_role_options,
        mh::get_menu_options,
        nh::get_user_notifications,
        nh::mark_as_read,
        nh::get_unread_count,
        s3h::get_presigned_url,
        s3h::get_file_view_url,
    ),
    components(
        schemas(
            CreateUser, LoginUser, UpdateUser, UpdateUserRoleRequest, ChangePassword, UserResponse, UserOption, MessageResponse,
            CreateTask, UpdateTask, TaskResponse, PaginationParams, PaginatedResponse<TaskResponse>,
            CreateGroup, UpdateGroup, GroupResponse, GroupMember, PaginatedResponse<GroupResponse>,
            ProgressOption, RoleOption, MenuOption, NotificationResponse, MarkAsReadRequest, UnreadCountResponse,
            ErrorResponse,
            PresignedUrlRequest, PresignedUrlResponse
        )
    ),
    modifiers(&BearerSecurityAddon),
    tags(
        (name = "auth", description = "Layanan autentikasi dan registrasi user"),
        (name = "users", description = "Layanan data user"),
        (name = "tasks", description = "Layanan manajemen tugas (Memerlukan JWT Token)"),
        (name = "groups", description = "Layanan manajemen grup (Memerlukan JWT Token)"),
        (name = "master", description = "Layanan data master"),
        (name = "notifications", description = "Layanan notifikasi (Memerlukan JWT Token)"),
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
    master_service: Arc<MasterService>,
    notification_service: Arc<NotificationService>,
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
        .route("/:id/members", get(get_group_members))
        .with_state(group_service);

    let master_routes = Router::new()
        .route("/progress", get(get_progress_options))
        .route("/role", get(mh::get_role_options))
        .route("/menu", get(mh::get_menu_options))
        .with_state(master_service);

    let s3_routes = Router::new()
        .route("/presigned-url", post(get_presigned_url))
        .route("/view/:file_name", get(get_file_view_url))
        .with_state(s3_client);

    let notification_routes = Router::new()
        .route("/", get(get_user_notifications))
        .route("/read", put(mark_as_read))
        .route("/unread-count", get(get_unread_count))
        .with_state(notification_service);

    Router::new()
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .route("/api/auth/register", post(register_user))
        .route("/api/auth/login", post(login_user))
        .route("/api/auth/me", get(handlers::get_me).put(handlers::update_me))
        .route("/api/auth/change-password", put(handlers::change_password))
        .route("/api/users/options", get(get_user_options))
        .route("/api/users/:id/role", put(update_user_role))
        .with_state(user_service)
        .nest("/api/tasks", task_routes)
        .nest("/api/groups", group_routes)
        .nest("/api/master", master_routes)
        .nest("/api/notifications", notification_routes)
        .nest("/api/s3", s3_routes)
        .layer(TraceLayer::new_for_http())
}
