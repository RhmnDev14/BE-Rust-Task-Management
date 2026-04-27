use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::{IntoParams, ToSchema};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, FromRow, ToSchema)]
pub struct Task {
    pub id: Uuid,
    pub task_name: String,
    pub description: Option<String>,
    pub story_point: Option<i32>,
    pub id_user: Uuid,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CreateTask {
    pub task_name: String,
    pub description: Option<String>,
    pub story_point: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct UpdateTask {
    pub task_name: Option<String>,
    pub description: Option<String>,
    pub story_point: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct TaskResponse {
    pub id: Uuid,
    pub task_name: String,
    pub description: Option<String>,
    pub story_point: Option<i32>,
    pub id_user: Uuid,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema, IntoParams)]
pub struct PaginationParams {
    pub page: Option<i64>,
    pub limit: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct PaginatedResponse<T> {
    pub items: Vec<T>,
    pub total_items: i64,
    pub page: i64,
    pub limit: i64,
    pub total_pages: i64,
}

impl Default for PaginationParams {
    fn default() -> Self {
        Self {
            page: Some(1),
            limit: Some(10),
        }
    }
}

#[async_trait::async_trait]
pub trait TaskRepository: Send + Sync {
    async fn create(&self, task: &CreateTask, id_user: Uuid) -> Result<Task, sqlx::Error>;
    async fn find_all(&self, pagination: &PaginationParams) -> Result<(Vec<Task>, i64), sqlx::Error>;
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Task>, sqlx::Error>;
    async fn find_by_user_id(
        &self,
        id_user: Uuid,
        pagination: &PaginationParams,
    ) -> Result<(Vec<Task>, i64), sqlx::Error>;
    async fn search(
        &self,
        id_user: Uuid,
        query: &str,
        pagination: &PaginationParams,
    ) -> Result<(Vec<Task>, i64), sqlx::Error>;
    async fn update(&self, id: Uuid, task: &UpdateTask) -> Result<Option<Task>, sqlx::Error>;
    async fn delete(&self, id: Uuid) -> Result<bool, sqlx::Error>;
}

#[derive(Debug)]
pub enum TaskError {
    DatabaseError(#[allow(dead_code)] sqlx::Error),
    TaskNotFound,
    Unauthorized,
}

impl From<sqlx::Error> for TaskError {
    fn from(err: sqlx::Error) -> Self {
        TaskError::DatabaseError(err)
    }
}
