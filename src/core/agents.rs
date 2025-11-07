use crate::repo::{
    agents::{AgentRepo, NewAgentDTO, UpdateAgentDTO},
    error::RepoError,
};
use std::sync::Arc;
use tackle_box::contracts::payloads::{GetAgentResponse, NewAgentPayload, UpdateAgentPayload};
use uuid::Uuid;

pub struct AgentService {
    pub repo: Arc<AgentRepo>,
}

impl AgentService {
    pub async fn new_agent(&self, user_id: Uuid, agent: NewAgentPayload) -> Result<(), RepoError> {
        let NewAgentPayload {
            name,
            game_type_id,
            version,
            description,
            policy,
        } = agent;

        let agent = NewAgentDTO {
            user_id,
            name,
            game_type_id,
            version,
            description,
            policy,
        };
        self.repo.new_agent(agent).await?;
        Ok(())
    }
    pub async fn update_agent(
        &self,
        user_id: Uuid,
        agent: UpdateAgentPayload,
    ) -> Result<(), RepoError> {
        let UpdateAgentPayload {
            agent_id,
            name,
            game_type_id,
            version,
            description,
            policy,
        } = agent;
        let agent = UpdateAgentDTO {
            user_id,
            agent_id,
            name,
            game_type_id,
            version,
            description,
            policy,
        };
        self.repo.update_agent(agent).await?;
        Ok(())
    }
    pub async fn delete_agent(&self, user_id: Uuid, agent_id: Uuid) -> Result<(), RepoError> {
        self.repo.delete_agent(agent_id, user_id).await?;
        Ok(())
    }
    pub async fn get_agent(
        &self,
        _user_id: Uuid,
        agent_id: Uuid,
    ) -> Result<GetAgentResponse, RepoError> {
        let agent = self.repo.get_agent(agent_id).await?;
        Ok(agent)
    }

    pub async fn get_my_agents(&self, user_id: Uuid) -> Result<Vec<GetAgentResponse>, RepoError> {
        let agents = self.repo.get_my_agents(user_id).await?;
        Ok(agents)
    }
}
