use crate::{
    api::{
        error::AppError,
        extractor::{check_jwt, Claims},
    },
    core::core::CoreMessage,
};
use base64::prelude::BASE64_STANDARD;
use base64::prelude::*;
use futures_util::{Stream, StreamExt};
use serde::{Deserialize, Serialize};
use std::{pin::Pin, sync::Arc};
use tackle_box::{
    connection::{
        client_service_server::{self, ClientServiceServer},
        MatchMonitorRequest, MatchMonitorResponse, MatchPlayerRequest, MatchPlayerResponse,
    },
    contracts::grpc::MatchMetadata,
};
use tokio::sync::mpsc::{self, Sender};
use tokio_stream::wrappers::ReceiverStream;
use tonic::{transport::Server, Request, Response, Status, Streaming};
use tracing::debug;
use uuid::Uuid;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct MessageMetadata {
    pub user_id: Uuid,
    pub metadata: MatchMetadata,
}

pub struct ClientService {
    core_tx: Sender<CoreMessage>,
}

impl ClientService {
    pub async fn new(core_tx: Sender<CoreMessage>) -> Result<Arc<Self>, AppError> {
        Ok(Arc::new(ClientService { core_tx }))
    }
}

// Server face to User/Client

#[derive(Clone)]
pub struct ClientServer {
    client_service: Arc<ClientService>,
}

impl ClientServer {
    async fn new(client_service: Arc<ClientService>) -> Self {
        Self { client_service }
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

        let (user_id, match_id) = match req.extensions().get::<MessageMetadata>().cloned() {
            Some(data) => match data.metadata {
                MatchMetadata::MatchMonitor { match_id } => (data.user_id, match_id),
                _ => return Err(Status::aborted("type error")),
            },
            None => return Err(Status::aborted("no user auth information")),
        };

        let monitor_stream: ReceiverStream<_> = ReceiverStream::new(rx);
        Ok(Response::new(
            Box::pin(monitor_stream) as Self::MatchMonitorStream
        ))
    }

    async fn match_player(
        &self,
        req: Request<Streaming<MatchPlayerRequest>>,
    ) -> MessageResult<MatchPlayerStream> {
        let (client_tx, rx) = mpsc::channel(8);
        let (user_id, agent_id) = match req.extensions().get::<MessageMetadata>().cloned() {
            Some(data) => match data.metadata {
                MatchMetadata::MatchPlayer { agent_id } => (data.user_id, agent_id),
                _ => return Err(Status::aborted("type error")),
            },
            None => return Err(Status::aborted("no user auth information")),
        };
        let mut client_instream = req.into_inner();
        let out_stream = ReceiverStream::new(rx);

        let client_service = self.client_service.clone();
        let core_tx = self.client_service.core_tx.clone();

        let mut client = Client {
            user_id,
            agent_id,
            match_id: None,
            core_tx,
            client_tx,
            client_instream,
        };
        tokio::spawn(async move {
            client.run().await;
        });

        Ok(Response::new(
            Box::pin(out_stream) as Self::MatchPlayerStream
        ))
    }
}

fn check_auth(mut req: Request<()>) -> Result<Request<()>, Status> {
    let token_value = match req.metadata().get("authorization") {
        Some(t) => t,
        _ => return Err(tonic::Status::unauthenticated("No auth token")),
    };
    let authenticated_data: String = token_value.to_str().unwrap().to_owned();
    let token = authenticated_data
        .strip_prefix("Bearer ")
        .ok_or(Status::aborted("no authorization"))?;

    let claims = check_jwt(token).map_err(|_| Status::aborted("authorization failed"))?;
    let Claims {
        user_id,
        username,
        exp,
        iat,
    } = claims;

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

    req.extensions_mut()
        .insert(MessageMetadata { user_id, metadata });
    Ok(req)
}

pub async fn run_client_server(service: Arc<ClientService>) -> Result<(), AppError> {
    let addr = "[::]:50050".parse().unwrap();
    let server = ClientServer::new(service).await;
    Server::builder()
        .add_service(ClientServiceServer::with_interceptor(server, check_auth))
        .serve(addr)
        .await;
    Ok(())
}

struct Client {
    user_id: Uuid,
    agent_id: Uuid,
    match_id: Option<Uuid>,

    core_tx: Sender<CoreMessage>,
    client_tx: Sender<Result<MatchPlayerResponse, Status>>,
    client_instream: Streaming<MatchPlayerRequest>,
}

impl Client {
    async fn run(&mut self) -> Result<(), AppError> {
        let (tx, mut rx) = mpsc::channel(8);
        self.regiser(tx).await?;
        loop {
            tokio::select! {
                msg = rx.recv() => {
                    let Some(msg) = msg else {
                        tracing::info!("Internal channel closed. Shutting down client handler.");
                        break;
                    };
                    self.process_core_message(msg).await?;
                },

                resp = self.client_instream.next() => {
                    match resp {
                        Some(Ok(resp)) => {
                            self.porcess_client_resp(resp).await?;
                        }
                        Some(Err(e)) => {
                            tracing::error!("Client stream error: {:?}", e);
                            break;
                        }
                        None => {
                            tracing::info!("Client stream closed normally. Shutting down client handler.");
                            break;
                        }
                    }
                },
            }
        }
        self.unregister().await?;
        Ok(())
    }
    async fn regiser(&mut self, tx: Sender<CoreMessage>) -> Result<(), AppError> {
        self.core_tx
            .send(CoreMessage::ClientRegiser {
                user_id: self.user_id,
                agent_id: self.agent_id,
                tx,
            })
            .await?;
        Ok(())
    }
    async fn unregister(&mut self) -> Result<(), AppError> {
        self.core_tx
            .send(CoreMessage::ClientUnregiser {
                user_id: self.user_id,
                agent_id: self.agent_id,
            })
            .await?;
        Ok(())
    }
    async fn process_core_message(&mut self, msg: CoreMessage) -> Result<(), AppError> {
        match msg {
            CoreMessage::GameState {
                agent_id,
                match_id,
                state,
            } => {
                self.match_id = Some(match_id);
                self.client_tx
                    .send(Ok(MatchPlayerResponse { state }))
                    .await
                    .map_err(|e| AppError::Internal("trans error".to_string()))?;
            }
            _ => {}
        }
        Ok(())
    }

    async fn porcess_client_resp(&mut self, resp: MatchPlayerRequest) -> Result<(), AppError> {
        let MatchPlayerRequest { action } = resp;
        self.core_tx
            .send(CoreMessage::AgentAction {
                agent_id: self.agent_id,
                match_id: self.match_id.unwrap(),
                action,
            })
            .await?;
        Ok(())
    }
}
