use crate::domain::task::{CreateTask, PaginationParams, Task, TaskRepository, UpdateTask};
use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

pub struct SqlxTaskRepository {
    pool: PgPool,
}

impl SqlxTaskRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl TaskRepository for SqlxTaskRepository {
    async fn create(&self, task: &CreateTask, id_user: Uuid) -> Result<Task, sqlx::Error> {
        let task = sqlx::query_as!(
            Task,
            r#"
            INSERT INTO tasks (task_name, description, story_point, id_user)
            VALUES ($1, $2, $3, $4)
            RETURNING id, task_name, description, story_point, id_user, created_at, updated_at
            "#,
            task.task_name,
            task.description,
            task.story_point,
            id_user
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(task)
    }

    async fn find_all(&self, pagination: &PaginationParams) -> Result<(Vec<Task>, i64), sqlx::Error> {
        let limit = pagination.limit.unwrap_or(10);
        let offset = (pagination.page.unwrap_or(1) - 1) * limit;

        let total_items = sqlx::query!(
            r#"SELECT COUNT(*) as count FROM tasks"#
        )
        .fetch_one(&self.pool)
        .await?
        .count
        .unwrap_or(0);

        let tasks = sqlx::query_as!(
            Task,
            r#"
            SELECT id, task_name, description, story_point, id_user, created_at, updated_at
            FROM tasks
            ORDER BY created_at DESC
            LIMIT $1 OFFSET $2
            "#,
            limit,
            offset
        )
        .fetch_all(&self.pool)
        .await?;

        Ok((tasks, total_items))
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<Task>, sqlx::Error> {
        let task = sqlx::query_as!(
            Task,
            r#"
            SELECT id, task_name, description, story_point, id_user, created_at, updated_at
            FROM tasks
            WHERE id = $1
            "#,
            id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(task)
    }

    async fn find_by_user_id(
        &self,
        id_user: Uuid,
        pagination: &PaginationParams,
    ) -> Result<(Vec<Task>, i64), sqlx::Error> {
        let limit = pagination.limit.unwrap_or(10);
        let offset = (pagination.page.unwrap_or(1) - 1) * limit;

        let total_items = sqlx::query!(
            r#"SELECT COUNT(*) as count FROM tasks WHERE id_user = $1"#,
            id_user
        )
        .fetch_one(&self.pool)
        .await?
        .count
        .unwrap_or(0);

        let tasks = sqlx::query_as!(
            Task,
            r#"
            SELECT id, task_name, description, story_point, id_user, created_at, updated_at
            FROM tasks
            WHERE id_user = $1
            ORDER BY created_at DESC
            LIMIT $2 OFFSET $3
            "#,
            id_user,
            limit,
            offset
        )
        .fetch_all(&self.pool)
        .await?;

        Ok((tasks, total_items))
    }

    async fn search(
        &self,
        id_user: Uuid,
        query: &str,
        pagination: &PaginationParams,
    ) -> Result<(Vec<Task>, i64), sqlx::Error> {
        let limit = pagination.limit.unwrap_or(10);
        let offset = (pagination.page.unwrap_or(1) - 1) * limit;
        let search_pattern = format!("%{}%", query);

        let total_items = sqlx::query!(
            r#"
            SELECT COUNT(*) as count FROM tasks
            WHERE id_user = $1 AND (task_name ILIKE $2 OR description ILIKE $2)
            "#,
            id_user,
            search_pattern
        )
        .fetch_one(&self.pool)
        .await?
        .count
        .unwrap_or(0);

        let tasks = sqlx::query_as!(
            Task,
            r#"
            SELECT id, task_name, description, story_point, id_user, created_at, updated_at
            FROM tasks
            WHERE id_user = $1 AND (task_name ILIKE $2 OR description ILIKE $2)
            ORDER BY created_at DESC
            LIMIT $3 OFFSET $4
            "#,
            id_user,
            search_pattern,
            limit,
            offset
        )
        .fetch_all(&self.pool)
        .await?;

        Ok((tasks, total_items))
    }

    async fn update(&self, id: Uuid, task: &UpdateTask) -> Result<Option<Task>, sqlx::Error> {
        let updated = sqlx::query_as!(
            Task,
            r#"
            UPDATE tasks
            SET
                task_name = COALESCE($1, task_name),
                description = COALESCE($2, description),
                story_point = COALESCE($3, story_point),
                updated_at = CURRENT_TIMESTAMP
            WHERE id = $4
            RETURNING id, task_name, description, story_point, id_user, created_at, updated_at
            "#,
            task.task_name,
            task.description,
            task.story_point,
            id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(updated)
    }

    async fn delete(&self, id: Uuid) -> Result<bool, sqlx::Error> {
        let result: sqlx::postgres::PgQueryResult = sqlx::query!(
            r#"
            DELETE FROM tasks
            WHERE id = $1
            "#,
            id
        )
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }
}
