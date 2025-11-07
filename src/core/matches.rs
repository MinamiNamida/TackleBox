use std::sync::Arc;
use tackle_box::contracts::payloads::{
    GetGameTypeResponse, GetMatchResponse, GetOnlineMatchResponse, GetParticipantsResponse,
    MatchStatus, NewMatchPayload, TurnLogResponse,
};
use tokio::sync::mpsc::Sender;
use uuid::Uuid;

use crate::{
    api::error::AppError,
    core::core::CoreMessage,
    repo::{
        agents::AgentRepo,
        game_type::GameTypeRepo,
        matches::{MatchRepo, NewMatchDTO},
        participation::ParticipationRepo,
        turns::TurnRepo,
        users::UserRepo,
    },
};

struct Repos {
    pub gametype_repo: Arc<GameTypeRepo>,
    pub user_repo: Arc<UserRepo>,
    pub agent_repo: Arc<AgentRepo>,
    pub match_repo: Arc<MatchRepo>,
    pub turn_repo: Arc<TurnRepo>,
    pub participation_repo: Arc<ParticipationRepo>,
}

struct Senders {
    pub core_tx: Sender<CoreMessage>,
}

pub struct MatchService {
    // pub orchestrator_service: Arc<OrchestratorService>,
    repos: Repos,
    senders: Senders,
}

impl MatchService {
    pub fn new(
        gametype_repo: Arc<GameTypeRepo>,
        user_repo: Arc<UserRepo>,
        agent_repo: Arc<AgentRepo>,
        match_repo: Arc<MatchRepo>,
        turn_repo: Arc<TurnRepo>,
        participation_repo: Arc<ParticipationRepo>,
        core_tx: Sender<CoreMessage>,
        // orchestrator_service: Arc<OrchestratorService>,
    ) -> Self {
        Self {
            repos: Repos {
                gametype_repo,
                user_repo,
                agent_repo,
                match_repo,
                turn_repo,
                participation_repo,
            }, // orchestrator_service,
            senders: Senders { core_tx },
        }
    }

    pub async fn get_match(&self, match_id: Uuid) -> Result<GetMatchResponse, AppError> {
        let one_match = self.repos.match_repo.get_match(match_id).await?;
        Ok(one_match)
    }

    pub async fn get_online_matches(
        &self,
        _user_id: Uuid,
    ) -> Result<Vec<GetOnlineMatchResponse>, AppError> {
        let matches = self.repos.match_repo.get_online_matches().await?;
        Ok(matches)
    }

    pub async fn get_my_joined_matches(
        &self,
        user_id: Uuid,
    ) -> Result<Vec<GetMatchResponse>, AppError> {
        let matches = self.repos.match_repo.get_my_joined_matches(user_id).await?;
        Ok(matches)
    }

    pub async fn new_match(
        &self,
        user_id: Uuid,
        one_match: NewMatchPayload,
    ) -> Result<(), AppError> {
        let NewMatchPayload {
            name,
            game_type_id,
            total_games,
            with_agent_ids,
            password,
        } = one_match;

        let one_match = NewMatchDTO {
            name: name.clone(),
            game_type_id,
            total_games,
            creater_id: user_id,
            password: password.clone(),
        };
        let match_id = self.repos.match_repo.new_match(one_match).await?;
        self.join_match(user_id, match_id, with_agent_ids, password)
            .await?;
        Ok(())
    }

    pub async fn join_match(
        &self,
        user_id: Uuid,
        match_id: Uuid,
        agent_ids: Vec<Uuid>,
        password: Option<String>,
    ) -> Result<(), AppError> {
        let one_match = self.repos.match_repo.get_match(match_id).await?;
        let match_id = one_match.match_id;
        let match_status: MatchStatus = self.repos.match_repo.get_match_status(match_id).await?;
        if match_status != MatchStatus::Pending {
            return Err(AppError::Internal(
                "match is not pending, cannot join".to_string(),
            ));
        }
        if password != one_match.password {
            return Err(AppError::Internal("error password".to_string()));
        }
        let join_agents_len = agent_ids.len() as i32;

        let counts = self
            .repos
            .participation_repo
            .count_participants(match_id)
            .await?;
        if counts + join_agents_len > one_match.max_slots {
            return Err(AppError::Internal(
                "exceeding max slots for this match".to_string(),
            ));
        } else {
            for agent_id in agent_ids {
                self.repos
                    .participation_repo
                    .insert_participant(match_id, agent_id)
                    .await?;
            }
        }

        if counts + join_agents_len >= one_match.min_slots {
            let GetMatchResponse {
                game_type_name,
                total_games,
                ..
            } = self.repos.match_repo.get_match(match_id).await?;
            let agent_ids = self
                .repos
                .participation_repo
                .get_participants(match_id)
                .await?
                .iter()
                .map(|p| p.agent_id)
                .collect();
            self.start_match(
                match_id,
                agent_ids,
                "rlcard".to_string(),
                game_type_name,
                total_games,
            )
            .await?;
        }

        Ok(())
    }

    pub async fn start_match(
        &self,
        match_id: Uuid,
        agent_ids: Vec<Uuid>,
        sponsor: String,
        game_type: String,
        total_games: i32,
    ) -> Result<(), AppError> {
        self.senders
            .core_tx
            .send(CoreMessage::MatchStart {
                match_id,
                agent_ids,
                sponsor,
                game_type,
                total_games,
            })
            .await?;
        Ok(())
    }

    // pub async fn leave_match(&self, user_id: Uuid, agent_name: &String) -> Result<(), AppError> {}

    pub async fn get_match_logs(
        &self,
        user_id: Uuid,
        match_id: Uuid,
    ) -> Result<Vec<TurnLogResponse>, AppError> {
        let turns = self.repos.turn_repo.get_turns(match_id).await?;
        Ok(turns)
    }

    pub async fn get_participants(
        &self,
        _user_id: Uuid,
        match_id: Uuid,
    ) -> Result<Vec<GetParticipantsResponse>, AppError> {
        let participants = self
            .repos
            .participation_repo
            .get_participants(match_id)
            .await?;
        Ok(participants)
    }

    pub async fn get_gametypes(&self) -> Result<Vec<GetGameTypeResponse>, AppError> {
        let game_types = self.repos.gametype_repo.get_game_types().await?;
        Ok(game_types)
    }
}
