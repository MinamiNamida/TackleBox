use crate::repo::{
    agents::{AgentRepo, GetAgentDTO, NewAgentDTO, UpdateAgentDTO},
    error::RepoError,
};
use std::sync::Arc;
use tackle_box::contracts::payloads_v1::{GetAgentResponse, NewAgentPayload, UpdateAgentPayload};
use uuid::Uuid;

pub struct AgentService {
    pub repo: Arc<AgentRepo>,
}

impl AgentService {
    pub async fn new_agent(
        &self,
        user_id: Uuid,
        agent: NewAgentPayload,
    ) -> Result<Uuid, RepoError> {
        let NewAgentPayload {
            name,
            game_type,
            version,
            description,
        } = agent;
        let agent = NewAgentDTO {
            owner_id: user_id,
            name,
            game_type: game_type,
            version: Some(version),
            description: description,
        };
        self.repo.new_agent(agent).await
    }
    pub async fn update_agent(
        &self,
        user_id: Uuid,
        agent: UpdateAgentPayload,
    ) -> Result<(), RepoError> {
        let UpdateAgentPayload {
            name,
            game_type,
            version,
            description,
        } = agent;
        let id = self.get_agent_id_by_name(user_id, &name).await?;
        let agent = UpdateAgentDTO {
            id,
            name,
            game_type,
            version,
            description,
        };
        self.repo.update_agent(agent).await
    }
    pub async fn delete_agent(&self, user_id: Uuid, agent_name: String) -> Result<(), RepoError> {
        let id = self.get_agent_id_by_name(user_id, &agent_name).await?;

        self.repo.delete_agent(id, user_id).await
    }
    pub async fn get_agent(
        &self,
        user_id: Uuid,
        agent_name: String,
    ) -> Result<GetAgentResponse, RepoError> {
        // let agent_id = self.get_agent_id_by_name(user_id, &agent_name).await?;
        let agent = self.repo.get_agent(agent_name).await?;
        let GetAgentDTO {
            agent_id,
            readable_agent_name,
            owner_id,
            owner_username,
            agent_name_base,
            game_type,
            version,
            description,
            played_games,
            won_games,
        } = agent;

        Ok(GetAgentResponse {
            id: agent_id,
            name: agent_name_base,
            version,
            game_type,
            description,
            // created_at,s
            played_games,
            won_games,
        })
    }
    pub async fn get_agent_id_by_name(
        &self,
        user_id: Uuid,
        name: &String,
    ) -> Result<Uuid, RepoError> {
        self.repo.get_agent_id_by_name(user_id, name).await
    }
    pub async fn get_agents_by_owner_id(
        &self,
        user_id: Uuid,
    ) -> Result<Vec<GetAgentResponse>, RepoError> {
        let agents = self.repo.get_agents_by_owner_id(user_id).await?;
        Ok(agents
            .into_iter()
            .map(|agent| {
                let GetAgentDTO {
                    agent_id,
                    readable_agent_name,
                    owner_id,
                    owner_username,
                    agent_name_base,
                    game_type,
                    version,
                    description,
                    played_games,
                    won_games,
                } = agent;

                GetAgentResponse {
                    id: agent_id,
                    name: agent_name_base,
                    version,
                    game_type,
                    description,
                    // created_at,s
                    played_games,
                    won_games,
                }
            })
            .collect())
    }
}
