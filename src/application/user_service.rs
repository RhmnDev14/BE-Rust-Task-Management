use crate::domain::user::{CreateUser, LoginUser, UserError, UserRepository, UserResponse};
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use chrono::{Duration, Utc};
use jsonwebtoken::{encode, EncodingKey, Header};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String,
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

        Ok(UserResponse {
            id: user.id,
            username: user.username,
            email: user.email,
            created_at: user.created_at,
            updated_at: user.updated_at,
        })
    }

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
            sub: user.id.to_string(),
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
}
