use crate::{
    api::middleware::AuthUser,
    application::notification_service::NotificationService,
    domain::notification::{MarkAsReadRequest, UnreadCountResponse},
    domain::user::MessageResponse,
};
use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use std::sync::Arc;

/// Mendapatkan semua notifikasi milik user yang sedang login
#[utoipa::path(
    get,
    path = "/api/notifications",
    responses(
        (status = 200, description = "Daftar notifikasi milik user", body = [NotificationResponse]),
        (status = 401, description = "Unauthorized - token tidak valid", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    security(("bearer_auth" = [])),
    tag = "notifications"
)]
pub async fn get_user_notifications(
    auth: AuthUser,
    State(notification_service): State<Arc<NotificationService>>,
) -> Result<impl IntoResponse, NotificationAppError> {
    let notifications = notification_service
        .get_user_notifications(auth.user_id)
        .await
        .map_err(NotificationAppError::from)?;
    Ok(Json(notifications))
}

/// Menandai beberapa notifikasi sebagai sudah dibaca
#[utoipa::path(
    put,
    path = "/api/notifications/read",
    request_body = MarkAsReadRequest,
    responses(
        (status = 200, description = "Berhasil menandai notifikasi sebagai sudah dibaca", body = MessageResponse),
        (status = 401, description = "Unauthorized - token tidak valid", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    security(("bearer_auth" = [])),
    tag = "notifications"
)]
pub async fn mark_as_read(
    auth: AuthUser,
    State(notification_service): State<Arc<NotificationService>>,
    Json(payload): Json<MarkAsReadRequest>,
) -> Result<impl IntoResponse, NotificationAppError> {
    notification_service
        .mark_as_read(auth.user_id, &payload.ids)
        .await
        .map_err(NotificationAppError::from)?;
    Ok(Json(MessageResponse {
        message: "Notifikasi berhasil ditandai sebagai sudah dibaca".to_string(),
    }))
}

/// Mendapatkan jumlah notifikasi yang belum dibaca
#[utoipa::path(
    get,
    path = "/api/notifications/unread-count",
    responses(
        (status = 200, description = "Jumlah notifikasi belum dibaca", body = UnreadCountResponse),
        (status = 401, description = "Unauthorized - token tidak valid", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    security(("bearer_auth" = [])),
    tag = "notifications"
)]
pub async fn get_unread_count(
    auth: AuthUser,
    State(notification_service): State<Arc<NotificationService>>,
) -> Result<impl IntoResponse, NotificationAppError> {
    let count = notification_service
        .get_unread_count(auth.user_id)
        .await
        .map_err(NotificationAppError::from)?;
    Ok(Json(UnreadCountResponse { count }))
}

pub struct NotificationAppError(#[allow(dead_code)] sqlx::Error);

impl From<sqlx::Error> for NotificationAppError {
    fn from(inner: sqlx::Error) -> Self {
        NotificationAppError(inner)
    }
}

impl IntoResponse for NotificationAppError {
    fn into_response(self) -> Response {
        let status = StatusCode::INTERNAL_SERVER_ERROR;
        let error_message = "Database error";
        (status, Json(json!({ "error": error_message }))).into_response()
    }
}
