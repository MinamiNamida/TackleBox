use crate::repo::error::RepoError;
use sqlx::{query, query_as, PgPool, Postgres, Transaction};
use std::sync::Arc;
use tackle_box::contracts::payloads::{AgentPolicy, AgentStatus, GetAgentResponse};
use uuid::Uuid;

// #[derive(Serialize, Deserialize)]
// pub struct GetAgentDTO {
//     pub agent_id: Uuid,
//     pub name: String,
//     pub game_type_id: Uuid,
//     pub game_type_name: String,
//     pub owner_id: Uuid,
//     pub owner_name: String,
//     pub version: String,
//     pub description: Option<String>,
//     pub played_games: i32,
//     pub won_games: i32,
//     pub status: AgentStatus,
//     pub policy: AgentPolicy,
//     pub created_at: DateTime<Utc>,
//     pub updated_at: DateTime<Utc>,
// }

pub struct NewAgentDTO {
    pub user_id: Uuid,
    pub name: String,
    pub game_type_id: Uuid,
    pub version: String,
    pub description: Option<String>,
    pub policy: AgentPolicy,
}

pub struct UpdateAgentDTO {
    pub user_id: Uuid,
    pub agent_id: Uuid,
    pub name: String,
    pub game_type_id: Uuid,
    pub version: String,
    pub description: Option<String>,
    pub policy: AgentPolicy,
}

pub struct GetRankableAgentDTO {
    pub owner_id: Uuid,
    pub agent_id: Uuid,
    pub game_type_id: Uuid,
    pub won_games: i32,
    pub played_games: i32,
}

pub struct AgentRepo {
    pub pool: Arc<PgPool>,
}

impl AgentRepo {
    // pub async fn get_agent_id_by_name(
    //     &self,
    //     owner_id: Uuid,
    //     name: &String,
    // ) -> Result<Uuid, RepoError> {
    //     let mut conn = self.pool.acquire().await?;
    //     let agent_id = query_scalar!(
    //         "select agent_id from agents where owner_id = $1 and name = $2",
    //         owner_id,
    //         name
    //     )
    //     .fetch_one(&mut *conn)
    //     .await?;
    //     Ok(agent_id)
    // }

    pub async fn get_my_agents(&self, user_id: Uuid) -> Result<Vec<GetAgentResponse>, RepoError> {
        let mut conn = self.pool.acquire().await?;
        let agents = query_as!(
            GetAgentResponse,
            r#"
            SELECT
                A.agent_id,
                A.name,
                A.game_type_id,
                U.user_id AS owner_id,
                G.name AS game_type_name,
                U.username AS owner_name,
                A.version,
                A.description,
                A.created_at,            
                A.played_games,
                A.won_games,
                A.updated_at,
                A.status AS "status!:AgentStatus",
                A.policy AS "policy!:AgentPolicy"
            FROM
                AGENTS AS A
            INNER JOIN
                GAMETYPES AS G ON A.game_type_id = G.game_type_id
            INNER JOIN
                USERS AS U ON A.owner_id = U.user_id
            WHERE A.owner_id = $1 AND A.status != 'Decommissioned'
            "#,
            user_id,
        )
        .fetch_all(&mut *conn)
        .await?;
        Ok(agents)
    }

    pub async fn get_agent(&self, agent_id: Uuid) -> Result<GetAgentResponse, RepoError> {
        let mut conn = self.pool.acquire().await?;
        let agent = query_as!(
            GetAgentResponse,
            r#"
            SELECT
                A.agent_id,
                A.name,
                A.game_type_id,
                U.user_id AS owner_id,
                G.name AS game_type_name,
                U.username AS owner_name,
                A.version,
                A.description,
                A.created_at,            
                A.played_games,
                A.won_games,
                A.updated_at,
                A.policy AS "policy!:AgentPolicy",
                A.status AS "status!:AgentStatus"
            FROM
                AGENTS AS A
            INNER JOIN
                GAMETYPES AS G ON A.game_type_id = G.game_type_id
            INNER JOIN
                USERS AS U ON A.owner_id = U.user_id
            WHERE A.agent_id = $1 AND A.status != 'Decommissioned'
            "#,
            agent_id,
        )
        .fetch_one(&mut *conn)
        .await?;
        Ok(agent)
    }

    pub async fn new_agent(&self, agent: NewAgentDTO) -> Result<(), RepoError> {
        let mut conn = self.pool.acquire().await?;
        let _ = query!(
            r#"
            insert into 
            agents (owner_id, name, game_type_id, version, description, policy) 
            values ($1, $2, $3, $4, $5, $6)
            "#,
            agent.user_id,
            agent.name,
            agent.game_type_id,
            agent.version,
            agent.description,
            agent.policy as AgentPolicy,
        )
        .execute(&mut *conn)
        .await?;
        Ok(())
    }

    pub async fn update_agent(&self, agent: UpdateAgentDTO) -> Result<(), RepoError> {
        let mut conn = self.pool.acquire().await?;
        let _ = query!(
            r#"
            update agents 
            set (name, game_type_id, version, description, policy) = ($1, $2, $3, $4, $5) 
            where agent_id = $6 and owner_id = $7
            "#,
            agent.name,
            agent.game_type_id,
            agent.version,
            agent.description,
            agent.policy as AgentPolicy,
            agent.agent_id,
            agent.user_id
        )
        .execute(&mut *conn)
        .await?;
        Ok(())
    }

    pub async fn update_agent_status(
        &self,
        agent_id: Uuid,
        status: AgentStatus,
    ) -> Result<(), RepoError> {
        let mut conn = self.pool.acquire().await?;
        let _ = query!(
            r#"
            update agents 
            set status = $1 
            where agent_id = $2
            "#,
            status as AgentStatus,
            agent_id
        )
        .execute(&mut *conn)
        .await?;
        Ok(())
    }

    pub async fn delete_agent(&self, agent_id: Uuid, owner_id: Uuid) -> Result<(), RepoError> {
        self.update_agent_status(agent_id, AgentStatus::Decommissioned)
            .await?;
        Ok(())
    }

    // pub async fn get_agents_by_username(
    //     &self,
    //     owner_id: Uuid,
    // ) -> Result<Vec<GetAgentDTO>, RepoError> {
    //     let mut conn = self.pool.acquire().await?;
    //     let agents = query_as!(
    //         GetAgentDTO,
    //         r#"
    //         select
    //             agent_id as "agent_id!",
    //             owner_id as "owner_id!",
    //             agent_name_base as "agent_name_base!",
    //             game_type as "game_type!",
    //             version as "version!",
    //             description,
    //             played_games as "played_games!",
    //             won_games as "won_games!",
    //             owner_username as "owner_username!",
    //             readable_agent_name as "readable_agent_name!"
    //         from v_readable_agents
    //         where owner_id = $1
    //         "#,
    //         owner_id,
    //     )
    //     .fetch_all(&mut *conn)
    //     .await?;
    //     Ok(agents)
    // }

    pub async fn agent_won(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        agent_id: Uuid,
    ) -> Result<(), RepoError> {
        let _ = query!(
            r#"
            update agents set (won_games, played_games) = 
            (won_games+1, played_games+1) where agent_id = $1
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
            (won_games, played_games+1) where agent_id = $1
            "#,
            agent_id
        )
        .execute(tx.as_mut())
        .await?;
        Ok(())
    }

    pub async fn rankable_agents(&self) -> Result<Vec<GetRankableAgentDTO>, RepoError> {
        let mut conn = self.pool.acquire().await?;
        let agents = query_as!(
            GetRankableAgentDTO,
            r#"
            SELECT 
                A.agent_id,
                A.game_type_id,
                A.owner_id,
                A.won_games,
                A.played_games
            FROM AGENTS AS A
            WHERE A.status != 'Decommissioned'
            "#
        )
        .fetch_all(&mut *conn)
        .await?;
        Ok(agents)
    }
}
