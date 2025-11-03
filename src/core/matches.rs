use std::{cmp::Ordering, collections::HashMap, iter::zip, sync::Arc};

use chrono::Utc;
use serde_json::json;
use tackle_box::contracts::payloads_v1::{
    GetGameTypeResponse, GetMatchResponse, GetOnlineMatchResponse, GetParticipantsResponse,
    NewMatchPayload, TurnLogResponse,
};
use tokio::task::JoinHandle;
use uuid::Uuid;

use crate::{
    api::error::AppError,
    core::orchestrator::{GameSettlement, OrchestratorService, TurnLog},
    repo::{
        agents::AgentRepo,
        game_type::GameTypeRepo,
        matches::{GetMatchDTO, MatchRepo, MatchStatus, NewMatchDTO},
        participation::ParticipationRepo,
        turns::{NewTurnDTO, TurnRepo},
        users::UserRepo,
    },
};

pub struct MatchService {
    pub gametype_repo: Arc<GameTypeRepo>,
    pub user_repo: Arc<UserRepo>,
    pub agent_repo: Arc<AgentRepo>,
    pub match_repo: Arc<MatchRepo>,
    pub turn_repo: Arc<TurnRepo>,
    pub participation_repo: Arc<ParticipationRepo>,
    pub orchestrator_service: Arc<OrchestratorService>,
}

impl MatchService {
    pub fn new(
        gametype_repo: Arc<GameTypeRepo>,
        user_repo: Arc<UserRepo>,
        agent_repo: Arc<AgentRepo>,
        match_repo: Arc<MatchRepo>,
        turn_repo: Arc<TurnRepo>,
        participation_repo: Arc<ParticipationRepo>,
        orchestrator_service: Arc<OrchestratorService>,
    ) -> Self {
        Self {
            gametype_repo,
            user_repo,
            agent_repo,
            match_repo,
            turn_repo,
            participation_repo,
            orchestrator_service,
        }
    }

    pub async fn get_match(&self, match_name: &String) -> Result<GetMatchResponse, AppError> {
        let one_match = self.match_repo.get_match(&match_name).await?;

        let status = match one_match.status {
            MatchStatus::Cancelled => "Cancelled".to_string(),
            MatchStatus::Completed => "Completed".to_string(),
            MatchStatus::Pending => "Pending".to_string(),
            MatchStatus::Running => "Running".to_string(),
        };

        let one_match = GetMatchResponse {
            id: one_match.match_id,
            name: match_name.clone(),
            game_type: one_match.game_type,
            total_games: one_match.total_games,
            creater_name: one_match.creater_username,
            winner_agent_name: one_match.winner_agent_readable_name,
            start_time: one_match.start_time,
            end_time: one_match.end_time,
            status,
        };
        Ok(one_match)
    }

    pub async fn get_online_matches(
        &self,
        _user_id: Uuid,
    ) -> Result<Vec<GetOnlineMatchResponse>, AppError> {
        let matches = self.match_repo.get_online_matches().await?;
        let matches: Vec<GetOnlineMatchResponse> = matches
            .into_iter()
            .map(|m| GetOnlineMatchResponse {
                match_id: m.match_id,
                match_name: m.readable_match_name,
                creater_name: m.creater_username,
                total_games: m.total_games,
                game_type: m.game_type,
                with_password: m.password.is_some(),
                start_time: m.start_time.to_string(),
                status: m.status.into(),
            })
            .collect();
        Ok(matches)
    }

    pub async fn get_my_matches(&self, user_id: Uuid) -> Result<Vec<GetMatchResponse>, AppError> {
        let matches = self.match_repo.get_matches(user_id).await?;
        let matches: Vec<GetMatchResponse> = matches
            .into_iter()
            .map(|m| GetMatchResponse {
                id: m.match_id,
                name: m.match_name_base,
                creater_name: m.creater_username,
                total_games: m.total_games,
                game_type: m.game_type,
                winner_agent_name: m.winner_agent_readable_name,
                start_time: m.start_time,
                end_time: m.end_time,
                status: m.status.into(),
            })
            .collect();
        Ok(matches)
    }

    pub async fn new_match(
        &self,
        user_id: Uuid,
        one_match: NewMatchPayload,
    ) -> Result<Uuid, AppError> {
        let NewMatchPayload {
            name,
            game_type,
            total_games,
            with_agent_names,
            password,
        } = one_match;

        let one_match = NewMatchDTO {
            name: name.clone(),
            game_type,
            total_games,
            creater_id: user_id,
            password,
        };
        let match_id = self.match_repo.new_match(one_match).await?;
        for agent_name in &with_agent_names {
            self.join_match(user_id, &name, agent_name).await?;
        }
        Ok(match_id)
    }

    pub async fn join_match(
        &self,
        user_id: Uuid,
        match_name: &String,
        agent_name: &String,
    ) -> Result<(), AppError> {
        let one_match = self.match_repo.get_match(match_name).await?;
        let match_id = one_match.match_id;
        let match_status: MatchStatus = self.match_repo.get_match_status(match_id).await?;
        if match_status != MatchStatus::Pending {
            return Err(AppError::Internal(
                "match is not pending, cannot join".to_string(),
            ));
        }

        let agent_id = self
            .agent_repo
            .get_agent_id_by_name(user_id, &agent_name)
            .await?;

        let exists = self
            .orchestrator_service
            .exists_agent_connection(agent_id)
            .await?;

        if exists {
            self.participation_repo
                .insert_participant(match_id, agent_id)
                .await?;
            let count = self.participation_repo.count_participants(match_id).await?;
            if count == 2 {
                let GetMatchDTO {
                    game_type,
                    total_games,
                    ..
                } = self.match_repo.get_match_by_id(match_id).await?;
                let agent_ids = self.participation_repo.get_participants(match_id).await?;
                let match_handle = self
                    .orchestrator_service
                    .run_game(match_id, game_type, total_games, agent_ids)
                    .await?;

                let match_repo = self.match_repo.clone();
                let agent_repo = self.agent_repo.clone();
                let turn_repo = self.turn_repo.clone();

                let handle: JoinHandle<Result<(), AppError>> = tokio::spawn(async move {
                    if let Ok(Ok(game_settlement)) = match_handle.await {
                        let GameSettlement {
                            match_id,
                            agent_ids,
                            connections,
                            logs,
                        } = game_settlement;

                        let mut score_map: HashMap<Uuid, f32> =
                            agent_ids.iter().map(|&id| (id, 0.0)).collect();

                        let mut tx = match_repo.get_transaction().await?;
                        for (i_turn, log) in logs.into_iter().enumerate() {
                            let TurnLog {
                                logs: turn_log,
                                payoffs,
                            } = log;
                            let score_deltas: HashMap<Uuid, f32> =
                                HashMap::from_iter(zip(agent_ids.clone(), payoffs));

                            for (id, score) in &score_deltas {
                                score_map.entry(*id).and_modify(|s| *s += score);
                            }

                            let turn = NewTurnDTO {
                                match_id,
                                i_turn: i_turn as i32,
                                log: json!(turn_log),
                                score_deltas: json!(score_deltas),
                                start_time: Utc::now(),
                                end_time: Utc::now(),
                            };
                            turn_repo.insert_turn(&mut tx, turn).await?;
                        }
                        let winner_id = score_map
                            .iter()
                            .max_by(|&(_, &v1), &(_, &v2)| {
                                v1.partial_cmp(&v2).unwrap_or(Ordering::Equal)
                            })
                            .map(|(k, _)| *k);
                        match winner_id {
                            Some(winner_id) => {
                                for agent_id in agent_ids {
                                    if winner_id == agent_id {
                                        agent_repo.agent_won(&mut tx, agent_id).await?
                                    } else {
                                        agent_repo.agent_failed(&mut tx, agent_id).await?
                                    }
                                }
                            }
                            None => {
                                for agent_id in agent_ids {
                                    agent_repo.agent_failed(&mut tx, agent_id).await?;
                                }
                            }
                        };

                        match_repo
                            .update_match_final_status(&mut tx, match_id, winner_id)
                            .await?;
                        tx.commit().await.unwrap();
                    }

                    Ok(())
                });
            }
        }

        Ok(())
    }

    // pub async fn leave_match(&self, user_id: Uuid, agent_name: &String) -> Result<(), AppError> {}

    pub async fn get_match_logs(
        &self,
        user_id: Uuid,
        match_name: &String,
    ) -> Result<Vec<TurnLogResponse>, AppError> {
        let turns = self.turn_repo.get_turns_by_match_name(match_name).await?;
        Ok(turns
            .into_iter()
            .map(|turn| TurnLogResponse {
                id: turn.match_id,
                match_name: turn.readable_match_name,
                log: turn.log,
                i_turn: turn.i_turn,
                score_deltas: turn.score_deltas,
            })
            .collect())
    }

    pub async fn get_participants(
        &self,
        _user_id: Uuid,
        match_name: &String,
    ) -> Result<Vec<GetParticipantsResponse>, AppError> {
        let participants = self
            .participation_repo
            .get_participants_by_match_name(match_name)
            .await?;
        let participants = participants
            .into_iter()
            .map(|agent_name| GetParticipantsResponse {
                match_name: match_name.clone(),
                agent_name: agent_name.clone(),
            })
            .collect();
        Ok(participants)
    }

    pub async fn get_gametypes(&self) -> Result<Vec<GetGameTypeResponse>, AppError> {
        let game_types = self.gametype_repo.get_geme_types().await?;
        let game_types = game_types
            .into_iter()
            .map(|game_type| GetGameTypeResponse {
                game_type: game_type.name,
                description: game_type.description,
            })
            .collect();
        Ok(game_types)
    }

    pub async fn settle_match(&self, match_id: Uuid) -> Result<(), AppError> {
        let mut tx = self.match_repo.get_transaction().await?;
        let all_turns = self.turn_repo.get_all_turns(&mut tx, match_id).await?;

        let mut total_score = HashMap::new();
        for turn in all_turns {
            let score_deltas: HashMap<Uuid, i32> = serde_json::from_value(turn.score_deltas)?;
            for (agent_id, delta) in score_deltas {
                total_score
                    .entry(agent_id)
                    .and_modify(|s| *s += delta)
                    .or_insert(0);
            }
        }
        let winner_id = total_score.iter().max_by_key(|&(_, &v)| v).map(|(k, _)| *k);

        self.match_repo
            .update_match_final_status(&mut tx, match_id, winner_id)
            .await?;
        Ok(())
    }
}
