#![allow(unused_imports)]
use crate::{
    application::user_service::UserService,
    domain::error::ErrorResponse,
    domain::user::{CreateUser, LoginUser, UserError},
};
use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use std::sync::Arc;

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
