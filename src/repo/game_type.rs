use sqlx::{query_as, PgPool};
use std::sync::Arc;
use tackle_box::contracts::payloads::GetGameTypeResponse;
use uuid::Uuid;

use crate::repo::error::RepoError;

// #[derive(Deserialize, Serialize, FromRow)]
// pub struct GetGameType {
//     pub name: String,
//     pub description: Option<String>,
// }

// #[derive(Deserialize, Serialize)]
// pub struct NewGameType {
//     pub name: String,
//     pub description: Option<String>,
// }

pub struct GameTypeRepo {
    pub pool: Arc<PgPool>,
}

impl GameTypeRepo {
    pub async fn get_game_types(&self) -> Result<Vec<GetGameTypeResponse>, RepoError> {
        let mut conn = self.pool.acquire().await?;
        let game_types = query_as!(
            GetGameTypeResponse,
            r#"
            SELECT
            game_type_id,
            name,
            sponsor,
            description,
            min_slots,
            max_slots
            FROM gametypes
            "#
        )
        .fetch_all(&mut *conn)
        .await?;
        Ok(game_types)
    }

    pub async fn get_game_type(
        &self,
        game_type_id: Uuid,
    ) -> Result<GetGameTypeResponse, RepoError> {
        let mut conn = self.pool.acquire().await?;
        let game_type = query_as!(
            GetGameTypeResponse,
            r#"
            SELECT
            game_type_id,
            name,
            sponsor,
            description,
            min_slots,
            max_slots
            FROM gametypes
            WHERE game_type_id = $1
            "#,
            game_type_id
        )
        .fetch_one(&mut *conn)
        .await?;
        Ok(game_type)
    }

    // pub async fn new_game_type(&self, game_type: NewGameType) -> Result<(), RepoError> {
    //     let mut conn = self.pool.acquire().await?;
    //     let _ = query!(
    //         "insert into gametypes (name, description) values ($1, $2)",
    //         game_type.name,
    //         game_type.description,
    //     )
    //     .execute(&mut *conn)
    //     .await?;
    //     Ok(())
    // }
}
