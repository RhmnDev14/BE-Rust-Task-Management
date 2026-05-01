use crate::domain::notification::{Notification, NotificationRepository};
use sqlx::PgPool;
use uuid::Uuid;

pub struct PostgresNotificationRepository {
    pool: PgPool,
}

impl PostgresNotificationRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl NotificationRepository for PostgresNotificationRepository {
    async fn find_by_user_id(&self, user_id: Uuid) -> Result<Vec<Notification>, sqlx::Error> {
        let notifications = sqlx::query_as::<_, Notification>(
            "SELECT id, user_id, title, body, data, read_at, created_at FROM notifications WHERE user_id = $1 ORDER BY created_at DESC"
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(notifications)
    }

    async fn mark_as_read(&self, user_id: Uuid, ids: &[Uuid]) -> Result<(), sqlx::Error> {
        if ids.is_empty() {
            return Ok(());
        }

        sqlx::query(
            "UPDATE notifications SET read_at = CURRENT_TIMESTAMP WHERE user_id = $1 AND id = ANY($2)"
        )
        .bind(user_id)
        .bind(ids)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn get_unread_count(&self, user_id: Uuid) -> Result<i64, sqlx::Error> {
        let count = sqlx::query!(
            "SELECT COUNT(*) as count FROM notifications WHERE user_id = $1 AND read_at IS NULL",
            user_id
        )
        .fetch_one(&self.pool)
        .await?
        .count
        .unwrap_or(0);

        Ok(count)
    }
}
