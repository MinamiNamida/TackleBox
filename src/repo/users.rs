use std::sync::Arc;

use chrono::{DateTime, Utc};
use sqlx::{prelude::FromRow, query, query_as, query_scalar, PgPool};
use uuid::Uuid;

use crate::repo::error::RepoError;

#[derive(FromRow)]
pub struct GetIdByNameDTO {
    pub id: Uuid,
    pub username: String,
}

#[derive(FromRow, Debug)]
pub struct GetPasswordDTO {
    pub id: Uuid,
    pub password_hash: String,
}

#[derive(FromRow, Debug)]
pub struct GetUserDTO {
    pub id: Uuid,
    pub username: String,
    pub created_at: DateTime<Utc>,
}

pub struct NewUserDTO {
    pub username: String,
    pub password_hash: String,
}

pub struct UserRepo {
    pub pool: Arc<PgPool>,
}

impl UserRepo {
    pub async fn get_username_by_id(&self, user_id: Uuid) -> Result<String, RepoError> {
        let mut conn = self.pool.acquire().await?;
        let user_id = query_scalar!("select username from users where id = $1", user_id)
            .fetch_one(&mut *conn)
            .await?;
        Ok(user_id)
    }

    pub async fn get_id_by_name(&self, username: &String) -> Result<Uuid, RepoError> {
        let mut conn = self.pool.acquire().await?;
        let user_id = query_scalar!("select id from users where username = $1", username)
            .fetch_one(&mut *conn)
            .await?;
        Ok(user_id)
    }

    pub async fn get_pwd_hash(&self, id: Uuid) -> Result<String, RepoError> {
        let mut conn = self.pool.acquire().await?;
        let pwd_hash = query_scalar!("select password_hash from users where id = $1", id)
            .fetch_one(&mut *conn)
            .await?;
        Ok(pwd_hash)
    }

    pub async fn get_user(&self, id: Uuid) -> Result<GetUserDTO, RepoError> {
        let mut conn = self.pool.acquire().await?;
        let user = query_as!(
            GetUserDTO,
            "select id, username, created_at from users where id = $1",
            id
        )
        .fetch_one(&mut *conn)
        .await?;
        Ok(user)
    }

    pub async fn new_user(&self, user: &NewUserDTO) -> Result<Uuid, RepoError> {
        let mut conn = self.pool.acquire().await?;
        let result = query!(
            "insert into users (username, password_hash) values ($1, $2) returning id",
            user.username,
            user.password_hash
        )
        .fetch_one(&mut *conn)
        .await?;
        Ok(result.id)
    }
}
