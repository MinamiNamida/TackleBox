use chrono::{DateTime, Utc};
use tokio::{sync::{mpsc::{self}, oneshot}, time::timeout};
use std::{ collections::HashMap, time::Duration};
use axum::{
    extract::{ws::Utf8Bytes}
};
use axum::extract::ws::{WebSocket, Message};
use futures_util::{ stream::{SplitSink, SplitStream}, SinkExt, StreamExt};
use tracing::{debug, error, info};

use crate::api::{command::{Endpoint, ResponseError, ServerMessage, ServerPayload, SystemCommand, SystemMessage, SystemResponse, UserCommand, UserInfo, UserMessage, UserResponse}, utils::MsgGen};


pub struct ClientConnection {
    ws_tx: SplitSink<WebSocket, Message>,
    ws_rx: SplitStream<WebSocket>,
    connected_at: DateTime<Utc>,
}

pub struct ClientChannels {
    rx: mpsc::Receiver<ServerMessage>,
    replicated_tx: mpsc::Sender<ServerMessage>,
    platform_tx: mpsc::Sender<ServerMessage>,
    game_txs: HashMap<String, mpsc::Sender<ServerMessage>>,
    finish_tx: Option<oneshot::Sender<()>>,
    finish_rx: oneshot::Receiver<()>,
}

pub struct Client {
    username: Option<String>,
    connection: ClientConnection,
    channels: ClientChannels,
    msg_gen: MsgGen,
}

impl Client {
    pub fn new(
        ws_tx: SplitSink<WebSocket, Message>,
        ws_rx: SplitStream<WebSocket>,
        platform_tx: mpsc::Sender<ServerMessage>,
    ) -> Self {
        let (replicated_tx, rx) = mpsc::channel(8);

        let msg_gen = MsgGen::new(Endpoint::Client { username: None });

        let (finish_tx, finish_rx) = oneshot::channel();

        Self {
            username: None,
            connection: ClientConnection {
                ws_tx,
                ws_rx,
                connected_at: Utc::now(),
            },
            channels: ClientChannels {
                rx,
                replicated_tx,
                platform_tx,
                game_txs: HashMap::new(),
                finish_rx,
                finish_tx: Some(finish_tx)
            },
            msg_gen,
        }
    }


    pub async fn run(&mut self) -> Result<(), String> {
        debug!("client start run");
        loop {
           let ret = tokio::select! {
                Some(Ok(msg)) = self.connection.ws_rx.next() => self._handle_user_msg(msg).await,
                Some(server_msg) = self.channels.rx.recv() => self._handle_server_msg(server_msg).await,
                _ = &mut self.channels.finish_rx => break,
           };

           if let Err(e) = ret {
                let err_msg = self.msg_gen.user_error(
                    Endpoint::User, 
                    ResponseError { message: e }
                );
                let _ = self._send(err_msg).await;
           }
        }
        Ok(())
    }

    fn _parse(&self, msg: Message) -> Result<UserCommand, String> {
        let user_cmd = match msg {
            Message::Text(text) => serde_json::from_str(text.as_str()).map_err(|e| e.to_string())?,
            _ => {
                return Err("Please use text instead of bytes".to_string())
            }
        };
        Ok(user_cmd)
    }

    fn _encode(&self, msg: UserResponse) -> Result<Message, String> {
        Ok(Message::Text(Utf8Bytes::from(serde_json::to_string(&msg).unwrap())))
    }

    async fn _handle_user_msg(&mut self, msg: Message) -> Result<(), String> {
        let user_cmd = self._parse(msg)?;
        let msg: ServerMessage = match user_cmd {
            UserCommand::Register { username, password } => {
                debug!("{} try to register", username);
                self.msg_gen.sys_command(Endpoint::Platform, SystemCommand::Register { 
                        username, password, tx: self.channels.replicated_tx.clone() 
                })
            }
            UserCommand::Login { username, password } => {
                debug!("{} try to login", username);

                self.msg_gen.sys_command(Endpoint::Platform, SystemCommand::Login { 
                        username, password, tx: self.channels.replicated_tx.clone() 
                })
            }
            UserCommand::Ping => {
                let resp = self.msg_gen.user_response(
                    Endpoint::User, 
                    UserResponse::Pong);
                resp
            }
            UserCommand::SendGameData { ref room_name, .. } | UserCommand::GetGameData { ref room_name } => {
                self.msg_gen.user_command(Endpoint::Game { room_name: room_name.clone() }, user_cmd)
            }
            _ => self.msg_gen.user_command(Endpoint::Platform, user_cmd)
        };
        self._send(msg).await?;
        Ok(())
    }

    async fn _handle_server_msg(&mut self, server_msg: ServerMessage) -> Result<(), String> {
        
        let payload = server_msg.payload;
        let msg: ServerMessage = match payload {
            ServerPayload::System(sys_msg) => {
                match sys_msg {
                    SystemMessage::Response(SystemResponse::GameStart { room_name, game_tx }) => {
                        self.channels.game_txs.insert(room_name.clone(), game_tx);
                        
                        let usr_resp = self.msg_gen.user_response(Endpoint::User, UserResponse::GameStarted { room_name: room_name.clone() });
                        self._send(usr_resp).await?;
                        let game_msg = self.msg_gen.sys_command(
                            Endpoint::Game { room_name }, 
                            SystemCommand::EnterGame { 
                                username: self.username.as_ref().unwrap().clone(), 
                                tx: self.channels.replicated_tx.clone() 
                            });
                        game_msg
                    }
                    _ => return Err("error for system msg".to_string()),
                }
            }
            ServerPayload::User(usr_msg) => {
                match usr_msg {
                    UserMessage::Response(usr_resp) => {
                        match &usr_resp {
                            UserResponse::Login(user_info) | UserResponse::UserInfo(user_info) =>  {
                                let username = user_info.username.clone();
                                self.username = Some(username.clone());
                                self.msg_gen = MsgGen::new(Endpoint::Client { username: self.username.clone() });
                                self.msg_gen.user_response(Endpoint::User, usr_resp)
                            }
                            UserResponse::GameEnded { room_name } => {
                                self.channels.game_txs.remove(room_name);
                                self.msg_gen.user_response(Endpoint::User, usr_resp)
                            }
                            _ => {
                                self.msg_gen.user_response(Endpoint::User, usr_resp)
                            }
                        }
                    }
                    UserMessage::Error(err) => {
                        self.msg_gen.user_error(Endpoint::User, err)
                    }
                    _ => return Err("Error user command ".to_string())
                }
            }
        };

        self._send(msg).await?;

        Ok(())
    }

    async fn _send(&mut self, msg: ServerMessage) -> Result<(), String> {
        let to = &msg.to;
        match to {
            Endpoint::User => {
                if let ServerPayload::User(user_msg) = msg.payload {
                    let ws_msg = match user_msg {
                        UserMessage::Response(resp) => self._encode(resp)?,
                        UserMessage::Error(err) => {
                            let err_json = serde_json::to_string(&UserMessage::Error(err)).map_err(|e| e.to_string())?;
                            Message::Text(Utf8Bytes::from(err_json))
                        }
                        _ => return Err("unsupported user message".to_string())
                    };
                    self.connection.ws_tx.send(ws_msg).await.map_err(|e| e.to_string())?;
                } else {
                    return Err("send error message to ws handle".to_string())
                }
            }
            Endpoint::Game { room_name } => {
                if let Some(tx) = self.channels.game_txs.get(room_name) {
                    debug!("send to {}", room_name);
                    tx.send(msg).await.map_err(|e| e.to_string())?;
                } else {
                    return Err("no found game tx".to_string());
                }
            }
            Endpoint::Platform | Endpoint::Room { room_name: _ } => {
                self.channels.platform_tx.send(msg).await.map_err(|e| e.to_string())?;
            }
            _ => {
                return Err("Error Endpoint".to_string());
            }
        };
        Ok(())
    }

}
