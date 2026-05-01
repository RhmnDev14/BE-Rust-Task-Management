use crate::domain::notification::{NotificationRepository, NotificationResponse};
use uuid::Uuid;

pub struct NotificationService {
    repository: Box<dyn NotificationRepository>,
}

impl NotificationService {
    pub fn new(repository: Box<dyn NotificationRepository>) -> Self {
        Self { repository }
    }

    pub async fn get_user_notifications(&self, user_id: Uuid) -> Result<Vec<NotificationResponse>, sqlx::Error> {
        let notifications = self.repository.find_by_user_id(user_id).await?;
        let responses = notifications.into_iter().map(Into::into).collect();
        Ok(responses)
    }

    pub async fn mark_as_read(&self, user_id: Uuid, ids: &[Uuid]) -> Result<(), sqlx::Error> {
        self.repository.mark_as_read(user_id, ids).await
    }

    pub async fn get_unread_count(&self, user_id: Uuid) -> Result<i64, sqlx::Error> {
        self.repository.get_unread_count(user_id).await
    }
}
