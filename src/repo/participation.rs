use std::sync::Arc;

use sqlx::{query, query_as, query_scalar, PgPool};
use uuid::Uuid;

use crate::repo::error::RepoError;

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
            insert into participations (match_id, agent_id) values ($1, $2)
            "#,
            match_id,
            agent_id,
        )
        .execute(&mut *conn)
        .await?;
        Ok(())
    }

    pub async fn get_participants(&self, match_id: Uuid) -> Result<Vec<Uuid>, RepoError> {
        let mut conn = self.pool.acquire().await?;
        let participants = query_scalar!(
            r"
            select agent_id from participations where match_id = $1
            ",
            match_id
        )
        .fetch_all(&mut *conn)
        .await?;
        Ok(participants)
    }

    pub async fn get_participants_by_match_name(
        &self,
        match_name: &String,
    ) -> Result<Vec<String>, RepoError> {
        let mut conn = self.pool.acquire().await?;
        let participants = query_scalar!(
            r#"
            select readable_agent_name as "readable_agent_name!" 
            from v_readable_participations
            where readable_match_name = $1
            "#,
            match_name,
        )
        .fetch_all(&mut *conn)
        .await?;
        Ok(participants)
    }

    pub async fn count_participants(&self, match_id: Uuid) -> Result<i32, RepoError> {
        let mut conn = self.pool.acquire().await?;
        let num = query_scalar!(
            r#"
            select count(*) from participations where match_id = $1
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
            "delete from participations where match_id = $1 and agent_id = $2",
            match_id,
            agent_id,
        )
        .execute(&mut *conn)
        .await?;
        Ok(())
    }
}
