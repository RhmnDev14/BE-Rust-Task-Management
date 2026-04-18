use crate::infrastructure::s3::S3Client;
use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::Arc;
use utoipa::ToSchema;

#[derive(Deserialize, ToSchema)]
pub struct PresignedUrlRequest {
    /// Nama file yang akan diupload (termasuk ekstensi)
    #[schema(example = "avatar.png")]
    pub file_name: String,
}

#[derive(Serialize, ToSchema)]
pub struct PresignedUrlResponse {
    /// URL presigned untuk upload file menggunakan metode PUT
    pub upload_url: String,
}

/// Menghasilkan URL presigned untuk upload file ke S3/MinIO
#[utoipa::path(
    post,
    path = "/api/s3/presigned-url",
    tag = "s3",
    request_body = PresignedUrlRequest,
    responses(
        (status = 200, description = "Presigned URL generated successfully", body = PresignedUrlResponse),
        (status = 500, description = "Internal server error")
    ),
    security(("bearer_auth" = []))
)]
pub async fn get_presigned_url(
    State(s3_client): State<Arc<S3Client>>,
    Json(payload): Json<PresignedUrlRequest>,
) -> impl IntoResponse {
    match s3_client.generate_presigned_url(&payload.file_name, 3600).await { // Valid for 1 hour
        Ok(url) => (StatusCode::OK, Json(json!({ "upload_url": url }))).into_response(),
        Err(e) => {
            tracing::error!("Failed to generate presigned URL: {:?}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": "Internal server error" }))).into_response()
        }
    }
}
