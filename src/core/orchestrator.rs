use crate::{
    api::{
        error::AppError,
        extractor::{check_jwt, Claims},
    },
    repo::{agents::AgentRepo, matches::MatchRepo, turns::NewTurnDTO},
};
use base64::prelude::BASE64_STANDARD;
use base64::prelude::*;
use futures_util::{Stream, StreamExt};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, pin::Pin, sync::Arc};
use tackle_box::{
    connection::{
        client_service_server::{self, ClientServiceServer},
        game_control::ControlType,
        process_game_request::RequestType,
        process_game_response::ResponseType,
        sponsor_service_client::SponsorServiceClient,
        GameControl, GameEndStatus, GameInitRequest, GameStateUpdate, MatchMonitorRequest,
        MatchMonitorResponse, MatchPlayerRequest, MatchPlayerResponse, PlayerAction,
        ProcessGameRequest, ProcessGameResponse,
    },
    contracts::grpc::MatchMetadata,
};
use tokio::{
    sync::{
        mpsc::{self, Receiver, Sender},
        oneshot, Mutex,
    },
    task::JoinHandle,
};
use tokio_stream::wrappers::ReceiverStream;
use tonic::{
    transport::{Channel, Server},
    Request, Response, Status, Streaming,
};
use tracing::{debug, error};
use uuid::Uuid;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct MessageMetadata {
    pub user_id: Uuid,
    pub metadata: MatchMetadata,
}

pub enum MatchMessage {
    AgentReady { match_id: Uuid, agent_id: Uuid },
    TurnLog(NewTurnDTO),
    MatchSettlement { match_id: Uuid },
}

#[derive(Debug)]
pub struct PlayerConnection {
    tx: Sender<Result<MatchPlayerResponse, Status>>,
    rx: Option<Streaming<MatchPlayerRequest>>,
}

pub struct TurnLog {
    pub logs: Vec<String>,
    pub payoffs: Vec<f32>,
}

pub struct GameSettlement {
    pub match_id: Uuid,
    pub agent_ids: Vec<Uuid>,
    pub connections: Vec<PlayerConnection>,
    pub logs: Vec<TurnLog>,
}

pub struct OrchestratorService {
    // match_service: Arc<MatchService>,
    agent_repo: Arc<AgentRepo>,
    match_repo: Arc<MatchRepo>,
    monitor_connections: Mutex<HashMap<Uuid, Vec<Sender<Result<MatchMonitorResponse, Status>>>>>,
    player_connections: Mutex<HashMap<Uuid, PlayerConnection>>,
    sponsor: Arc<SponsorServiceClient<Channel>>,
}

impl OrchestratorService {
    pub async fn new(
        agent_repo: Arc<AgentRepo>,
        match_repo: Arc<MatchRepo>,
        game_server_url: String, // 接收 Game Server 的地址
    ) -> Result<Self, AppError> {
        let sponser = SponsorServiceClient::connect(game_server_url).await?;
        Ok(OrchestratorService {
            agent_repo,
            match_repo,
            monitor_connections: Mutex::new(HashMap::new()),
            player_connections: Mutex::new(HashMap::new()),
            sponsor: Arc::new(sponser),
        })
    }

    pub async fn run_game(
        &self,
        match_id: Uuid,
        game_type: String,
        total_games: i32,
        agent_ids: Vec<Uuid>,
    ) -> Result<JoinHandle<Result<GameSettlement, AppError>>, AppError> {
        debug!("try run game");

        let mut sponsor = (*self.sponsor).clone();
        let mut connections = self.player_connections.lock().await;
        let mut agent_conns = HashMap::new();
        for agent_id in agent_ids.clone() {
            let conn = connections
                .get_mut(&agent_id)
                .ok_or(AppError::Internal("not connect".to_string()))?;
            let PlayerConnection { tx, rx } = conn;
            let tx = tx.clone();
            let rx = match rx.take() {
                Some(rx) => rx,
                None => {
                    return Err(AppError::Internal(
                        "no found connection or still running".to_string(),
                    ))
                }
            };
            agent_conns.insert(agent_id, (tx, rx));
        }

        let handle: JoinHandle<Result<GameSettlement, AppError>> = tokio::spawn(async move {
            let mut game_logs = Vec::new();
            let init_req = ProcessGameRequest {
                request_type: Some(RequestType::Init(GameInitRequest { game_type })),
            };
            let (tx, rx) = mpsc::channel(16);
            let request_stream = ReceiverStream::new(rx);

            let resp = sponsor.process_game(Request::new(request_stream)).await?;

            let cloned_tx = tx.clone();

            tokio::spawn(async move {
                cloned_tx.send(init_req).await.expect("no send");
            });

            // init successful
            let mut resp_stream = resp.into_inner();

            // 丢弃一次应答
            let _ = resp_stream.next().await;
            if let Some(Ok(ProcessGameResponse { response_type })) = resp_stream.next().await {
                match response_type {
                    Some(ResponseType::InitResponse(_)) => {}
                    _ => {
                        return Err(AppError::Internal("failed to init game".to_string()));
                    }
                }
            }
            for i_turn in 0..total_games {
                let mut turn_log = Vec::new();
                while let Some(Ok(ProcessGameResponse { response_type })) = resp_stream.next().await
                {
                    match response_type {
                        Some(ResponseType::StateUpdate(data)) => {
                            let GameStateUpdate {
                                state,
                                is_over,
                                i_player,
                            } = data;
                            let player_id = agent_ids[i_player as usize];
                            let (player_tx, player_rx) = agent_conns.get_mut(&player_id).unwrap();
                            player_tx
                                .send(Ok(MatchPlayerResponse {
                                    state: state.clone(),
                                }))
                                .await
                                .expect("no send");
                            turn_log.push(format!("env: {}", state));
                            let action = match player_rx.next().await {
                                Some(Ok(MatchPlayerRequest { action })) => action,
                                _ => return Err(AppError::Internal("get wrong info".to_string())),
                            };
                            tx.send(ProcessGameRequest {
                                request_type: Some(RequestType::Action(PlayerAction {
                                    action: action.clone(),
                                })),
                            })
                            .await
                            .expect("no send");
                            turn_log.push(format!("agent {}: {}", player_id, action));
                        }
                        Some(ResponseType::EndStatus(data)) => {
                            let GameEndStatus { payoffs } = data;

                            let turn_log = TurnLog {
                                logs: turn_log,
                                payoffs,
                            };
                            game_logs.push(turn_log);

                            if i_turn == total_games - 1 {
                                tx.send(ProcessGameRequest {
                                    request_type: Some(RequestType::Control(GameControl {
                                        r#type: ControlType::Pause.into(),
                                    })),
                                })
                                .await
                                .expect("no send");

                                return Ok(GameSettlement {
                                    match_id,
                                    agent_ids: agent_conns.keys().cloned().collect(),
                                    connections: agent_conns
                                        .into_values()
                                        .map(|(player_tx, player_rx)| PlayerConnection {
                                            tx: player_tx,
                                            rx: Some(player_rx),
                                        })
                                        .collect(),
                                    logs: game_logs,
                                });
                            } else {
                                tx.send(ProcessGameRequest {
                                    request_type: Some(RequestType::Control(GameControl {
                                        r#type: ControlType::Resume.into(),
                                    })),
                                })
                                .await
                                .expect("no send");
                                break;
                            }
                        }
                        _ => return Err(AppError::Internal("unknow error".to_string())),
                    }
                }
            }
            Err(AppError::Internal("failed to settle game".to_string()))
        });

        Ok(handle)
    }

    pub async fn exists_agent_connection(&self, agent_id: Uuid) -> Result<bool, AppError> {
        Ok(self.player_connections.lock().await.contains_key(&agent_id))
    }

    async fn new_monitor_connection(
        &self,
        user_id: Uuid,
        match_name: String,
        monitor_tx: Sender<Result<MatchMonitorResponse, Status>>,
    ) -> Result<(), AppError> {
        let mut connections = self.monitor_connections.lock().await;
        let match_id = self
            .match_repo
            .get_match_id_by_name(user_id, &match_name)
            .await?;
        if let Some(conn) = connections.get_mut(&user_id) {
            conn.push(monitor_tx);
        } else {
            connections.insert(match_id, vec![monitor_tx]);
        }

        Ok(())
    }

    async fn new_player_connection(
        &self,
        user_id: Uuid,
        agent_name: String,
        in_stream: Streaming<MatchPlayerRequest>,
        player_tx: Sender<Result<MatchPlayerResponse, Status>>,
    ) -> Result<(), AppError> {
        let mut connections = self.player_connections.lock().await;
        let agent_id = self
            .agent_repo
            .get_agent_id_by_name(user_id, &agent_name)
            .await?;

        connections.insert(
            agent_id,
            PlayerConnection {
                tx: player_tx,
                rx: Some(in_stream),
            },
        );

        Ok(())
    }
}

// Server face to User/Client

#[derive(Clone)]
pub struct ClientServer {
    orchestrator_service: Arc<OrchestratorService>,
}

impl ClientServer {
    async fn new(orchestrator_service: Arc<OrchestratorService>) -> Self {
        Self {
            orchestrator_service,
        }
    }
}

type MessageResult<T> = Result<Response<T>, Status>;
type MatchMonitorStream = Pin<Box<dyn Stream<Item = Result<MatchMonitorResponse, Status>> + Send>>;
type MatchPlayerStream = Pin<Box<dyn Stream<Item = Result<MatchPlayerResponse, Status>> + Send>>;

#[tonic::async_trait]
impl client_service_server::ClientService for ClientServer {
    type MatchMonitorStream = MatchMonitorStream;
    type MatchPlayerStream = MatchPlayerStream;

    async fn match_monitor(
        &self,
        req: Request<MatchMonitorRequest>,
    ) -> MessageResult<MatchMonitorStream> {
        let (monitor_tx, rx) = mpsc::channel(8);

        let (user_id, match_name) = match req.extensions().get::<MessageMetadata>().cloned() {
            Some(data) => match data.metadata {
                MatchMetadata::MatchMonitor { match_name } => (data.user_id, match_name),
                _ => return Err(Status::aborted("type error")),
            },
            None => return Err(Status::aborted("no user auth information")),
        };
        // let MatchMonitorRequest { match_name } = req.get_ref();
        self.orchestrator_service
            .new_monitor_connection(user_id, match_name.clone(), monitor_tx)
            .await;

        let monitor_stream: ReceiverStream<_> = ReceiverStream::new(rx);
        Ok(Response::new(
            Box::pin(monitor_stream) as Self::MatchMonitorStream
        ))
    }

    async fn match_player(
        &self,
        req: Request<Streaming<MatchPlayerRequest>>,
    ) -> MessageResult<MatchPlayerStream> {
        let (player_tx, rx) = mpsc::channel(8);
        let (user_id, agent_name) = match req.extensions().get::<MessageMetadata>().cloned() {
            Some(data) => match data.metadata {
                MatchMetadata::MatchPlayer { agent_name } => (data.user_id, agent_name),
                _ => return Err(Status::aborted("type error")),
            },
            None => return Err(Status::aborted("no user auth information")),
        };
        let in_stream = req.into_inner();
        let out_stream = ReceiverStream::new(rx);
        self.orchestrator_service
            .new_player_connection(user_id, agent_name, in_stream, player_tx)
            .await;
        Ok(Response::new(
            Box::pin(out_stream) as Self::MatchPlayerStream
        ))
    }
}

fn check_auth(mut req: Request<()>) -> Result<Request<()>, Status> {
    debug!("someone connected");
    let token_value = match req.metadata().get("authorization") {
        Some(t) => t,
        _ => return Err(tonic::Status::unauthenticated("No auth token")),
    };
    debug!("get token value: {:?}", token_value);
    let authenticated_data: String = token_value.to_str().unwrap().to_owned();
    let token = authenticated_data
        .strip_prefix("Bearer ")
        .ok_or(Status::aborted("no authorization"))?;

    debug!("get token: {:?}", token);
    let claims = check_jwt(token).map_err(|_| Status::aborted("authorization failed"))?;
    let Claims {
        user_id,
        username,
        exp,
        iat,
    } = claims;
    debug!("get username: {}", username);

    let metadata = match req.metadata().get("x-message-metadata") {
        Some(metadata_value) => {
            let base64_bytes = metadata_value.as_bytes();
            debug!("serde base64 to bytes");
            if let Ok(decoded_bytes) = BASE64_STANDARD.decode(base64_bytes) {
                let metadata: MatchMetadata = serde_json::from_slice(&decoded_bytes)
                    .map_err(|_| Status::aborted("serde error".to_string()))?;
                metadata
            } else {
                return Err(Status::aborted("decode error"));
            }
        }
        None => MatchMetadata::None,
    };
    debug!("metadata: {:?}", &metadata);

    req.extensions_mut()
        .insert(MessageMetadata { user_id, metadata });
    Ok(req)
}

pub async fn run_client_server(service: Arc<OrchestratorService>) -> Result<(), AppError> {
    let addr = "[::]:50050".parse().unwrap();
    let server = ClientServer::new(service).await;
    Server::builder()
        .add_service(ClientServiceServer::with_interceptor(server, check_auth))
        .serve(addr)
        .await;
    Ok(())
}

// // Server face to Sponsor

// pub struct SponsorServer {
//     orchestrator_service: Arc<OrchestratorService>,
// }

// impl SponsorServer {
//     async fn new(orchestrator_service: Arc<OrchestratorService>) -> Self {
//         Self {
//             orchestrator_service,
//         }
//     }
// }

// type GameResult<T> = Result<Response<T>, Status>;
// type ProcessGameStream = Pin<Box<dyn Stream<Item = Result<ProcessGameResponse, Status>> + Send>>;

// #[tonic::async_trait]
// impl sponsor_service_server::SponsorService for SponsorServer {
//     type ProcessGameStream = ProcessGameStream;

//     async fn process_game(
//         &self,
//         req: Request<Streaming<ProcessGameRequest>>,
//     ) -> GameResult<ProcessGameStream> {
//         let (tx, rx) = mpsc::channel(1024);
//         let in_stream = req.into_inner();
//         let out_stream = ReceiverStream::new(rx);
//         tokio::spawn(async move {});
//         Ok(Response::new(
//             Box::pin(out_stream) as Self::ProcessGameStream
//         ))
//     }
// }

// pub async fn run_sponsor_server(service: Arc<OrchestratorService>) -> Result<(), AppError> {
//     let addr = "[::]:50051".parse().unwrap();
//     let server = SponsorServer::new(service);
//     Server::builder()
//         .add_service(SponsorServiceServer::new(server))
//         .serve(addr)
//         .await;
//     Ok(())
// }
