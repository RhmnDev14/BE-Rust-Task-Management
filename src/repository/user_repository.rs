use crate::domain::user::{CreateUser, UpdateUser, User, UserRepository};
use async_trait::async_trait;
use sqlx::PgPool;

pub struct SqlxUserRepository {
    pool: PgPool,
}

impl SqlxUserRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl UserRepository for SqlxUserRepository {
    async fn create(&self, user: &CreateUser, password_hash: &str) -> Result<User, sqlx::Error> {
        let user = sqlx::query_as!(
            User,
            r#"
            INSERT INTO users (username, email, password_hash)
            VALUES ($1, $2, $3)
            RETURNING id, username, email, password_hash, avatar_url, created_at, updated_at
            "#,
            user.username,
            user.email,
            password_hash
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(user)
    }

    async fn find_by_email(&self, email: &str) -> Result<Option<User>, sqlx::Error> {
        let user = sqlx::query_as!(
            User,
            r#"
            SELECT id, username, email, password_hash, avatar_url, created_at, updated_at
            FROM users
            WHERE email = $1
            "#,
            email
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(user)
    }

    async fn find_by_username(&self, username: &str) -> Result<Option<User>, sqlx::Error> {
        let user = sqlx::query_as!(
            User,
            r#"
            SELECT id, username, email, password_hash, avatar_url, created_at, updated_at
            FROM users
            WHERE username = $1
            "#,
            username
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(user)
    }

    async fn find_by_id(&self, id: &uuid::Uuid) -> Result<Option<User>, sqlx::Error> {
        let user = sqlx::query_as!(
            User,
            r#"
            SELECT id, username, email, password_hash, avatar_url, created_at, updated_at
            FROM users
            WHERE id = $1
            "#,
            id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(user)
    }

    async fn find_all_options(&self) -> Result<Vec<crate::domain::user::UserOption>, sqlx::Error> {
        use crate::domain::user::UserOption;
        let options = sqlx::query_as!(
            UserOption,
            r#"
            SELECT id, username FROM users
            "#
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(options)
    }

    async fn update(&self, id: &uuid::Uuid, user: &UpdateUser) -> Result<User, sqlx::Error> {
        let updated_user = sqlx::query_as!(
            User,
            r#"
            UPDATE users
            SET 
                username = COALESCE($1, username),
                avatar_url = COALESCE($2, avatar_url),
                updated_at = CURRENT_TIMESTAMP
            WHERE id = $3
            RETURNING id, username, email, password_hash, avatar_url, created_at, updated_at
            "#,
            user.username,
            user.avatar_url,
            id
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(updated_user)
    }

    async fn update_password(&self, id: &uuid::Uuid, password_hash: &str) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"
            UPDATE users
            SET
                password_hash = $1,
                updated_at = CURRENT_TIMESTAMP
            WHERE id = $2
            "#,
            password_hash,
            id
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}
