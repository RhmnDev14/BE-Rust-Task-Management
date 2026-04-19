use crate::domain::group::{CreateGroup, Group, GroupRepository, UpdateGroup};
use crate::domain::task::PaginationParams;
use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

pub struct SqlxGroupRepository {
    pool: PgPool,
}

impl SqlxGroupRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl GroupRepository for SqlxGroupRepository {
    async fn create(&self, group: &CreateGroup, user_id: Uuid) -> Result<Group, sqlx::Error> {
        let group = sqlx::query_as!(
            Group,
            r#"
            INSERT INTO groups (name, created_by, updated_by)
            VALUES ($1, $2, $3)
            RETURNING id, name, created_at, created_by, updated_at, updated_by
            "#,
            group.name,
            user_id,
            user_id
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(group)
    }

    async fn find_all(&self, pagination: &PaginationParams) -> Result<(Vec<Group>, i64), sqlx::Error> {
        let limit = pagination.limit.unwrap_or(10);
        let offset = (pagination.page.unwrap_or(1) - 1) * limit;

        let total_items = sqlx::query!(
            r#"SELECT COUNT(*) as count FROM groups"#
        )
        .fetch_one(&self.pool)
        .await?
        .count
        .unwrap_or(0);

        let groups = sqlx::query_as!(
            Group,
            r#"
            SELECT id, name, created_at, created_by, updated_at, updated_by
            FROM groups
            ORDER BY created_at DESC
            LIMIT $1 OFFSET $2
            "#,
            limit,
            offset
        )
        .fetch_all(&self.pool)
        .await?;

        Ok((groups, total_items))
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<Group>, sqlx::Error> {
        let group = sqlx::query_as!(
            Group,
            r#"
            SELECT id, name, created_at, created_by, updated_at, updated_by
            FROM groups
            WHERE id = $1
            "#,
            id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(group)
    }

    async fn update(&self, id: Uuid, group: &UpdateGroup, user_id: Uuid) -> Result<Option<Group>, sqlx::Error> {
        let updated = sqlx::query_as!(
            Group,
            r#"
            UPDATE groups
            SET
                name = COALESCE($1, name),
                updated_by = $2,
                updated_at = CURRENT_TIMESTAMP
            WHERE id = $3
            RETURNING id, name, created_at, created_by, updated_at, updated_by
            "#,
            group.name,
            user_id,
            id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(updated)
    }

    async fn delete(&self, id: Uuid) -> Result<bool, sqlx::Error> {
        let result = sqlx::query!(
            r#"
            DELETE FROM groups
            WHERE id = $1
            "#,
            id
        )
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }
}
