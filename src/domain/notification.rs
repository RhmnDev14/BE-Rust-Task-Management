use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, FromRow, ToSchema)]
pub struct Notification {
    pub id: Uuid,
    pub user_id: Uuid,
    pub title: String,
    pub body: String,
    pub data: Option<String>,
    pub read_at: Option<DateTime<Utc>>,
    pub created_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct NotificationResponse {
    pub id: Uuid,
    pub title: String,
    pub body: String,
    pub data: Option<String>,
    pub read_at: Option<DateTime<Utc>>,
    pub created_at: Option<DateTime<Utc>>,
}

impl From<Notification> for NotificationResponse {
    fn from(n: Notification) -> Self {
        Self {
            id: n.id,
            title: n.title,
            body: n.body,
            data: n.data,
            read_at: n.read_at,
            created_at: n.created_at,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct MarkAsReadRequest {
    pub ids: Vec<Uuid>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct UnreadCountResponse {
    pub count: i64,
}

#[async_trait::async_trait]
pub trait NotificationRepository: Send + Sync {
    async fn find_by_user_id(&self, user_id: Uuid) -> Result<Vec<Notification>, sqlx::Error>;
    async fn mark_as_read(&self, user_id: Uuid, ids: &[Uuid]) -> Result<(), sqlx::Error>;
    async fn get_unread_count(&self, user_id: Uuid) -> Result<i64, sqlx::Error>;
}
