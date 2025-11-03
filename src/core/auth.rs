use crate::repo::{
    error::RepoError,
    users::{GetUserDTO, NewUserDTO, UserRepo},
};
use argon2::{
    password_hash::{rand_core::OsRng, SaltString},
    Argon2, PasswordHash, PasswordHasher, PasswordVerifier,
};
use std::sync::Arc;
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum AuthError {
    #[error("密码错误！")]
    WrongPassword,
    #[error("数据库错误")]
    RepoError(#[from] RepoError),
    #[error("生成密码哈希失败")]
    PasswordUnhashable,
    #[error("未找到该用户")]
    NotFoundUsername,
    #[error("该用户名已被注册")]
    AlreadyExists,
    #[error("jwt生成失败")]
    JWTError,
}

#[derive(Clone)]
pub struct AuthConfig {
    pub jwt_secret: String,
    pub expiration: usize, // sec
}

#[derive(Clone)]
pub struct AuthService {
    pub user_repo: Arc<UserRepo>,
    pub config: AuthConfig,
}

impl AuthService {
    pub async fn new(user_repo: Arc<UserRepo>, config: AuthConfig) -> Self {
        Self { user_repo, config }
    }

    pub async fn login(&self, username: &String, pwd: &String) -> Result<Uuid, AuthError> {
        let id = self.user_repo.get_id_by_name(username).await?;
        let password_hash = self.user_repo.get_pwd_hash(id).await?;
        let parsed_hash = PasswordHash::new(&password_hash).unwrap();
        let user_id = match Argon2::default().verify_password(pwd.as_bytes(), &parsed_hash) {
            Ok(_) => Ok(id),
            Err(_) => Err(AuthError::WrongPassword),
        }?;
        Ok(user_id)
    }

    pub async fn register(&self, username: &String, pwd: &String) -> Result<Uuid, AuthError> {
        if self.user_repo.get_id_by_name(username).await.is_ok() {
            return Err(AuthError::AlreadyExists);
        }

        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();

        let password_hash = argon2
            .hash_password(pwd.as_bytes(), &salt)
            .map_err(|_| AuthError::PasswordUnhashable)?
            .to_string();
        let user_id = self
            .user_repo
            .new_user(&NewUserDTO {
                username: username.clone(),
                password_hash,
            })
            .await?;
        Ok(user_id)
    }

    pub async fn me(&self, id: Uuid) -> Result<GetUserDTO, AuthError> {
        let userinfo = self.user_repo.get_user(id).await?;
        Ok(userinfo)
    }
}
