use serde::Serialize;
use sqlx::{prelude::FromRow, query, query_as, query_scalar, PgPool, Postgres, Transaction};
use std::sync::Arc;
use uuid::Uuid;

use crate::{api::error::AppError, repo::error::RepoError};

#[derive(Serialize, FromRow)]
pub struct GetAgentDTO {
    pub agent_id: Uuid,
    pub readable_agent_name: String,
    pub owner_id: Uuid,
    pub owner_username: String,
    pub agent_name_base: String,
    pub game_type: String,
    pub version: String,
    pub description: Option<String>,
    pub played_games: i32,
    pub won_games: i32,
}

pub struct NewAgentDTO {
    pub owner_id: Uuid,
    pub name: String,
    pub game_type: String,
    pub version: Option<String>,
    pub description: Option<String>,
}

pub struct UpdateAgentDTO {
    pub id: Uuid,
    pub name: String,
    pub game_type: String,
    pub version: String,
    pub description: Option<String>,
}

pub struct AgentRepo {
    pub pool: Arc<PgPool>,
}

impl AgentRepo {
    pub async fn get_agent_id_by_name(
        &self,
        owner_id: Uuid,
        name: &String,
    ) -> Result<Uuid, RepoError> {
        let mut conn = self.pool.acquire().await?;
        let agent_id = query_scalar!(
            "select id from agents where owner_id = $1 and name = $2",
            owner_id,
            name
        )
        .fetch_one(&mut *conn)
        .await?;
        Ok(agent_id)
    }

    pub async fn get_agents_by_owner_id(
        &self,
        user_id: Uuid,
    ) -> Result<Vec<GetAgentDTO>, RepoError> {
        let mut conn = self.pool.acquire().await?;
        let agents = query_as!(
            GetAgentDTO,
            r#"
            select 
                agent_id as "agent_id!", 
                owner_id as "owner_id!", 
                agent_name_base as "agent_name_base!", 
                game_type as "game_type!", 
                version as "version!", 
                description, 
                played_games as "played_games!", 
                won_games as "won_games!", 
                owner_username as "owner_username!", 
                readable_agent_name as "readable_agent_name!"
            from v_readable_agents 
            where owner_id = $1
            "#,
            user_id,
        )
        .fetch_all(&mut *conn)
        .await?;
        Ok(agents)
    }

    pub async fn get_agent(&self, agent_name: String) -> Result<GetAgentDTO, RepoError> {
        let mut conn = self.pool.acquire().await?;
        let agent = query_as!(
            GetAgentDTO,
            r#"
            select 
                agent_id as "agent_id!", 
                owner_id as "owner_id!", 
                agent_name_base as "agent_name_base!", 
                game_type as "game_type!", 
                version as "version!", 
                description, 
                played_games as "played_games!", 
                won_games as "won_games!", 
                owner_username as "owner_username!", 
                readable_agent_name as "readable_agent_name!"
            from v_readable_agents 
            where readable_agent_name = $1
            "#,
            agent_name,
        )
        .fetch_one(&mut *conn)
        .await?;
        Ok(agent)
    }

    pub async fn new_agent(&self, agent: NewAgentDTO) -> Result<Uuid, RepoError> {
        let mut conn = self.pool.acquire().await?;
        let result = query!(
            r#"
            insert into 
            agents (owner_id, name, game_type, version, description) 
            values ($1, $2, $3, $4, $5) returning id
            "#,
            agent.owner_id,
            agent.name,
            agent.game_type,
            agent.version,
            agent.description
        )
        .fetch_one(&mut *conn)
        .await?;
        Ok(result.id)
    }

    pub async fn update_agent(&self, agent: UpdateAgentDTO) -> Result<(), RepoError> {
        let mut conn = self.pool.acquire().await?;
        let _ = query!(
            r#"
            update agents 
            set (name, game_type, version, description) = ($1, $2, $3, $4) 
            where id = $5
            "#,
            agent.name,
            agent.game_type,
            agent.version,
            agent.description,
            agent.id
        )
        .execute(&mut *conn)
        .await?;
        Ok(())
    }

    pub async fn delete_agent(&self, agent_id: Uuid, owner_id: Uuid) -> Result<(), RepoError> {
        let mut conn = self.pool.acquire().await?;
        let _ = query!(
            "delete from agents where id = $1 and owner_id = $2",
            agent_id,
            owner_id
        )
        .execute(&mut *conn)
        .await?;
        Ok(())
    }

    pub async fn get_agents_by_username(
        &self,
        owner_id: Uuid,
    ) -> Result<Vec<GetAgentDTO>, RepoError> {
        let mut conn = self.pool.acquire().await?;
        let agents = query_as!(
            GetAgentDTO,
            r#"
            select 
                agent_id as "agent_id!", 
                owner_id as "owner_id!", 
                agent_name_base as "agent_name_base!", 
                game_type as "game_type!", 
                version as "version!", 
                description, 
                played_games as "played_games!", 
                won_games as "won_games!", 
                owner_username as "owner_username!", 
                readable_agent_name as "readable_agent_name!"
            from v_readable_agents 
            where owner_id = $1
            "#,
            owner_id,
        )
        .fetch_all(&mut *conn)
        .await?;
        Ok(agents)
    }

    pub async fn agent_won(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        agent_id: Uuid,
    ) -> Result<(), RepoError> {
        let _ = query!(
            r#"
            update agents set (won_games, played_games) = 
            (won_games+1, played_games+1) where id = $1
            "#,
            agent_id
        )
        .execute(tx.as_mut())
        .await?;
        Ok(())
    }

    pub async fn agent_failed(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        agent_id: Uuid,
    ) -> Result<(), RepoError> {
        let _ = query!(
            r#"
            update agents set (won_games, played_games) = 
            (won_games, played_games+1) where id = $1
            "#,
            agent_id
        )
        .execute(tx.as_mut())
        .await?;
        Ok(())
    }
}
