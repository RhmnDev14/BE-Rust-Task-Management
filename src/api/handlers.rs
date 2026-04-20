#![allow(unused_imports)]
use crate::{
    application::user_service::UserService,
    domain::error::ErrorResponse,
    domain::user::{
        ChangePassword, CreateUser, LoginUser, MessageResponse, UpdateUser, UpdateUserRoleRequest,
        UserError, UserOption, UserResponse,
    },
};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use std::sync::Arc;
use crate::api::middleware::AuthUser;

/// Mendaftarkan user baru
#[utoipa::path(
    post,
    path = "/api/auth/register",
    tag = "auth",
    request_body = CreateUser,
    responses(
        (status = 201, description = "User registered successfully", body = UserResponse),
        (status = 409, description = "User already exists", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    )
)]
#[tracing::instrument(skip_all, fields(username = %payload.username, email = %payload.email))]
pub async fn register_user(
    State(user_service): State<Arc<UserService>>,
    Json(payload): Json<CreateUser>,
) -> Result<impl IntoResponse, AppError> {
    tracing::info!("Registering user: {}", payload.username);
    // Using AppError for cleaner error handling
    let user_response = user_service.register(payload).await.map_err(|e| {
        tracing::error!("Registration failed: {:?}", e);
        AppError::from(e)
    })?;

    Ok((StatusCode::CREATED, Json(user_response)))
}

/// Login user dan mendapatkan token JWT
#[utoipa::path(
    post,
    path = "/api/auth/login",
    tag = "auth",
    request_body = LoginUser,
    responses(
        (status = 200, description = "Login successful", body = String),
        (status = 401, description = "Email or password does not match", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    )
)]
#[tracing::instrument(skip_all, fields(email = %payload.email))]
pub async fn login_user(
    State(user_service): State<Arc<UserService>>,
    Json(payload): Json<LoginUser>,
) -> Result<impl IntoResponse, AppError> {
    tracing::info!("Login attempt for user: {}", payload.email);
    let token = user_service.login(payload).await.map_err(|e| {
        tracing::error!("Login failed: {:?}", e);
        AppError::from(e)
    })?;

    Ok(Json(json!({ "token": token })))
}

/// Mendapatkan profil user yang sedang login
#[utoipa::path(
    get,
    path = "/api/auth/me",
    tag = "auth",
    responses(
        (status = 200, description = "User profile retrieved successfully", body = UserResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    security(("bearer_auth" = []))
)]
#[tracing::instrument(skip_all, fields(user_id = %auth.user_id))]
pub async fn get_me(
    auth: AuthUser,
    State(user_service): State<Arc<UserService>>,
) -> Result<impl IntoResponse, AppError> {
    tracing::info!("Retrieving profile for user: {}", auth.user_id);
    let user_response = user_service
        .get_user_by_id(auth.user_id)
        .await
        .map_err(|e| {
            tracing::error!("Profile retrieval failed: {:?}", e);
            AppError::from(e)
        })?;

    Ok(Json(user_response))
}

/// Memperbarui profil user yang sedang login
#[utoipa::path(
    put,
    path = "/api/auth/me",
    tag = "auth",
    request_body = UpdateUser,
    responses(
        (status = 200, description = "User profile updated successfully", body = UserResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    security(("bearer_auth" = []))
)]
#[tracing::instrument(skip_all, fields(user_id = %auth.user_id))]
pub async fn update_me(
    auth: AuthUser,
    State(user_service): State<Arc<UserService>>,
    Json(payload): Json<UpdateUser>,
) -> Result<impl IntoResponse, AppError> {
    tracing::info!("Updating profile for user: {}", auth.user_id);
    let user_response = user_service
        .update_profile(auth.user_id, payload)
        .await
        .map_err(|e| {
            tracing::error!("Profile update failed: {:?}", e);
            AppError::from(e)
        })?;

    Ok(Json(user_response))
}

/// Mengubah password user yang sedang login
#[utoipa::path(
    put,
    path = "/api/auth/change-password",
    tag = "auth",
    request_body = ChangePassword,
    responses(
        (status = 200, description = "Password changed successfully", body = MessageResponse),
        (status = 401, description = "Current password is incorrect", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    security(("bearer_auth" = []))
)]
#[tracing::instrument(skip_all, fields(user_id = %auth.user_id))]
pub async fn change_password(
    auth: AuthUser,
    State(user_service): State<Arc<UserService>>,
    Json(payload): Json<ChangePassword>,
) -> Result<impl IntoResponse, AppError> {
    tracing::info!("Changing password for user: {}", auth.user_id);
    user_service
        .change_password(auth.user_id, payload)
        .await
        .map_err(|e| {
            tracing::error!("Password change failed: {:?}", e);
            AppError::from(e)
        })?;

    Ok(Json(MessageResponse { message: "Password berhasil diubah".to_string() }))
}

/// Mendapatkan daftar user untuk pilihan (options)
#[utoipa::path(
    get,
    path = "/api/users/options",
    tag = "users",
    responses(
        (status = 200, description = "Daftar user options berhasil diambil", body = Vec<UserOption>),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    security(("bearer_auth" = []))
)]
#[tracing::instrument(skip_all)]
pub async fn get_user_options(
    _auth: AuthUser,
    State(user_service): State<Arc<UserService>>,
) -> Result<impl IntoResponse, AppError> {
    tracing::info!("Retrieving user options");
    let options = user_service.get_user_options().await.map_err(|e| {
        tracing::error!("User options retrieval failed: {:?}", e);
        AppError::from(e)
    })?;

    Ok(Json(options))
}

/// Memperbarui role user
#[utoipa::path(
    put,
    path = "/api/users/{id}/role",
    tag = "users",
    params(
        ("id" = Uuid, Path, description = "User ID")
    ),
    request_body = UpdateUserRoleRequest,
    responses(
        (status = 200, description = "User role updated successfully", body = MessageResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    security(("bearer_auth" = []))
)]
#[tracing::instrument(skip(user_service, _auth, id, payload))]
pub async fn update_user_role(
    _auth: AuthUser,
    Path(id): Path<uuid::Uuid>,
    State(user_service): State<Arc<UserService>>,
    Json(payload): Json<UpdateUserRoleRequest>,
) -> Result<impl IntoResponse, AppError> {
    tracing::info!("Updating role for user: {}", id);
    user_service
        .update_user_role(id, payload.role_id)
        .await
        .map_err(|e| {
            tracing::error!("User role update failed: {:?}", e);
            AppError::from(e)
        })?;

    Ok(Json(MessageResponse {
        message: "Role user berhasil diperbarui".to_string(),
    }))
}

// Simple Error wrapper for Axum response
pub struct AppError(UserError);

impl From<UserError> for AppError {
    fn from(inner: UserError) -> Self {
        AppError(inner)
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self.0 {
            UserError::DatabaseError(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Database error"),
            UserError::UserAlreadyExists => (StatusCode::CONFLICT, "User already exists"),
            UserError::UserNotFound => {
                (StatusCode::UNAUTHORIZED, "email or password does not match")
            }
            UserError::InvalidCredentials => {
                (StatusCode::UNAUTHORIZED, "email or password does not match")
            }
            UserError::TokenCreationError => {
                (StatusCode::INTERNAL_SERVER_ERROR, "Token creation error")
            }
        };

        let body = Json(json!({
            "error": error_message,
        }));

        (status, body).into_response()
    }
}
