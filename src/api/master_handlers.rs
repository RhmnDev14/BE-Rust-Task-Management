#![allow(unused_imports)]
use crate::{
    api::middleware::AuthUser,
    application::master_service::MasterService,
    domain::error::ErrorResponse,
    domain::master::{MasterError, ProgressOption},
};
use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use std::sync::Arc;

// ─── Get Progress Options ────────────────────────────────────────────────────

/// Mendapatkan daftar pilihan progress (master_progress)
#[utoipa::path(
    get,
    path = "/api/master/progress",
    responses(
        (status = 200, description = "Daftar progress options berhasil diambil", body = Vec<ProgressOption>),
        (status = 401, description = "Unauthorized - token tidak valid", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    security(("bearer_auth" = [])),
    tag = "master"
)]
#[tracing::instrument(skip(master_service, _auth))]
pub async fn get_progress_options(
    _auth: AuthUser,
    State(master_service): State<Arc<MasterService>>,
) -> Result<impl IntoResponse, MasterAppError> {
    tracing::info!("Retrieving progress options");
    let options = master_service.get_progress_options().await.map_err(|e| {
        tracing::error!("Progress options retrieval failed: {:?}", e);
        MasterAppError::from(e)
    })?;
    Ok(Json(options))
}

// ─── Error Handling ──────────────────────────────────────────────────────────

pub struct MasterAppError(MasterError);

impl From<MasterError> for MasterAppError {
    fn from(inner: MasterError) -> Self {
        MasterAppError(inner)
    }
}

impl IntoResponse for MasterAppError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self.0 {
            MasterError::DatabaseError(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Database error"),
        };

        (status, Json(json!({ "error": error_message }))).into_response()
    }
}
