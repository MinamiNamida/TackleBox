use std::sync::Arc;

use serde::{Deserialize, Serialize};
use sqlx::{prelude::FromRow, query, query_as, PgPool};

use crate::repo::error::RepoError;

#[derive(Deserialize, Serialize, FromRow)]
pub struct GetGameType {
    pub name: String,
    pub description: Option<String>,
}

#[derive(Deserialize, Serialize)]
pub struct NewGameType {
    pub name: String,
    pub description: Option<String>,
}

pub struct GameTypeRepo {
    pub pool: Arc<PgPool>,
}

impl GameTypeRepo {
    pub async fn get_geme_types(&self) -> Result<Vec<GetGameType>, RepoError> {
        let mut conn = self.pool.acquire().await?;
        let game_types = query_as!(GetGameType, "select name, description from gametypes")
            .fetch_all(&mut *conn)
            .await?;
        Ok(game_types)
    }

    pub async fn new_game_type(&self, game_type: NewGameType) -> Result<(), RepoError> {
        let mut conn = self.pool.acquire().await?;
        let _ = query!(
            "insert into gametypes (name, description) values ($1, $2)",
            game_type.name,
            game_type.description,
        )
        .execute(&mut *conn)
        .await?;
        Ok(())
    }
}
