use std::sync::Arc;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{
    prelude::{FromRow, Type},
    query, query_as, query_scalar, PgPool, Postgres, Transaction,
};
use uuid::Uuid;

use crate::repo::error::RepoError;

#[derive(Debug, Type, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[sqlx(type_name = "match_status", rename_all = "PascalCase")]
pub enum MatchStatus {
    Pending,
    Running,
    Completed,
    Cancelled,
}

impl Into<String> for MatchStatus {
    fn into(self) -> String {
        match self {
            MatchStatus::Cancelled => "Cancelled".to_owned(),
            MatchStatus::Completed => "Completed".to_owned(),
            MatchStatus::Pending => "Pending".to_owned(),
            MatchStatus::Running => "Running".to_owned(),
        }
    }
}

pub struct NewMatchDTO {
    pub name: String,
    pub game_type: String,
    pub total_games: i32,
    pub creater_id: Uuid,
    pub password: Option<String>,
}

#[derive(FromRow, Serialize)]
pub struct GetMatchDTO {
    pub match_name_base: String,
    pub match_id: Uuid,
    pub creater_id: Uuid,
    pub creater_username: String,
    pub winner_id: Option<Uuid>,
    pub readable_match_name: String,
    pub winner_agent_readable_name: Option<String>,
    pub game_type: String,
    pub password: Option<String>,
    pub total_games: i32,
    pub status: MatchStatus,
    pub start_time: DateTime<Utc>,
    pub end_time: Option<DateTime<Utc>>,
}

#[derive(FromRow, Serialize)]
pub struct GetOnlineMatchDTO {
    pub readable_match_name: String,
    pub match_name_base: String,
    pub match_id: Uuid,
    pub creater_id: Uuid,
    pub creater_username: String,
    pub game_type: String,
    pub total_games: i32,
    pub status: MatchStatus,
    pub start_time: DateTime<Utc>,
    pub password: Option<String>,
}

pub struct MatchRepo {
    pub pool: Arc<PgPool>,
}

impl MatchRepo {
    pub async fn get_transaction(&self) -> Result<Transaction<'_, Postgres>, RepoError> {
        Ok(self.pool.begin().await?)
    }

    pub async fn new_match(&self, one_match: NewMatchDTO) -> Result<Uuid, RepoError> {
        let mut conn = self.pool.acquire().await?;
        let result = query!(
            r#"
            insert into matches (name, game_type, total_games, creater_id, status, password)
            values ($1, $2, $3, $4, $5::match_status, $6) returning id;
            "#,
            one_match.name,
            one_match.game_type,
            one_match.total_games,
            one_match.creater_id,
            MatchStatus::Pending as MatchStatus,
            one_match.password,
        )
        .fetch_one(&mut *conn)
        .await?;
        Ok(result.id)
    }

    pub async fn get_match_id_by_name(
        &self,
        user_id: Uuid,
        match_name: &String,
    ) -> Result<Uuid, RepoError> {
        let mut conn = self.pool.acquire().await?;
        let match_id = query_scalar!(
            "select id from matches where creater_id = $1 and name = $2",
            user_id,
            match_name
        )
        .fetch_one(&mut *conn)
        .await?;
        Ok(match_id)
    }

    pub async fn get_match_by_id(&self, match_id: Uuid) -> Result<GetMatchDTO, RepoError> {
        let mut conn = self.pool.acquire().await?;
        let one_match = query_as!(
            GetMatchDTO,
            r#"
            select match_id as "match_id!", 
            readable_match_name as "readable_match_name!",
            match_name_base as "match_name_base!", 
            creater_username as "creater_username!",
            game_type as "game_type!", 
            total_games as "total_games!", 
            creater_id as "creater_id!", 
            winner_id,
            winner_agent_readable_name,
            status as "status!:MatchStatus", 
            start_time as "start_time!", 
            end_time,
            password
            from v_readable_matches
            where match_id = $1
            "#,
            match_id
        )
        .fetch_one(&mut *conn)
        .await?;
        Ok(one_match)
    }

    pub async fn get_match(&self, match_name: &String) -> Result<GetMatchDTO, RepoError> {
        let mut conn = self.pool.acquire().await?;
        let one_match = query_as!(
            GetMatchDTO,
            r#"
            select match_id as "match_id!", 
            readable_match_name as "readable_match_name!",
            match_name_base as "match_name_base!", 
            creater_username as "creater_username!",
            game_type as "game_type!", 
            total_games as "total_games!", 
            creater_id as "creater_id!", 
            winner_id,
            winner_agent_readable_name,
            status as "status!:MatchStatus", 
            start_time as "start_time!", 
            end_time,
            password
            from v_readable_matches
            where readable_match_name = $1
            "#,
            match_name
        )
        .fetch_one(&mut *conn)
        .await?;
        Ok(one_match)
    }

    pub async fn get_matches(&self, user_id: Uuid) -> Result<Vec<GetMatchDTO>, RepoError> {
        let mut conn = self.pool.acquire().await?;
        let one_match = query_as!(
            GetMatchDTO,
            r#"
            select match_id as "match_id!", 
            readable_match_name as "readable_match_name!",
            match_name_base as "match_name_base!", 
            creater_username as "creater_username!",
            game_type as "game_type!", 
            total_games as "total_games!", 
            creater_id as "creater_id!", 
            winner_id,
            winner_agent_readable_name,
            status as "status!:MatchStatus", 
            start_time as "start_time!", 
            end_time,
            password
            from v_readable_matches
            where creater_id = $1
            "#,
            user_id
        )
        .fetch_all(&mut *conn)
        .await?;
        Ok(one_match)
    }

    pub async fn get_online_matches(&self) -> Result<Vec<GetOnlineMatchDTO>, RepoError> {
        let mut conn = self.pool.acquire().await?;
        let matches = query_as!(
            GetOnlineMatchDTO,
            r#"
            select match_id as "match_id!", 
            readable_match_name as "readable_match_name!",
            match_name_base as "match_name_base!", 
            creater_username as "creater_username!",
            game_type as "game_type!", 
            total_games as "total_games!", 
            creater_id as "creater_id!", 
            status as "status!:MatchStatus", 
            password,
            start_time as "start_time!"
            from v_readable_matches
            where status = $1
            "#,
            MatchStatus::Pending as MatchStatus
        )
        .fetch_all(&mut *conn)
        .await?;
        Ok(matches)
    }

    pub async fn get_match_status(&self, match_id: Uuid) -> Result<MatchStatus, RepoError> {
        let mut conn = self.pool.acquire().await?;
        let status: MatchStatus = query_scalar!(
            r#"
            SELECT status as "status!: MatchStatus" FROM "matches" WHERE id = $1;
            "#,
            match_id
        )
        .fetch_one(&mut *conn)
        .await?;
        Ok(status)
    }

    pub async fn update_match_status(
        &self,
        match_id: Uuid,
        status: MatchStatus,
    ) -> Result<(), RepoError> {
        let mut conn = self.pool.acquire().await?;
        let _ = query!(
            r#"
            update matches 
            set status = $1
            where id = $2
            "#,
            status as MatchStatus,
            match_id,
        )
        .execute(&mut *conn)
        .await?;
        Ok(())
    }

    pub async fn delete_match(&self, match_id: Uuid, creater_id: Uuid) -> Result<(), RepoError> {
        let mut conn = self.pool.acquire().await?;
        let _ = query!(
            r#"
            delete from matches where id = $1 and creater_id = $2;
            "#,
            match_id,
            creater_id,
        )
        .execute(&mut *conn)
        .await?;
        Ok(())
    }

    pub async fn update_match_final_status(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        match_id: Uuid,
        winner_id: Option<Uuid>,
    ) -> Result<(), RepoError> {
        let _ = query!(
            r#"
            update matches set status = $1, winner_id = $2 where id = $3
            "#,
            MatchStatus::Completed as MatchStatus,
            winner_id,
            match_id
        )
        .execute(tx.as_mut())
        .await?;
        Ok(())
    }
}
