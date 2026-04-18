#![allow(unused_imports)]
use crate::{
    api::middleware::AuthUser,
    application::task_service::TaskService,
    domain::error::ErrorResponse,
    domain::task::{CreateTask, TaskError, TaskResponse, UpdateTask},
};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use std::sync::Arc;
use uuid::Uuid;

// ─── Create Task ────────────────────────────────────────────────────────────

/// Membuat tugas baru
#[utoipa::path(
    post,
    path = "/api/tasks",
    request_body = CreateTask,
    responses(
        (status = 201, description = "Task berhasil dibuat", body = TaskResponse),
        (status = 401, description = "Unauthorized - token tidak valid", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    security(("bearer_auth" = [])),
    tag = "tasks"
)]
#[tracing::instrument(skip(task_service, _auth, payload))]
pub async fn create_task(
    _auth: AuthUser,
    State(task_service): State<Arc<TaskService>>,
    Json(payload): Json<CreateTask>,
) -> Result<impl IntoResponse, TaskAppError> {
    tracing::info!("Creating task: {}", payload.task_name);
    let task = task_service.create_task(payload).await.map_err(|e| {
        tracing::error!("Create task failed: {:?}", e);
        TaskAppError::from(e)
    })?;
    Ok((StatusCode::CREATED, Json(task)))
}

// ─── Get All Tasks ───────────────────────────────────────────────────────────

/// Menampilkan semua tugas
#[utoipa::path(
    get,
    path = "/api/tasks",
    responses(
        (status = 200, description = "Daftar semua task", body = Vec<TaskResponse>),
        (status = 401, description = "Unauthorized - token tidak valid", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    security(("bearer_auth" = [])),
    tag = "tasks"
)]
#[tracing::instrument(skip(task_service, _auth))]
pub async fn get_all_tasks(
    _auth: AuthUser,
    State(task_service): State<Arc<TaskService>>,
) -> Result<impl IntoResponse, TaskAppError> {
    tracing::info!("Retrieving all tasks");
    let tasks = task_service.get_all_tasks().await.map_err(|e| {
        tracing::error!("Retrieve all tasks failed: {:?}", e);
        TaskAppError::from(e)
    })?;
    Ok(Json(tasks))
}

// ─── Get Task By ID ──────────────────────────────────────────────────────────

/// Mendapatkan tugas berdasarkan ID
#[utoipa::path(
    get,
    path = "/api/tasks/{id}",
    params(
        ("id" = Uuid, Path, description = "Task ID")
    ),
    responses(
        (status = 200, description = "Task ditemukan", body = TaskResponse),
        (status = 401, description = "Unauthorized - token tidak valid", body = ErrorResponse),
        (status = 404, description = "Task tidak ditemukan", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    security(("bearer_auth" = [])),
    tag = "tasks"
)]
#[tracing::instrument(skip(task_service, _auth))]
pub async fn get_task_by_id(
    _auth: AuthUser,
    State(task_service): State<Arc<TaskService>>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, TaskAppError> {
    tracing::info!("Retrieving task by ID: {}", id);
    let task = task_service.get_task_by_id(id).await.map_err(|e| {
        tracing::error!("Retrieve task by ID failed: {:?}", e);
        TaskAppError::from(e)
    })?;
    Ok(Json(task))
}

// ─── Get Tasks By User ───────────────────────────────────────────────────────

/// Mendapatkan semua tugas milik user tertentu
#[utoipa::path(
    get,
    path = "/api/tasks/user/{id_user}",
    params(
        ("id_user" = Uuid, Path, description = "User ID untuk filter task")
    ),
    responses(
        (status = 200, description = "Daftar task milik user", body = Vec<TaskResponse>),
        (status = 401, description = "Unauthorized - token tidak valid", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    security(("bearer_auth" = [])),
    tag = "tasks"
)]
pub async fn get_tasks_by_user(
    _auth: AuthUser,
    State(task_service): State<Arc<TaskService>>,
    Path(id_user): Path<Uuid>,
) -> Result<impl IntoResponse, TaskAppError> {
    let tasks = task_service
        .get_tasks_by_user(id_user)
        .await
        .map_err(TaskAppError::from)?;
    Ok(Json(tasks))
}

// ─── Update Task ─────────────────────────────────────────────────────────────

/// Memperbarui informasi tugas
#[utoipa::path(
    put,
    path = "/api/tasks/{id}",
    params(
        ("id" = Uuid, Path, description = "Task ID")
    ),
    request_body = UpdateTask,
    responses(
        (status = 200, description = "Task berhasil diupdate", body = TaskResponse),
        (status = 401, description = "Unauthorized - token tidak valid", body = ErrorResponse),
        (status = 404, description = "Task tidak ditemukan", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    security(("bearer_auth" = [])),
    tag = "tasks"
)]
#[tracing::instrument(skip(task_service, _auth, payload))]
pub async fn update_task(
    _auth: AuthUser,
    State(task_service): State<Arc<TaskService>>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateTask>,
) -> Result<impl IntoResponse, TaskAppError> {
    tracing::info!("Updating task ID: {}", id);
    let task = task_service.update_task(id, payload).await.map_err(|e| {
        tracing::error!("Update task ID failed: {:?}", e);
        TaskAppError::from(e)
    })?;
    Ok(Json(task))
}

// ─── Delete Task ─────────────────────────────────────────────────────────────

/// Menghapus tugas berdasarkan ID
#[utoipa::path(
    delete,
    path = "/api/tasks/{id}",
    params(
        ("id" = Uuid, Path, description = "Task ID")
    ),
    responses(
        (status = 200, description = "Task berhasil dihapus"),
        (status = 401, description = "Unauthorized - token tidak valid", body = ErrorResponse),
        (status = 404, description = "Task tidak ditemukan", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    security(("bearer_auth" = [])),
    tag = "tasks"
)]
#[tracing::instrument(skip(task_service, _auth))]
pub async fn delete_task(
    _auth: AuthUser,
    State(task_service): State<Arc<TaskService>>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, TaskAppError> {
    tracing::info!("Deleting task ID: {}", id);
    task_service.delete_task(id).await.map_err(|e| {
        tracing::error!("Delete task ID failed: {:?}", e);
        TaskAppError::from(e)
    })?;
    Ok(Json(json!({ "message": "Task berhasil dihapus" })))
}

// ─── Error Handling ──────────────────────────────────────────────────────────

pub struct TaskAppError(TaskError);

impl From<TaskError> for TaskAppError {
    fn from(inner: TaskError) -> Self {
        TaskAppError(inner)
    }
}

impl IntoResponse for TaskAppError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self.0 {
            TaskError::DatabaseError(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Database error"),
            TaskError::TaskNotFound => (StatusCode::NOT_FOUND, "Task tidak ditemukan"),
            TaskError::Unauthorized => (StatusCode::UNAUTHORIZED, "Unauthorized"),
        };

        (status, Json(json!({ "error": error_message }))).into_response()
    }
}
