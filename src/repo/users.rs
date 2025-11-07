use std::sync::Arc;

use chrono::{DateTime, Utc};
use sqlx::{prelude::FromRow, query, query_as, query_scalar, PgPool};
use uuid::Uuid;

use crate::repo::error::RepoError;

#[derive(FromRow, Debug)]
pub struct GetUserDTO {
    pub user_id: Uuid,
    pub username: String,
    pub email: String,
    pub password_hash: String,
    pub created_at: DateTime<Utc>,
}

pub struct NewUserDTO {
    pub username: String,
    pub password_hash: String,
    pub email: String,
}

pub struct UserRepo {
    pub pool: Arc<PgPool>,
}

impl UserRepo {
    pub async fn get_id_by_name(&self, username: &String) -> Result<Uuid, RepoError> {
        let mut conn = self.pool.acquire().await?;
        let user_id = query_scalar!("SELECT user_id FROM users WHERE username = $1", username)
            .fetch_one(&mut *conn)
            .await?;
        Ok(user_id)
    }

    pub async fn get_user(&self, user_id: Uuid) -> Result<GetUserDTO, RepoError> {
        let mut conn = self.pool.acquire().await?;
        let user = query_as!(
            GetUserDTO,
            r#"
            SELECT 
                user_id,
                username,
                email,
                password_hash,
                created_at
            FROM users
            WHERE user_id = $1
            "#,
            user_id
        )
        .fetch_one(&mut *conn)
        .await?;
        Ok(user)
    }

    pub async fn new_user(&self, user: &NewUserDTO) -> Result<Uuid, RepoError> {
        let mut conn = self.pool.acquire().await?;
        let result = query!(
            r#"
            INSERT INTO users (username, password_hash, email)
            VALUES ($1, $2, $3)
            RETURNING user_id AS id
            "#,
            user.username,
            user.password_hash,
            user.email
        )
        .fetch_one(&mut *conn)
        .await?;
        Ok(result.id)
    }
}
