use crate::domain::master::{MasterRepository, ProgressOption};
use async_trait::async_trait;
use sqlx::PgPool;

pub struct SqlxMasterRepository {
    pool: PgPool,
}

impl SqlxMasterRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl MasterRepository for SqlxMasterRepository {
    async fn find_all_progress_options(&self) -> Result<Vec<ProgressOption>, sqlx::Error> {
        let options = sqlx::query_as!(
            ProgressOption,
            r#"
            SELECT id, name
            FROM master_progress
            ORDER BY name ASC
            "#
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(options)
    }
}
