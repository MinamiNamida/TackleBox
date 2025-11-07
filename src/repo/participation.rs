use std::sync::Arc;

use sqlx::{query, query_as, query_scalar, PgPool};
use tackle_box::contracts::payloads::GetParticipantsResponse;
use uuid::Uuid;

use crate::repo::error::RepoError;

// pub struct GetParticipantDTO {
//     pub match_id: Uuid,
//     pub match_name: String,
//     pub agent_id: Uuid,
//     pub agent_name: String,
// }

pub struct ParticipationRepo {
    pub pool: Arc<PgPool>,
}

impl ParticipationRepo {
    pub async fn insert_participant(
        &self,
        match_id: Uuid,
        agent_id: Uuid,
    ) -> Result<(), RepoError> {
        let mut conn = self.pool.acquire().await?;
        let _ = query!(
            r#"
            insert into participants (match_id, agent_id) values ($1, $2)
            "#,
            match_id,
            agent_id,
        )
        .execute(&mut *conn)
        .await?;
        Ok(())
    }

    pub async fn get_participants(
        &self,
        match_id: Uuid,
    ) -> Result<Vec<GetParticipantsResponse>, RepoError> {
        let mut conn = self.pool.acquire().await?;
        let participants = query_as!(
            GetParticipantsResponse,
            r"
            SELECT 
                P.match_id,
                M.name as match_name,
                P.agent_id,
                A.name as agent_name
            FROM participants P
            JOIN matches M ON P.match_id = M.match_id
            JOIN agents A ON P.agent_id = A.agent_id
            WHERE P.match_id = $1
            ",
            match_id
        )
        .fetch_all(&mut *conn)
        .await?;
        Ok(participants)
    }

    pub async fn count_participants(&self, match_id: Uuid) -> Result<i32, RepoError> {
        let mut conn = self.pool.acquire().await?;
        let num = query_scalar!(
            r#"
            select count(*) from participants where match_id = $1
            "#,
            match_id
        )
        .fetch_one(&mut *conn)
        .await?;
        Ok(num.unwrap_or(0) as i32)
    }

    pub async fn remove_participants(
        &self,
        match_id: Uuid,
        agent_id: Uuid,
    ) -> Result<(), RepoError> {
        let mut conn = self.pool.acquire().await?;
        let _ = query!(
            "delete from participants where match_id = $1 and agent_id = $2",
            match_id,
            agent_id,
        )
        .execute(&mut *conn)
        .await?;
        Ok(())
    }
}
