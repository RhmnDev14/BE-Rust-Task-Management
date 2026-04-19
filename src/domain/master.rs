use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, FromRow, ToSchema)]
pub struct ProgressOption {
    pub id: Uuid,
    pub name: String,
}

#[async_trait::async_trait]
pub trait MasterRepository: Send + Sync {
    async fn find_all_progress_options(&self) -> Result<Vec<ProgressOption>, sqlx::Error>;
}

#[derive(Debug)]
pub enum MasterError {
    DatabaseError(sqlx::Error),
}

impl From<sqlx::Error> for MasterError {
    fn from(err: sqlx::Error) -> Self {
        MasterError::DatabaseError(err)
    }
}
