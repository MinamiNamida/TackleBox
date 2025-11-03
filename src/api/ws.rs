// use crate::{api::extractor::AuthenticatedAgent, core::orchestrator::OrchestratorService};
// use axum::{
//     extract::{
//         ws::{Message, Utf8Bytes, WebSocket},
//         State, WebSocketUpgrade,
//     },
//     response::IntoResponse,
// };
// use futures_util::{
//     stream::{SplitSink, SplitStream},
//     SinkExt, StreamExt,
// };
// use serde_json::json;
// use std::{collections::HashMap, sync::Arc};
// use thiserror::Error;
// use tokio::sync::{
//     mpsc::{self, Receiver, Sender},
//     Mutex,
// };
// use uuid::Uuid;
// use TackleBox::contracts::ws::{AgentMessage, ServerMessage};

// #[derive(Debug, Error)]
// pub enum WsError {
//     #[error("parse json from input error")]
//     Parse(#[from] serde_json::Error),
//     #[error("unused coding")]
//     Encoding,
//     #[error("handle error")]
//     Handle,
//     #[error("no exists id?")]
//     Invaid,
// }

// pub struct WsState {
//     pub ws_service: Arc<WsService>,
//     pub orchestrator_service: Arc<OrchestratorService>,
// }

// pub struct WsClient {
//     pub agent_id: Uuid,
//     pub state: WsState,
//     pub ws_tx: SplitSink<WebSocket, Message>,
//     pub ws_rx: SplitStream<WebSocket>,
//     pub rx: Receiver<ServerMessage>,
// }

// impl WsClient {
//     async fn new(
//         agent_id: Uuid,
//         state: WsState,
//         ws_tx: SplitSink<WebSocket, Message>,
//         ws_rx: SplitStream<WebSocket>,
//     ) -> Self {
//         let (tx, rx) = mpsc::channel(8);
//         state.ws_service.register(agent_id, tx).await;
//         Self {
//             agent_id,
//             state,
//             ws_rx,
//             ws_tx,
//             rx,
//         }
//     }

//     async fn parse_message(&self, msg: &String) -> Result<AgentMessage, WsError> {
//         let msg: AgentMessage = serde_json::from_str(msg)?;
//         Ok(msg)
//     }

//     async fn unwarp(&self, msg: &Message) -> Result<String, WsError> {
//         match msg {
//             Message::Text(bytes) => Ok(bytes.to_string()),
//             _ => Err(WsError::Encoding),
//         }
//     }

//     async fn encode(&self, resp: &ServerMessage) -> Result<Message, WsError> {
//         Ok(Message::Text(Utf8Bytes::from(json!(resp).to_string())))
//     }

//     async fn notice(&mut self, resp: &ServerMessage) -> Result<(), WsError> {
//         let msg = self.encode(resp).await?;
//         self.ws_tx.send(msg).await;
//         Ok(())
//     }

//     async fn handle(&mut self, msg: Message) -> Result<ServerMessage, WsError> {
//         let msg = self.parse_message(&self.unwarp(&msg).await?).await?;
//         let resp = self
//             .state
//             .orchestrator_service
//             .handle_ws_message(msg)
//             .await
//             .map_err(|_| WsError::Handle)?;
//         Ok(resp)
//     }

//     async fn handle_with_error(&mut self, msg: Message) -> Result<(), WsError> {
//         match self.handle(msg).await {
//             Ok(resp) => self.notice(&resp).await?,
//             Err(e) => {
//                 self.notice(&ServerMessage::Error {
//                     agent_id: self.agent_id,
//                     message: e.to_string(),
//                 })
//                 .await?
//             }
//         }
//         Ok(())
//     }

//     async fn run(&mut self) -> Result<(), WsError> {
//         loop {
//             tokio::select! {
//                 Some(resp) = self.rx.recv() => self.notice(&resp).await?,
//                 Some(Ok(msg)) = self.ws_rx.next() => self.handle_with_error(msg).await?,
//                 else => break,
//             }
//         }
//         Ok(())
//     }
// }

// // #[derive(Clone)]
// pub struct WsService {
//     agents: Arc<Mutex<HashMap<Uuid, Sender<ServerMessage>>>>,
//     rx: Receiver<ServerMessage>,
// }

// impl WsService {
//     pub fn new(rx: Receiver<ServerMessage>) -> Self {
//         Self {
//             agents: Arc::new(Mutex::new(HashMap::new())),
//             rx,
//         }
//     }

//     pub async fn register(&self, agent_id: Uuid, tx: Sender<ServerMessage>) -> Result<(), WsError> {
//         let mut agents = self.agents.lock().await;
//         agents.entry(agent_id).or_insert(tx);
//         Ok(())
//     }

//     pub async fn notice(&self, agent_id: Uuid, resp: ServerMessage) -> Result<(), WsError> {
//         let agents = self.agents.lock().await;
//         match agents.get(&agent_id) {
//             Some(tx) => {
//                 tx.send(resp).await;
//             }
//             None => return Err(WsError::Invaid),
//         }
//         Ok(())
//     }

//     pub async fn run(&mut self) -> Result<(), WsError> {
//         loop {
//             let Some(resp) = self.rx.recv().await else {
//                 return Err(WsError::Invaid);
//             };
//             self.handle(resp).await;
//         }
//     }

//     pub async fn handle(&self, resp: ServerMessage) -> Result<(), WsError> {
//         let agent_id = match &resp {
//             ServerMessage::ConnectionStatus { agent_id, .. }
//             | ServerMessage::MatchFinished { agent_id, .. }
//             | ServerMessage::Error { agent_id, .. }
//             | ServerMessage::ReadyCheck { agent_id, .. }
//             | ServerMessage::Observation { agent_id, .. }
//             | ServerMessage::MatchStart { agent_id, .. } => agent_id.clone(),
//         };
//         self.notice(agent_id, resp).await?;
//         Ok(())
//     }
// }

// pub async fn handle_ws(
//     AuthenticatedAgent { user_id, agent_id }: AuthenticatedAgent,
//     ws: WebSocketUpgrade,
//     State(state): State<WsState>,
// ) -> impl IntoResponse {
//     ws.on_upgrade(move |socket| handle_socket(user_id, agent_id, socket, state))
// }

// pub async fn handle_socket(user_id: Uuid, agent_id: Uuid, socket: WebSocket, state: WsState) {
//     let (ws_tx, ws_rx) = socket.split();
//     let client = WsClient::new(agent_id, state, ws_tx, ws_rx);
//     client.await.run().await;
// }
