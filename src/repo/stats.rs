use std::sync::Arc;

use chrono::{DateTime, Utc};
use sqlx::{query, query_as, PgPool};
use tackle_box::contracts::payloads::GetStatsResponse;
use uuid::Uuid;

use crate::repo::error::RepoError;

pub struct UpdateStatsDTO {
    pub agent_ids: Vec<Uuid>,
    pub game_type_ids: Vec<Uuid>,
    pub new_ranks: Vec<i32>,
}

pub struct StatsRepo {
    pub pool: Arc<PgPool>,
}

impl StatsRepo {
    pub async fn get_stats(&self) -> Result<Vec<GetStatsResponse>, RepoError> {
        let mut conn = self.pool.acquire().await?;
        let stats = query_as!(
            GetStatsResponse,
            r#"
            SELECT 
            S.game_type_id,
            G.name AS game_type_name,
            S.agent_id,
            A.name AS agent_name,
            S.rank,
            S.updated_time
            FROM 
                STATS AS S
            INNER JOIN
                GAMETYPES AS G ON G.game_type_id = S.game_type_id
            INNER JOIN 
                AGENTS AS A ON A.game_type_id = S.game_type_id
            WHERE
                A.status != 'Decommissioned'
            ORDER BY
                S.rank ASC,
                S.updated_time DESC
            "#
        )
        .fetch_all(&mut *conn)
        .await?;
        Ok(stats)
    }

    pub async fn update_stats(&self, data: UpdateStatsDTO) -> Result<(), RepoError> {
        let mut conn = self.pool.acquire().await?;
        let UpdateStatsDTO {
            agent_ids,
            game_type_ids,
            new_ranks,
        } = data;
        let now = Utc::now();
        let updated_times: Vec<DateTime<Utc>> =
            std::iter::repeat(now).take(agent_ids.len()).collect();
        query!(
            r#"
            WITH CalculatedRanks AS (
                SELECT
                    unnest($1::uuid[]) AS agent_id,
                    unnest($2::uuid[]) AS game_type_id,
                    unnest($3::int[]) AS new_rank,     -- Glicko Rating 排名或 Win Rate 排名
                    unnest($4::timestamp with time zone[]) AS update_time
            ),
            UpdateExisting AS (
                UPDATE STATS AS S
                SET 
                    rank = CR.new_rank,
                    updated_time = CR.update_time
                FROM 
                    CalculatedRanks AS CR
                WHERE 
                    S.agent_id = CR.agent_id AND S.game_type_id = CR.game_type_id
                RETURNING S.agent_id
            )
            INSERT INTO STATS (game_type_id, agent_id, rank, updated_time)
            SELECT 
                CR.game_type_id,
                CR.agent_id,
                CR.new_rank,
                CR.update_time
            FROM 
                CalculatedRanks AS CR
            WHERE 
                CR.agent_id NOT IN (SELECT agent_id FROM UpdateExisting);
            "#,
            &agent_ids,
            &game_type_ids,
            &new_ranks,
            &updated_times
        )
        .execute(&mut *conn)
        .await?;
        Ok(())
    }
}
