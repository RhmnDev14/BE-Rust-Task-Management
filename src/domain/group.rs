use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;
use uuid::Uuid;
use crate::domain::task::PaginationParams;

#[derive(Debug, Serialize, Deserialize, FromRow, ToSchema)]
pub struct Group {
    pub id: Uuid,
    pub name: String,
    pub created_at: Option<DateTime<Utc>>,
    pub created_by: Uuid,
    pub updated_at: Option<DateTime<Utc>>,
    pub updated_by: Uuid,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CreateGroup {
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct UpdateGroup {
    pub name: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct GroupResponse {
    pub id: Uuid,
    pub name: String,
    pub created_at: Option<DateTime<Utc>>,
    pub created_by: Uuid,
    pub updated_at: Option<DateTime<Utc>>,
    pub updated_by: Uuid,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct GroupMember {
    pub user_id: Uuid,
    pub username: String,
}

#[async_trait::async_trait]
pub trait GroupRepository: Send + Sync {
    async fn create(&self, group: &CreateGroup, user_id: Uuid) -> Result<Group, sqlx::Error>;
    async fn find_all(&self, pagination: &PaginationParams) -> Result<(Vec<Group>, i64), sqlx::Error>;
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Group>, sqlx::Error>;
    async fn find_users_by_group_id(&self, group_id: Uuid) -> Result<Vec<GroupMember>, sqlx::Error>;
    async fn update(&self, id: Uuid, group: &UpdateGroup, user_id: Uuid) -> Result<Option<Group>, sqlx::Error>;
    async fn delete(&self, id: Uuid) -> Result<bool, sqlx::Error>;
}

#[derive(Debug)]
pub enum GroupError {
    DatabaseError(sqlx::Error),
    GroupNotFound,
    Unauthorized,
}

impl From<sqlx::Error> for GroupError {
    fn from(err: sqlx::Error) -> Self {
        GroupError::DatabaseError(err)
    }
}
