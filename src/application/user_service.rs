use crate::domain::user::{
    CreateUser, LoginUser, UpdateUser, UserError, UserRepository, UserResponse,
};
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use chrono::{Duration, Utc};
use jsonwebtoken::{encode, EncodingKey, Header};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    id: String,
    exp: usize,
    iat: usize,
}

pub struct UserService {
    user_repository: Box<dyn UserRepository>,
    jwt_secret: String,
}

impl UserService {
    pub fn new(user_repository: Box<dyn UserRepository>, jwt_secret: String) -> Self {
        Self {
            user_repository,
            jwt_secret,
        }
    }

    #[tracing::instrument(skip(self, create_user))]
    pub async fn register(&self, create_user: CreateUser) -> Result<UserResponse, UserError> {
        if self
            .user_repository
            .find_by_email(&create_user.email)
            .await?
            .is_some()
        {
            return Err(UserError::UserAlreadyExists);
        }

        if self
            .user_repository
            .find_by_username(&create_user.username)
            .await?
            .is_some()
        {
            // Ideally distinctive, but for now reuse
            return Err(UserError::UserAlreadyExists);
        }

        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        let password_hash = argon2
            .hash_password(create_user.password.as_bytes(), &salt)
            .map_err(|_| UserError::InvalidCredentials)? // Mapping error simply
            .to_string();

        let user = self
            .user_repository
            .create(&create_user, &password_hash)
            .await?;

        // Berikan role 'User' secara default
        self.user_repository
            .assign_role(&user.id, "User")
            .await?;

        Ok(UserResponse {
            id: user.id,
            username: user.username,
            email: user.email,
            avatar_url: user.avatar_url,
            created_at: user.created_at,
            updated_at: user.updated_at,
        })
    }

    #[tracing::instrument(skip(self, login_user), fields(email = %login_user.email))]
    pub async fn login(&self, login_user: LoginUser) -> Result<String, UserError> {
        let user = self
            .user_repository
            .find_by_email(&login_user.email)
            .await?
            .ok_or(UserError::UserNotFound)?;

        let parsed_hash =
            PasswordHash::new(&user.password_hash).map_err(|_| UserError::InvalidCredentials)?;

        Argon2::default()
            .verify_password(login_user.password.as_bytes(), &parsed_hash)
            .map_err(|_| UserError::InvalidCredentials)?;

        let expiration = Utc::now()
            .checked_add_signed(Duration::hours(24))
            .expect("valid timestamp")
            .timestamp();

        let claims = Claims {
            id: user.id.to_string(),
            exp: expiration as usize,
            iat: Utc::now().timestamp() as usize,
        };

        let token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.jwt_secret.as_bytes()),
        )
        .map_err(|_| UserError::TokenCreationError)?;

        Ok(token)
    }

    #[tracing::instrument(skip(self))]
    pub async fn get_user_by_id(&self, id: uuid::Uuid) -> Result<UserResponse, UserError> {
        let user = self
            .user_repository
            .find_by_id(&id)
            .await?
            .ok_or(UserError::UserNotFound)?;

        Ok(UserResponse {
            id: user.id,
            username: user.username,
            email: user.email,
            avatar_url: user.avatar_url,
            created_at: user.created_at,
            updated_at: user.updated_at,
        })
    }

    #[tracing::instrument(skip(self, update_user))]
    pub async fn update_profile(
        &self,
        id: uuid::Uuid,
        update_user: UpdateUser,
    ) -> Result<UserResponse, UserError> {
        let user = self.user_repository.update(&id, &update_user).await?;

        Ok(UserResponse {
            id: user.id,
            username: user.username,
            email: user.email,
            avatar_url: user.avatar_url,
            created_at: user.created_at,
            updated_at: user.updated_at,
        })
    }

    #[tracing::instrument(skip(self, change_password))]
    pub async fn change_password(
        &self,
        id: uuid::Uuid,
        change_password: crate::domain::user::ChangePassword,
    ) -> Result<(), UserError> {
        let user = self
            .user_repository
            .find_by_id(&id)
            .await?
            .ok_or(UserError::UserNotFound)?;

        // Verify current password
        let parsed_hash =
            PasswordHash::new(&user.password_hash).map_err(|_| UserError::InvalidCredentials)?;

        Argon2::default()
            .verify_password(change_password.current_password.as_bytes(), &parsed_hash)
            .map_err(|_| UserError::InvalidCredentials)?;

        // Hash new password
        let salt = SaltString::generate(&mut OsRng);
        let new_password_hash = Argon2::default()
            .hash_password(change_password.new_password.as_bytes(), &salt)
            .map_err(|_| UserError::InvalidCredentials)?
            .to_string();

        self.user_repository
            .update_password(&id, &new_password_hash)
            .await?;

        Ok(())
    }

    #[tracing::instrument(skip(self))]
    pub async fn get_user_options(&self) -> Result<Vec<crate::domain::user::UserOption>, UserError> {
        let options = self.user_repository.find_all_options().await?;
        Ok(options)
    }
}
