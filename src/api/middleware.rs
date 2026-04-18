use axum::{
    async_trait,
    extract::FromRequestParts,
    http::{request::Parts, StatusCode},
    response::{IntoResponse, Json, Response},
};
use jsonwebtoken::{decode, DecodingKey, Validation};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::env;
use uuid::Uuid;

// ─── JWT Claims ───────────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub id: String, // user_id explicitly
    pub exp: usize,
    pub iat: usize,
}

// ─── Auth Extractor ───────────────────────────────────────────────────────────

/// Extractor ini dipakai sebagai parameter di handler.
/// Jika token tidak ada / invalid, request langsung ditolak 401.
pub struct AuthUser {
    pub user_id: Uuid,
}

#[async_trait]
impl<S> FromRequestParts<S> for AuthUser
where
    S: Send + Sync,
{
    type Rejection = AuthError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        // 1. Ambil header Authorization
        let auth_header = parts
            .headers
            .get("Authorization")
            .and_then(|v| v.to_str().ok())
            .ok_or(AuthError::MissingToken)?;

        // 2. Pastikan formatnya "Bearer <token>"
        let token = auth_header
            .strip_prefix("Bearer ")
            .ok_or(AuthError::InvalidFormat)?;

        // 3. Baca JWT_SECRET dari environment
        let secret = env::var("JWT_SECRET").unwrap_or_else(|_| "supersecretkey".to_string());

        // 4. Decode & validasi token
        let token_data = decode::<Claims>(
            token,
            &DecodingKey::from_secret(secret.as_bytes()),
            &Validation::default(),
        )
        .map_err(|e| match e.kind() {
            jsonwebtoken::errors::ErrorKind::ExpiredSignature => AuthError::TokenExpired,
            _ => AuthError::InvalidToken,
        })?;

        // 5. Parse sub (user_id) sebagai UUID
        let user_id =
            Uuid::parse_str(&token_data.claims.id).map_err(|_| AuthError::InvalidToken)?;

        Ok(AuthUser { user_id })
    }
}

// ─── Auth Error ───────────────────────────────────────────────────────────────

pub enum AuthError {
    MissingToken,
    InvalidFormat,
    InvalidToken,
    TokenExpired,
}

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            AuthError::MissingToken => (
                StatusCode::UNAUTHORIZED,
                "Authorization header tidak ditemukan",
            ),
            AuthError::InvalidFormat => (
                StatusCode::UNAUTHORIZED,
                "Format Authorization tidak valid, gunakan: Bearer <token>",
            ),
            AuthError::InvalidToken => (StatusCode::UNAUTHORIZED, "Token tidak valid"),
            AuthError::TokenExpired => (StatusCode::UNAUTHORIZED, "Token sudah kadaluarsa"),
        };

        (status, Json(json!({ "error": message }))).into_response()
    }
}
