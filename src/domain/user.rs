use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, FromRow, ToSchema)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    #[serde(skip)]
    pub password_hash: String,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CreateUser {
    pub username: String,
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct LoginUser {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct UserResponse {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[async_trait::async_trait]
pub trait UserRepository: Send + Sync {
    async fn create(&self, user: &CreateUser, password_hash: &str) -> Result<User, sqlx::Error>;
    async fn find_by_email(&self, email: &str) -> Result<Option<User>, sqlx::Error>;
    async fn find_by_username(&self, username: &str) -> Result<Option<User>, sqlx::Error>;
}

#[derive(Debug)]
pub enum UserError {
    DatabaseError(#[allow(dead_code)] sqlx::Error),
    UserAlreadyExists,
    UserNotFound,
    InvalidCredentials,
    TokenCreationError,
}

impl From<sqlx::Error> for UserError {
    fn from(err: sqlx::Error) -> Self {
        UserError::DatabaseError(err)
    }
}
