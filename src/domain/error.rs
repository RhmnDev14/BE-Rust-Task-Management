use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Serialize, Deserialize, ToSchema)]
pub struct ErrorResponse {
    /// Pesan error yang informatif
    pub error: String,
}
