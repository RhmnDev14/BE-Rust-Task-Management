use crate::domain::task::{CreateTask, Task, TaskRepository, UpdateTask};
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
            INSERT INTO tasks (task_name, description, id_user)
            VALUES ($1, $2, $3)
            RETURNING id, task_name, description, id_user, created_at, updated_at
            "#,
            task.task_name,
            task.description,
            id_user
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(task)
    }

    async fn find_all(&self) -> Result<Vec<Task>, sqlx::Error> {
        let tasks = sqlx::query_as!(
            Task,
            r#"
            SELECT id, task_name, description, id_user, created_at, updated_at
            FROM tasks
            ORDER BY created_at DESC
            "#
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(tasks)
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<Task>, sqlx::Error> {
        let task = sqlx::query_as!(
            Task,
            r#"
            SELECT id, task_name, description, id_user, created_at, updated_at
            FROM tasks
            WHERE id = $1
            "#,
            id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(task)
    }

    async fn find_by_user_id(&self, id_user: Uuid) -> Result<Vec<Task>, sqlx::Error> {
        let tasks = sqlx::query_as!(
            Task,
            r#"
            SELECT id, task_name, description, id_user, created_at, updated_at
            FROM tasks
            WHERE id_user = $1
            ORDER BY created_at DESC
            "#,
            id_user
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(tasks)
    }

    async fn update(&self, id: Uuid, task: &UpdateTask) -> Result<Option<Task>, sqlx::Error> {
        let updated = sqlx::query_as!(
            Task,
            r#"
            UPDATE tasks
            SET
                task_name = COALESCE($1, task_name),
                description = COALESCE($2, description),
                updated_at = CURRENT_TIMESTAMP
            WHERE id = $3
            RETURNING id, task_name, description, id_user, created_at, updated_at
            "#,
            task.task_name,
            task.description,
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
