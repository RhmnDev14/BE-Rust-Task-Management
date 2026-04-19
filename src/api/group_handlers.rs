#![allow(unused_imports)]
use crate::{
    api::middleware::AuthUser,
    application::group_service::GroupService,
    domain::error::ErrorResponse,
    domain::group::{CreateGroup, GroupError, GroupMember, GroupResponse, UpdateGroup},
    domain::task::{PaginatedResponse, PaginationParams},
    domain::user::MessageResponse,
};
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use std::sync::Arc;
use uuid::Uuid;

// ─── Create Group ────────────────────────────────────────────────────────────

/// Membuat grup baru
#[utoipa::path(
    post,
    path = "/api/groups",
    request_body = CreateGroup,
    responses(
        (status = 201, description = "Group berhasil dibuat", body = GroupResponse),
        (status = 401, description = "Unauthorized - token tidak valid", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    security(("bearer_auth" = [])),
    tag = "groups"
)]
#[tracing::instrument(skip(group_service, auth, payload))]
pub async fn create_group(
    auth: AuthUser,
    State(group_service): State<Arc<GroupService>>,
    Json(payload): Json<CreateGroup>,
) -> Result<impl IntoResponse, GroupAppError> {
    tracing::info!("Creating group: {}", payload.name);
    let group = group_service.create_group(payload, auth.user_id).await.map_err(|e| {
        tracing::error!("Create group failed: {:?}", e);
        GroupAppError::from(e)
    })?;
    Ok((StatusCode::CREATED, Json(group)))
}

// ─── Get All Groups ──────────────────────────────────────────────────────────

/// Menampilkan semua grup
#[utoipa::path(
    get,
    path = "/api/groups",
    params(
        PaginationParams
    ),
    responses(
        (status = 200, description = "Daftar semua grup dengan paginasi", body = PaginatedResponse<GroupResponse>),
        (status = 401, description = "Unauthorized - token tidak valid", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    security(("bearer_auth" = [])),
    tag = "groups"
)]
#[tracing::instrument(skip(group_service, _auth, pagination))]
pub async fn get_all_groups(
    _auth: AuthUser,
    State(group_service): State<Arc<GroupService>>,
    Query(pagination): Query<PaginationParams>,
) -> Result<impl IntoResponse, GroupAppError> {
    tracing::info!("Retrieving all groups with pagination: {:?}", pagination);
    let groups = group_service.get_all_groups(pagination).await.map_err(|e| {
        tracing::error!("Retrieve all groups failed: {:?}", e);
        GroupAppError::from(e)
    })?;
    Ok(Json(groups))
}

// ─── Get Group By ID ─────────────────────────────────────────────────────────

/// Mendapatkan grup berdasarkan ID
#[utoipa::path(
    get,
    path = "/api/groups/{id}",
    params(
        ("id" = Uuid, Path, description = "Group ID")
    ),
    responses(
        (status = 200, description = "Group ditemukan", body = GroupResponse),
        (status = 401, description = "Unauthorized - token tidak valid", body = ErrorResponse),
        (status = 404, description = "Group tidak ditemukan", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    security(("bearer_auth" = [])),
    tag = "groups"
)]
#[tracing::instrument(skip(group_service, _auth))]
pub async fn get_group_by_id(
    _auth: AuthUser,
    State(group_service): State<Arc<GroupService>>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, GroupAppError> {
    tracing::info!("Retrieving group by ID: {}", id);
    let group = group_service.get_group_by_id(id).await.map_err(|e| {
        tracing::error!("Retrieve group by ID failed: {:?}", e);
        GroupAppError::from(e)
    })?;
    Ok(Json(group))
}

// ─── Get Group Members ───────────────────────────────────────────────────────

/// Mendapatkan daftar anggota grup
#[utoipa::path(
    get,
    path = "/api/groups/{id}/members",
    params(
        ("id" = Uuid, Path, description = "Group ID")
    ),
    responses(
        (status = 200, description = "Daftar anggota grup", body = Vec<GroupMember>),
        (status = 401, description = "Unauthorized - token tidak valid", body = ErrorResponse),
        (status = 404, description = "Group tidak ditemukan", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    security(("bearer_auth" = [])),
    tag = "groups"
)]
#[tracing::instrument(skip(group_service, _auth))]
pub async fn get_group_members(
    _auth: AuthUser,
    State(group_service): State<Arc<GroupService>>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, GroupAppError> {
    tracing::info!("Retrieving members for group ID: {}", id);
    let members = group_service.get_group_members(id).await.map_err(|e| {
        tracing::error!("Retrieve group members failed: {:?}", e);
        GroupAppError::from(e)
    })?;
    Ok(Json(members))
}

// ─── Update Group ────────────────────────────────────────────────────────────

/// Memperbarui informasi grup
#[utoipa::path(
    put,
    path = "/api/groups/{id}",
    params(
        ("id" = Uuid, Path, description = "Group ID")
    ),
    request_body = UpdateGroup,
    responses(
        (status = 200, description = "Group berhasil diupdate", body = GroupResponse),
        (status = 401, description = "Unauthorized - token tidak valid", body = ErrorResponse),
        (status = 404, description = "Group tidak ditemukan", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    security(("bearer_auth" = [])),
    tag = "groups"
)]
#[tracing::instrument(skip(group_service, auth, payload))]
pub async fn update_group(
    auth: AuthUser,
    State(group_service): State<Arc<GroupService>>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateGroup>,
) -> Result<impl IntoResponse, GroupAppError> {
    tracing::info!("Updating group ID: {}", id);
    let group = group_service.update_group(id, payload, auth.user_id).await.map_err(|e| {
        tracing::error!("Update group ID failed: {:?}", e);
        GroupAppError::from(e)
    })?;
    Ok(Json(group))
}

// ─── Delete Group ────────────────────────────────────────────────────────────

/// Menghapus grup berdasarkan ID
#[utoipa::path(
    delete,
    path = "/api/groups/{id}",
    params(
        ("id" = Uuid, Path, description = "Group ID")
    ),
    responses(
        (status = 200, description = "Group berhasil dihapus", body = MessageResponse),
        (status = 401, description = "Unauthorized - token tidak valid", body = ErrorResponse),
        (status = 404, description = "Group tidak ditemukan", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    security(("bearer_auth" = [])),
    tag = "groups"
)]
#[tracing::instrument(skip(group_service, _auth))]
pub async fn delete_group(
    _auth: AuthUser,
    State(group_service): State<Arc<GroupService>>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, GroupAppError> {
    tracing::info!("Deleting group ID: {}", id);
    group_service.delete_group(id).await.map_err(|e| {
        tracing::error!("Delete group ID failed: {:?}", e);
        GroupAppError::from(e)
    })?;
    Ok(Json(MessageResponse { message: "Group berhasil dihapus".to_string() }))
}

// ─── Error Handling ──────────────────────────────────────────────────────────

pub struct GroupAppError(GroupError);

impl From<GroupError> for GroupAppError {
    fn from(inner: GroupError) -> Self {
        GroupAppError(inner)
    }
}

impl IntoResponse for GroupAppError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self.0 {
            GroupError::DatabaseError(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Database error"),
            GroupError::GroupNotFound => (StatusCode::NOT_FOUND, "Group tidak ditemukan"),
            GroupError::Unauthorized => (StatusCode::UNAUTHORIZED, "Unauthorized"),
        };

        (status, Json(json!({ "error": error_message }))).into_response()
    }
}
