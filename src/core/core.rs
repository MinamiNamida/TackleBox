use chrono::Utc;
use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::{clone, cmp::Ordering, collections::HashMap, iter::zip, sync::Arc};
use tackle_box::{
    connection::{
        GameControl, GameEndStatus, GameInitRequest, GameStateUpdate, PlayerAction, ProcessGameRequest, ProcessGameResponse, game_control::ControlType, process_game_request::RequestType, process_game_response::ResponseType, sponsor_service_client::SponsorServiceClient
    },
    contracts::payloads::{AgentStatus, MatchStatus},
};
use tokio::sync::mpsc::{self, Receiver, Sender};
use tokio_stream::wrappers::ReceiverStream;
use tonic::{transport::Channel, Request, Streaming};
use tracing::debug;
use uuid::Uuid;

use crate::{
    api::error::AppError,
    repo::{
        agents::AgentRepo,
        matches::MatchRepo,
        turns::{NewTurnDTO, TurnRepo},
    },
};

pub enum CoreMessage {
    ClientRegiser {
        user_id: Uuid,
        agent_id: Uuid,
        tx: Sender<CoreMessage>,
    },
    ClientUnregiser {
        user_id: Uuid,
        agent_id: Uuid,
    },
    MatchStart {
        match_id: Uuid,
        agent_ids: Vec<Uuid>,
        sponsor: String,
        game_type: String,
        total_games: i32,
    },
    AgentAction {
        agent_id: Uuid,
        match_id: Uuid,
        action: String,
    },
    GameState {
        agent_id: Uuid,
        match_id: Uuid,
        state: String,
    },
    MatchPause {
        match_id: Uuid,
    },
    MatchSettle {
        match_id: Uuid,
        settler: GameSettlement,
    },
}

struct Connections {
    clients: HashMap<Uuid, Sender<CoreMessage>>,
    sponsors: HashMap<String, SponsorServiceClient<Channel>>,
    matches: HashMap<Uuid, Sender<CoreMessage>>,
    tx: Sender<CoreMessage>,
    rx: Receiver<CoreMessage>,
}

struct Repos {
    match_repo: Arc<MatchRepo>,
    agent_repo: Arc<AgentRepo>,
    turn_repo: Arc<TurnRepo>,
}

pub struct Core {
    connections: Connections,
    repos: Repos,
}

impl Core {
    pub async fn new(
        sponsor_urls: HashMap<String, String>,
        match_repo: Arc<MatchRepo>,
        agent_repo: Arc<AgentRepo>,
        turn_repo: Arc<TurnRepo>,
    ) -> Result<Self, AppError> {
        let (tx, rx) = mpsc::channel(8);
        let mut sponsors = HashMap::new();
        for (name, url) in sponsor_urls {
            let sponsor = SponsorServiceClient::connect(url).await?;
            sponsors.insert(name, sponsor);
        }
        let clients = HashMap::new();
        let matches = HashMap::new();
        Ok(Self {
            connections: Connections {
                tx,
                rx,
                sponsors,
                clients,
                matches,
            },
            repos: Repos {
                match_repo,
                agent_repo,
                turn_repo,
            },
        })
    }

    pub fn tx(&self) -> Sender<CoreMessage> {
        self.connections.tx.clone()
    }
    pub async fn run(&mut self) -> Result<(), AppError> {
        loop {
            while let Some(msg) = self.connections.rx.recv().await {
                match msg {
                    CoreMessage::ClientRegiser {
                        user_id,
                        agent_id,
                        tx,
                    } => {
                        self.process_client_register(agent_id, user_id, tx).await?;
                    }
                    CoreMessage::ClientUnregiser { user_id, agent_id } => {
                        self.process_client_unregiser(agent_id, user_id).await?;
                    }
                    CoreMessage::AgentAction {
                        agent_id,
                        match_id,
                        action,
                    } => {
                        self.process_agent_action(agent_id, match_id, action)
                            .await?;
                    }
                    CoreMessage::GameState {
                        agent_id,
                        match_id,
                        state,
                    } => {
                        self.process_game_state(agent_id, match_id, state).await?;
                    }
                    CoreMessage::MatchStart {
                        match_id,
                        agent_ids,
                        sponsor,
                        game_type,
                        total_games,
                    } => {
                        self.process_match_start(
                            match_id,
                            agent_ids,
                            sponsor,
                            game_type,
                            total_games,
                        )
                        .await?;
                    }
                    CoreMessage::MatchPause { match_id } => {
                        self.process_match_pause(match_id).await?;
                    }
                    CoreMessage::MatchSettle { match_id, settler } => {
                        self.process_match_settle(settler).await?;
                    }
                }
            }
        }
        Ok(())
    }

    async fn process_client_register(
        &mut self,
        agent_id: Uuid,
        user_id: Uuid,
        tx: Sender<CoreMessage>,
    ) -> Result<(), AppError> {
        debug!("client regisered");
        self.connections.clients.insert(agent_id, tx);
        self.repos
            .agent_repo
            .update_agent_status(agent_id, AgentStatus::Ready)
            .await?;
        debug!("updated agent status");
        Ok(())
    }

    async fn process_client_unregiser(
        &mut self,
        agent_id: Uuid,
        user_id: Uuid,
    ) -> Result<(), AppError> {
        let _ = self.connections.clients.remove(&agent_id);
        self.repos
            .agent_repo
            .update_agent_status(agent_id, AgentStatus::Idle)
            .await?;
        Ok(())
    }

    async fn process_game_state(
        &mut self,
        agent_id: Uuid,
        match_id: Uuid,
        state: String,
    ) -> Result<(), AppError> {
        let client = self
            .connections
            .clients
            .get(&agent_id)
            .ok_or(AppError::Internal("no found Error".to_string()))?;
        client
            .send(CoreMessage::GameState {
                agent_id,
                match_id,
                state,
            })
            .await?;
        Ok(())
    }

    async fn process_match_start(
        &mut self,
        match_id: Uuid,
        agent_ids: Vec<Uuid>,
        sponsor: String,
        game_type: String,
        total_games: i32,
    ) -> Result<(), AppError> {
        let (match_tx, match_rx) = mpsc::channel(8);
        let core_tx = self.tx();
        let (sponsor_tx, sponsor_rx) = match self.connections.sponsors.get_mut(&sponsor) {
            Some(sponsor) => {
                let (sponser_tx, sponsor_rx) = mpsc::channel(16);
                let init_req = ProcessGameRequest {
                    request_type: Some(RequestType::Init(GameInitRequest {
                        game_type: game_type.clone(),
                    })),
                };
                let request_stream = ReceiverStream::new(sponsor_rx);
                let resp = sponsor.process_game(Request::new(request_stream)).await?;
                sponser_tx.send(init_req).await?;
                let mut sponsor_instream = resp.into_inner();
                // 丢弃一次应答, 因为Python后段依赖至少一次回复来生成流
                let _ = sponsor_instream.next().await;
                (sponser_tx, sponsor_instream)
            }
            None => return Err(AppError::Internal("Not Find Sponsor".to_string())),
        };
        self.connections.matches.insert(match_id, match_tx);
        let mut match_runner = MatchRunner {
            match_id,
            agent_ids,
            sponsor,
            game_type,
            total_games,
            match_rx,
            core_tx,
            sponsor_tx,
            sponsor_rx,
            i_turn: 0,
            turn_log: Some(Vec::new()),
            game_logs: Some(Vec::new()),
        };

        tokio::spawn(async move {
            match match_runner.run().await {
                Ok(_) => {}
                Err(e) => {
                    tracing::error!("{}", e.to_string());
                }
            }
        });
        Ok(())
    }

    async fn process_agent_action(
        &mut self,
        agent_id: Uuid,
        match_id: Uuid,
        action: String,
    ) -> Result<(), AppError> {
        debug!("core recv agent action");
        let match_runner = self
            .connections
            .matches
            .get(&match_id)
            .ok_or(AppError::Internal("no found Error".to_string()))?;
        match_runner
            .send(CoreMessage::AgentAction {
                agent_id,
                match_id,
                action,
            })
            .await?;
        debug!("send to match runner");
        Ok(())
    }
    pub async fn process_match_settle(&self, settler: GameSettlement) -> Result<(), AppError> {
        let GameSettlement {
            match_id,
            agent_ids,
            logs,
        } = settler;
        let Repos {
            agent_repo,
            match_repo,
            turn_repo,
            ..
        } = &self.repos;

        let mut score_map: HashMap<Uuid, f32> = agent_ids.iter().map(|&id| (id, 0.0)).collect();

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
            .max_by(|&(_, &v1), &(_, &v2)| v1.partial_cmp(&v2).unwrap_or(Ordering::Equal))
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
        Ok(())
    }

    async fn process_match_pause(&self, match_id: Uuid) -> Result<(), AppError> {
        self.repos.match_repo.update_match_status(match_id, MatchStatus::Cancelled).await?;
        Ok(())
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TurnLog {
    pub logs: Vec<GameStreamType>,
    pub payoffs: Vec<f32>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GameSettlement {
    pub match_id: Uuid,
    pub agent_ids: Vec<Uuid>,
    pub logs: Vec<TurnLog>,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum GameStreamType {
    State(String),
    Action(String),
}

struct MatchRunner {
    match_id: Uuid,
    agent_ids: Vec<Uuid>,
    sponsor: String,
    game_type: String,
    total_games: i32,

    match_rx: Receiver<CoreMessage>,
    core_tx: Sender<CoreMessage>,
    sponsor_tx: Sender<ProcessGameRequest>,
    sponsor_rx: Streaming<ProcessGameResponse>,

    i_turn: i32,
    game_logs: Option<Vec<TurnLog>>,
    turn_log: Option<Vec<GameStreamType>>,
}

impl MatchRunner {
    async fn run(&mut self) -> Result<(), AppError> {
        let loop_result: Result<(), AppError> = async {
            while self.i_turn < self.total_games {
                let r = tokio::select! {
                    Some(msg) = self.match_rx.recv() => {
                        self.process_core_message(msg).await
                    },
                    Some(Ok(resp)) = self.sponsor_rx.next() => {
                        self.process_sponsor_message(resp).await
                    },
                    else => {
                        return Err(AppError::MatchAborted("Input stream/channel closed unexpectedly.".to_string()));
                    }
                };
                r?; 
            }
            // 比赛正常完成
            Ok(())
        }.await;
        if let Err(e) = loop_result {
                // if e.is_connection_error() || e.is_match_aborted() { // 假设 AppError 有这些辅助方法
                    
                //     // 🚀 核心：更新比赛状态为 CANCELLED
                //     self.core_tx.send(CoreMessage::MatchStatusUpdate {
                //         match_id: self.match_id,
                //         status: MatchStatus::Cancelled,
                //     }).await?;
                    
                //     // 返回错误，但已经执行了清理/取消操作
                //     return Err(e);
                // } else {
                //     // 如果是其他不应该导致取消的内部逻辑错误
                //     return Err(e);
                // }
                self.core_tx.send(CoreMessage::MatchPause { match_id: self.match_id }).await?;
                return Err(e);
            }
        self.core_tx
            .send(CoreMessage::MatchSettle {
                match_id: self.match_id,
                settler: GameSettlement {
                    match_id: self.match_id,
                    agent_ids: self.agent_ids.clone(),
                    logs: self.game_logs.take().unwrap(),
                },
            })
            .await?;
        Ok(())
    }

    async fn process_core_message(&mut self, msg: CoreMessage) -> Result<(), AppError> {
        match msg {
            CoreMessage::AgentAction {
                agent_id,
                match_id,
                action,
            } => {
                self.turn_log
                    .get_or_insert(vec![])
                    .push(GameStreamType::Action(action.clone()));
                self.sponsor_tx
                    .send(ProcessGameRequest {
                        request_type: Some(RequestType::Action(PlayerAction { action: action })),
                    })
                    .await?;
            }
            _ => return Err(AppError::Internal("unknow error".to_string())),
        };
        Ok(())
    }

    async fn process_sponsor_message(&mut self, resp: ProcessGameResponse) -> Result<(), AppError> {
        let ProcessGameResponse { response_type } = resp;
        match response_type {
            Some(ResponseType::StateUpdate(data)) => {
                let GameStateUpdate {
                    state,
                    is_over,
                    i_player,
                } = data;
                let agent_id = self.agent_ids[i_player as usize];
                self.turn_log
                    .get_or_insert(vec![])
                    .push(GameStreamType::State(state.clone()));
                if !is_over {
                    self.core_tx
                        .send(CoreMessage::GameState {
                            agent_id,
                            match_id: self.match_id,
                            state: state,
                        })
                        .await?;
                }
            }
            Some(ResponseType::EndStatus(data)) => {
                let GameEndStatus { payoffs } = data;
                let turn_log = TurnLog {
                    logs: self.turn_log.take().unwrap(),
                    payoffs,
                };
                self.game_logs.get_or_insert(vec![]).push(turn_log);
                self.i_turn += 1;
                if self.i_turn == self.total_games {
                    self.sponsor_tx
                        .send(ProcessGameRequest {
                            request_type: Some(RequestType::Control(GameControl {
                                r#type: ControlType::Pause.into(),
                            })),
                        })
                        .await?;
                    // self.sponsor_tx.closed().await;
                } else {
                    self.sponsor_tx
                        .send(ProcessGameRequest {
                            request_type: Some(RequestType::Control(GameControl {
                                r#type: ControlType::Resume.into(),
                            })),
                        })
                        .await?;
                }
            }
            _ => return Err(AppError::Internal("unknow error".to_string())),
        };
        Ok(())
    }
}
