use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::{query, query_as, PgPool, Postgres, Transaction};
use std::sync::Arc;
use tackle_box::contracts::payloads::TurnLogResponse;
use uuid::Uuid;

use crate::repo::error::RepoError;

pub struct NewTurnDTO {
    pub match_id: Uuid,
    pub i_turn: i32,         // 回合数/局数
    pub score_deltas: Value, // JSONB 格式的得分变化
    pub log: Value,          // JSONB 格式的详细日志
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
}

#[derive(Serialize, Deserialize)]
pub struct GetSettleTurnDTO {
    pub match_id: Uuid,
    pub i_turn: i32,         // 回合数/局数
    pub score_deltas: Value, // JSONB 格式的得分变化
}

// #[derive(Serialize, Deserialize, FromRow)]
// pub struct GetTurnDTO {
//     pub turn_id: Uuid,
//     pub match_id: Uuid,
//     pub i_turn: i32,
//     pub score_deltas: Value,
//     pub log: Value,
// }

// #[derive(Serialize, Deserialize, FromRow)]
// pub struct GetReadableTurnDTO {
//     pub match_name: String,
//     pub i_turn: i32,
//     pub score_deltas: Value,
//     pub log: Value,
// }

pub struct TurnRepo {
    pub pool: Arc<PgPool>,
}

impl TurnRepo {
    pub async fn insert_turn(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        turn: NewTurnDTO,
    ) -> Result<(), RepoError> {
        // let mut conn = self.pool.acquire().await?;
        let _ = query!(
            r#"
            insert into turns (match_id, i_turn, score_deltas, log, start_time, end_time)
            values ($1, $2, $3, $4, $5, $6)
            "#,
            turn.match_id,
            turn.i_turn,
            turn.score_deltas,
            turn.log,
            turn.start_time,
            turn.end_time,
        )
        .execute(tx.as_mut())
        .await?;
        Ok(())
    }

    pub async fn get_all_turns(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        match_id: Uuid,
    ) -> Result<Vec<GetSettleTurnDTO>, RepoError> {
        let turns = query_as!(
            GetSettleTurnDTO,
            r#"
            select match_id, i_turn, score_deltas from turns
            where match_id = $1
            "#,
            match_id
        )
        .fetch_all(tx.as_mut())
        .await?;
        Ok(turns)
    }

    pub async fn get_i_turn(
        &self,
        match_id: Uuid,
        i_turn: i32,
    ) -> Result<TurnLogResponse, RepoError> {
        let mut conn = self.pool.acquire().await?;
        let turn = query_as!(
            TurnLogResponse,
            r#"
            SELECT 
                turn_id,
                match_id,
                i_turn,
                score_deltas,
                log
            FROM turns
            WHERE match_id = $1 AND i_turn = $2
            "#,
            match_id,
            i_turn,
        )
        .fetch_one(&mut *conn)
        .await?;
        Ok(turn)
    }

    pub async fn get_turns(&self, match_id: Uuid) -> Result<Vec<TurnLogResponse>, RepoError> {
        let mut conn = self.pool.acquire().await?;
        let turn = query_as!(
            TurnLogResponse,
            r#"
            SELECT 
                turn_id,
                match_id,
                i_turn,
                score_deltas,
                log
            FROM turns
            WHERE match_id = $1
            "#,
            match_id,
        )
        .fetch_all(&mut *conn)
        .await?;
        Ok(turn)
    }
}
