use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, FromRow, ToSchema)]
pub struct ProgressOption {
    pub id: Uuid,
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize, FromRow, ToSchema)]
pub struct RoleOption {
    pub id: Uuid,
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize, FromRow, ToSchema)]
pub struct MenuOption {
    pub id: Uuid,
    pub name: String,
}

#[async_trait::async_trait]
pub trait MasterRepository: Send + Sync {
    async fn find_all_progress_options(&self) -> Result<Vec<ProgressOption>, sqlx::Error>;
    async fn find_all_role_options(&self) -> Result<Vec<RoleOption>, sqlx::Error>;
    async fn find_all_menu_options(&self) -> Result<Vec<MenuOption>, sqlx::Error>;
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
