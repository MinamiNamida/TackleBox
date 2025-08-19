use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde_json::Value;
use tokio::{sync::{mpsc::{self, Receiver, Sender}, oneshot}, time::timeout};
use std::{collections::HashMap, time::Duration};
use uuid::Uuid;
use axum::{
    extract::{ws::Utf8Bytes, State, WebSocketUpgrade}, response::IntoResponse, routing::any, Error, Router
};
use axum::extract::ws::{WebSocket, Message};
use futures_util::{ stream::{SplitSink, SplitStream}, SinkExt, StreamExt};
use tracing::{debug, error, info};
use tokio::io::{self, AsyncBufReadExt};

use crate::api::command::{ClientInfo, GameInfo, ResponsePayload, ServerCommand, ServerMessage, Target, UserCommand, UserResponse};

pub struct Client {
    user_id: Uuid,
    ws_tx: SplitSink<WebSocket, Message>,
    ws_rx: SplitStream<WebSocket>,
    // 这里存放一个respond的复制体, 因为只需要一个rx便可以收到所有消息了
    rx: mpsc::Receiver<ServerMessage>,
    replicated_tx: mpsc::Sender<ServerMessage>,

    platform_tx: mpsc::Sender<ServerMessage>,
    rooms: HashMap<Uuid, mpsc::Sender<ServerMessage>>,
    connected_at: DateTime<Utc>,
}

impl Client {

    fn new( 
        ws_tx: SplitSink<WebSocket, Message>,
        ws_rx: SplitStream<WebSocket>,
        platform_tx: mpsc::Sender<ServerMessage>,
    ) -> Self {
        let (tx, rx) = mpsc::channel(8);
        let client = Self {
            ws_tx: ws_tx,
            ws_rx: ws_rx,
            user_id: Uuid::new_v4(),
            rx: rx,
            replicated_tx: tx,
            platform_tx: platform_tx,
            rooms: HashMap::new(),
            connected_at: Utc::now(),
        };
        client
    }

    fn client_info(&self) -> ClientInfo {
        ClientInfo {
            user_id: self.user_id,
            connected_rooms: self.rooms.keys().cloned().collect(),
            create_at: self.connected_at
        }
    }

    fn parser(&self, raw_msg: Message) -> Result<UserCommand, String> {
        match raw_msg {
            Message::Binary(_) => {
                Err("please use text/json instead of bytes or mutimotal data to interact".to_string())
            }
            Message::Close(_) => {
                Ok(UserCommand::Close)
            }
            Message::Ping(_) => {
                Ok(UserCommand::Ping)
            }
            Message::Pong(_) => {
                Ok(UserCommand::Ping)
            }
            Message::Text(utf_bytes) => {
                let text = utf_bytes.to_string();
                let cmd: UserCommand = serde_json::from_str(&text).map_err(|err| err.to_string())?;
                Ok(cmd)
            }
        }
    }

    fn encode(&self, response: UserResponse) -> Message {
        let msg = serde_json::to_string(&response).unwrap();
        let prefix = "[Server Response]: ".to_string();
        Message::Text(Utf8Bytes::from(prefix + msg.as_str()))
    }

    async fn send_to_client(&mut self, response: UserResponse) -> Result<(), String> {
        self.ws_tx.send(self.encode(response)).await.map_err(|e| e.to_string())?;
        Ok(())
    }

    async fn process_inputs(&mut self) {
        debug!("Client {} start processing inputs", self.user_id);
        loop {
            let r = tokio::select! {
                Some(server_msg) = self.rx.recv() => self.process_input_from_server(server_msg).await, 
                Some(result) = self.ws_rx.next() => self.process_input_from_user(result).await,
                else => break,
            };
            match r {
                Err(e) => { 
                    error!("Client {} error in process_inputs: {}", self.user_id, e);
                    break;
                }
                _ => {}
            }
        }
        debug!("Client {} stopped processing inputs", self.user_id);
    }

    async fn process_input_from_server(&mut self, server_msg: ServerMessage) -> Result<(), String> {
        debug!("Client {} received server message: {:?}", self.user_id, server_msg);

        let ServerMessage {
            from,
            to,
            payload,
            ..
        } = server_msg;

        assert!(to == Target::Client { user_id: self.user_id });

        if let Ok(cmd) = payload {
            match cmd {
                ServerCommand::UserMessage { msg } => {
                    self.send_to_client(msg).await?;
                }
                ServerCommand::ResponseCreateRoom { room_id, tx } => {
                    self.rooms.insert(room_id, tx);

                    let response = UserResponse {
                        from: Target::Room { room_id },
                        payload: Ok(ResponsePayload::CreateRoom { room_id }),
                        timestamp: Utc::now(),
                    };
                    self.send_to_client(response).await?;
                    
                    let server_msg = ServerMessage {
                        from: Target::Client { user_id: self.user_id },
                        to: Target::Room { room_id },
                        payload: Ok(ServerCommand::RegisterClient { user_id: self.user_id, tx: self.replicated_tx.clone() }),
                        timestamp: Utc::now(),
                    };
                    self.router_and_send(server_msg).await?;
                    
                }
                ServerCommand::ResponseUnregisterClient {  } => {
                    if let Target::Room { room_id } = from {
                        let response = UserResponse {
                            from: Target::Room { room_id },
                            payload: Ok(ResponsePayload::LeaveRoom),
                            timestamp: Utc::now(),
                        };
                        self.send_to_client(response).await?;
                    }
                }
                ServerCommand::ResponseRegisterClient {  } => {
                    if let Target::Room { room_id } = from {
                        let response = UserResponse {
                            from: Target::Room { room_id },
                            payload: Ok(ResponsePayload::JoinRoom),
                            timestamp: Utc::now(),
                        };
                        self.send_to_client(response).await?;
                    }
                }
                ServerCommand::ResponseGameAction { data, user_id} => {
                    assert!(self.user_id == user_id);
                    if let Target::Room { room_id } = from {
                        let response = UserResponse {
                            from: Target::Room { room_id },
                            payload: Ok(ResponsePayload::GameAction { data }),
                            timestamp: Utc::now(),
                        };
                        self.send_to_client(response).await?;
                    }
                }
                ServerCommand::ResponseListRooms { rooms_id } => {
                    let response = UserResponse {
                        from: Target::Platform,
                        payload: Ok(ResponsePayload::ListRooms { rooms_id }),
                        timestamp: Utc::now(),
                    };
                    self.send_to_client(response).await?;
                }
                ServerCommand::ResponseGetRoom { room_id, tx } => {
                    self.rooms.insert(room_id, tx);

                    let response = UserResponse {
                        from: Target::Platform,
                        payload: Ok(ResponsePayload::JoinRoom { }),
                        timestamp: Utc::now(),
                    };
                    self.send_to_client(response).await?;
                }
                ServerCommand::ResponseChooseGame {  } => {
                    let response = UserResponse {
                        from: from,
                        payload: Ok(ResponsePayload::ChooseGame { }),
                        timestamp: Utc::now(),
                    };
                    self.send_to_client(response).await?;
                }
                ServerCommand::ResponseRoomReady {  } => {
                    let response = UserResponse {
                        from: from,
                        payload: Ok(ResponsePayload::Ready { }),
                        timestamp: Utc::now(),
                    };
                    self.send_to_client(response).await?;
                }
                ServerCommand::ResponseRequestRoomInfo {  } => {

                }
                ServerCommand::GameStart {  } => {
                    let response = UserResponse {
                        from: from,
                        payload: Ok(ResponsePayload::GameStart),
                        timestamp: Utc::now(),
                    };
                    self.send_to_client(response).await?;
                }
                ServerCommand::GameFinish {  } => {
                    let response = UserResponse {
                        from: from,
                        payload: Ok(ResponsePayload::GameFinish),
                        timestamp: Utc::now(),
                    };
                    self.send_to_client(response).await?;
                }
                _ => {}
            }
        }

        Ok(())
    }

    async fn process_input_from_user(&mut self, result: Result<Message, Error>) -> Result<(), String> {
        debug!("Client {} received user message", self.user_id);

        match result {
            Ok(msg) => {
                match self.parser(msg) {
                    Ok(cmd) => {
                        match cmd {
                            UserCommand::Close {} => {
                                return Err("User close".to_string());
                            }
                            UserCommand::CreateRoom => {
                                let server_msg = ServerMessage {
                                    from: Target::Client { user_id: self.user_id },
                                    to: Target::Platform,
                                    payload: Ok(ServerCommand::CreateRoom),
                                    timestamp: Utc::now(),
                                };
                                self.router_and_send(server_msg).await?;
                            }
                            UserCommand::GameAction { room_id, data } => {
                                let server_msg = ServerMessage {
                                    from: Target::Client { user_id: self.user_id },
                                    to: Target::Room { room_id },
                                    payload: Ok(ServerCommand::GameAction { user_id: self.user_id, data }),
                                    timestamp: Utc::now(),
                                };
                                self.router_and_send(server_msg).await?;
                            }
                            UserCommand::HasJoinedRooms => {
                                let response = UserResponse {
                                    from: Target::Client { user_id: self.user_id },
                                    payload: Ok(ResponsePayload::HasJoinedRooms {rooms_id: self.rooms.keys().cloned().collect() }),
                                    timestamp: Utc::now(),
                                };
                                self.send_to_client(response).await?;
                            }
                            UserCommand::JoinRoom { room_id } => {
                                let server_msg = ServerMessage {
                                    from: Target::Client { user_id: self.user_id },
                                    to: Target::Platform,
                                    payload: Ok(ServerCommand::GetRoom { room_id }),
                                    timestamp: Utc::now(),
                                };
                                self.router_and_send(server_msg).await?;
                            }
                            UserCommand::LeaveRoom { room_id } => {
                                let server_msg = ServerMessage {
                                    from: Target::Client { user_id: self.user_id },
                                    to: Target::Room { room_id },
                                    payload: Ok(ServerCommand::ResponseUnregisterClient {  }),
                                    timestamp: Utc::now(),
                                };
                                self.router_and_send(server_msg).await?;
                            }
                            UserCommand::ListRooms => {
                                let server_msg = ServerMessage {
                                    from: Target::Client { user_id: self.user_id },
                                    to: Target::Platform,
                                    payload: Ok(ServerCommand::ListRooms { }),
                                    timestamp: Utc::now(),
                                };
                                self.router_and_send(server_msg).await?;
                            }
                            UserCommand::Ping => {
                                let response = UserResponse {
                                    from: Target::Client { user_id: self.user_id },
                                    payload: Ok(ResponsePayload::Pong),
                                    timestamp: Utc::now(),
                                };
                                self.send_to_client(response).await?;
                            }
                            UserCommand::Pong => {}
                            UserCommand::Ready { room_id, yes } => {
                                let server_msg = ServerMessage {
                                    from: Target::Client { user_id: self.user_id },
                                    to: Target::Room { room_id },
                                    payload: Ok(ServerCommand::RoomReady { yes }),
                                    timestamp: Utc::now(),
                                };
                                self.router_and_send(server_msg).await?;
                            }
                            UserCommand::RoomInfo { room_id } => {
                                let server_msg = ServerMessage {
                                    from: Target::Client { user_id: self.user_id },
                                    to: Target::Room { room_id },
                                    payload: Ok(ServerCommand::RequestRoomInfo { }),
                                    timestamp: Utc::now(),
                                };
                                self.router_and_send(server_msg).await?;
                            }
                            UserCommand::Talk { user_id, msg } => {
                                let server_msg = ServerMessage {
                                    from: Target::Client { user_id },
                                    to: Target::Platform,
                                    payload: Ok(ServerCommand::UserMessage { 
                                        msg: UserResponse { 
                                            from: Target::Client { user_id: self.user_id }, 
                                            payload: Ok(ResponsePayload::Talk { msg }),
                                            timestamp: Utc::now()
                                        } 
                                    }),
                                    timestamp: Utc::now(),
                                };
                                self.router_and_send(server_msg).await?;
                            }
                            UserCommand::GetClientInfo {} => {
                                let response = UserResponse {
                                    from: Target::Client { user_id: self.user_id },
                                    payload: Ok(ResponsePayload::ClientInfo { info: self.client_info() }),
                                    timestamp: Utc::now(),
                                };
                                self.send_to_client(response).await?;
                            }
                            UserCommand::ChooseGame { room_id, name } => {
                                let server_msg = ServerMessage {
                                    from: Target::Client { user_id: self.user_id },
                                    to: Target::Room { room_id },
                                    payload: Ok(ServerCommand::ChooseGame { name }),
                                    timestamp: Utc::now(),
                                };
                                self.router_and_send(server_msg).await?;
                            } 
                        }
                    }
                    Err(e) => {
                        let response = UserResponse {
                            from: Target::Client { user_id: self.user_id },
                            payload: Err(e),
                            timestamp: Utc::now(),
                        };   
                        self.send_to_client(response).await?;
                    }
                }
                Ok(())
            }
            Err(e) => {
                error!("Client {} error in process_input_from_user: {}", self.user_id, e);
                Err(e.to_string())
            }
        }
    }

    async fn register(&mut self) -> Result<(), String> {
        debug!("Client {} registering to platform", self.user_id);

        let register_request = ServerMessage {
            from: Target::Client { user_id: self.user_id },
            to: Target::Platform,
            timestamp: Utc::now(),
            payload: Ok(ServerCommand::RegisterClient { 
                user_id: self.user_id,
                tx: self.replicated_tx.clone(), 
            })
        };
        self.router_and_send(register_request).await?;

        match timeout(Duration::from_secs(30_u64), self.rx.recv()).await {
            Ok(server_msg) => {
                match server_msg {
                    Some(msg) => {
                        debug!("Client {} register response: {:?}", self.user_id, msg);
                        if let Ok(ServerCommand::ResponseRegisterClient {} ) = msg.payload {
                            self.send_to_client(UserResponse { 
                                from: Target::Client { user_id: self.user_id }, 
                                payload: Ok(ResponsePayload::RegisterSuccess), 
                                timestamp: Utc::now(), 
                            }).await?;
                            Ok(())
                        } else {
                            error!("Client {} failed to register: {:?}", self.user_id, msg);
                            Err("platform response error: failed register".to_string())
                        }
                    }
                    None => {
                        error!("Client {} failed to register: platform response None", self.user_id);
                        Err("platform response error: failed register".to_string())
                    }
                }
            },
            Err(_) => {
                error!("Client {} register timeout", self.user_id);
                let time_out_response = UserResponse {
                    from: Target::Client {user_id: self.user_id},
                    payload: Err("timeout to connect platform".to_string()),
                    timestamp: Utc::now(),
                };
                self.send_to_client(time_out_response).await?;
                Err("timeout to connect platform".to_string())
            }
        }
    }

    async fn unregister(&mut self) -> Result<(), String> {
        debug!("Client {} unregistering from platform", self.user_id);

        let unregister_request = ServerMessage {
            from: Target::Client { user_id: self.user_id },
            to: Target::Platform,
            timestamp: Utc::now(),
            payload: Ok(ServerCommand::UnregisterClient { 
                user_id: self.user_id,
            })
        };
        self.router_and_send(unregister_request).await?;

        match timeout(Duration::from_secs(30_u64), self.rx.recv()).await {
            Ok(server_msg) => {
                match server_msg {
                    Some(msg) => {
                        debug!("Client {} unregister response: {:?}", self.user_id, msg);
                        if let Ok(ServerCommand::ResponseUnregisterClient {} ) = msg.payload {
                            Ok(())
                        } else {
                            error!("Client {} failed to unregister: {:?}", self.user_id, msg);
                            Err("platform response error: failed unregister".to_string())
                        }
                    }
                    None => {
                        error!("Client {} failed to unregister: platform response None", self.user_id);
                        Err("platform response error: failed unregister".to_string())
                    }
                }
            },
            Err(_) => {
                error!("Client {} unregister timeout", self.user_id);
                let time_out_response = UserResponse {
                    from: Target::Client {user_id: self.user_id},
                    payload: Err("timeout to connect platform".to_string()),
                    timestamp: Utc::now(),
                };
                self.send_to_client(time_out_response).await?;
                Err("timeout to connect platform".to_string())
            }
        }
    }

    async fn router_and_send(&mut self, server_msg: ServerMessage) -> Result<(), String> {
        debug!("Client {} routing message to {:?}", self.user_id, server_msg.to);

        let to = &server_msg.to;
        match to {
            Target::Room { room_id } => {
                let tx = self.rooms.get(room_id).take().expect("no found room");
                tx.send(server_msg).await.map_err(|e| e.to_string())?;
            }
            Target::Platform => {
                self.platform_tx.send(server_msg).await.map_err(|e| e.to_string())?;
            }
            _ => {}
        }
        Ok(())
    }

    async fn run(&mut self) -> Result<(), String> {
        if let Err(e) = self.register().await {
            error!("Client register failed: {}", e);
            return Err(e);
        }

        self.process_inputs().await;

        if let Err(e) = self.unregister().await {
            error!("Client unregister failed: {}", e);
            return Err(e);
        }

        self.ws_tx.send(Message::Close(Some(axum::extract::ws::CloseFrame {
            code: axum::extract::ws::close_code::NORMAL,
            reason: "User exit".into(),
        }))).await.ok();

        Ok(())
    }

}


pub struct Platform {
    rx: mpsc::Receiver<ServerMessage>,
    replicated_tx: mpsc::Sender<ServerMessage>,
    users: HashMap<Uuid, mpsc::Sender<ServerMessage>>,
    rooms: HashMap<Uuid, mpsc::Sender<ServerMessage>>,
    stdin_rx: Receiver<String>, 
}

impl Drop for Platform {
    fn drop(&mut self) {
        error!(" Platform dropped!");
    }
}

impl Platform {
    fn new() -> Self {
        let (tx, rx) = mpsc::channel(1024);

        let (stdin_tx, stdin_rx) = mpsc::channel(8);
        Self::spawn_stdin_task(stdin_tx);

        debug!("Platform created");
        Platform { 
            rx, 
            replicated_tx: tx, 
            users: HashMap::new(), 
            rooms: HashMap::new(), 
            stdin_rx, 
            
        }
    }

    fn spawn_stdin_task(tx: Sender<String>) -> Result<(), String> {
        tokio::spawn(async move {
            let stdin = io::BufReader::new(io::stdin());
            let mut lines = stdin.lines();
            println!("Platform debug console started. Type 'list-users', 'list-rooms', 'help', 'quit'.");
            while let Ok(Some(line)) = lines.next_line().await {
                if tx.send(line).await.is_err() {
                    break;
                }
            }
        });
        Ok(())
    }

    async fn run(&mut self) {
        debug!("Platform running");
        loop {
            let r = tokio::select! {
                Some(response) = self.rx.recv() => self.process_input_from_server(response).await,
                Some(cmd) = self.stdin_rx.recv() => self.process_input_from_stdin(cmd).await,
                else => break,
            };
            if let Err(e) = r {
                error!(e);
            }
        }
        debug!("Platform stopped running");
    }

    async fn process_input_from_stdin(&mut self, cmd: String) -> Result<(), String> {
        match cmd.trim() {
            "list-users" => {
                println!("Current users:");
                for k in self.users.keys() {
                    println!("  user_id: {}", k);
                }
            }
            "list-rooms" => {
                println!("Current rooms:");
                for k in self.rooms.keys() {
                    println!("  room_id: {}", k);
                }
            }
            "help" => {
                println!("Available commands: list-users, list-rooms, help");
            }
            "" => {}
            other => {
                println!("Unknown command '{}'. Type 'help' for available commands.", other);
            }
        }
        Ok(())
    }

    async fn process_input_from_server(&mut self, server_msg: ServerMessage) -> Result<(), String> {
        debug!("Platform received server message: {:?}", server_msg);

        let ServerMessage {
            from,
            to,
            payload,
            ..
        } = server_msg;

        assert!(to == Target::Platform);

        if let Ok(cmd) = payload {
            match cmd {
                ServerCommand::RegisterClient { user_id, tx } => {
                    self.users.insert(user_id, tx);
                    let response = ServerMessage {
                        from: Target::Platform,
                        to: from,
                        payload: Ok(ServerCommand::ResponseRegisterClient {}),
                        timestamp: Utc::now()
                    };
                    self.route_and_send(response).await?;
                }
                ServerCommand::UnregisterClient { user_id } => {
                    let response = ServerMessage {
                        from: Target::Platform,
                        to: from,
                        payload: Ok(ServerCommand::ResponseUnregisterClient {}),
                        timestamp: Utc::now()
                    };
                    self.route_and_send(response).await?;
                    self.users.remove(&user_id);
                }
                ServerCommand::CreateRoom => {
                    let (room_id, tx) = Room::new(self.replicated_tx.clone());
                    self.rooms.insert(room_id, tx.clone());
                    let response = ServerMessage {
                        from: Target::Platform,
                        to: from,
                        payload: Ok(ServerCommand::ResponseCreateRoom { room_id, tx }),
                        timestamp: Utc::now()
                    };
                    self.route_and_send(response).await?;
                }
                ServerCommand::GetRoom { room_id } => {
                    let tx = self.rooms.get(&room_id).take().expect("no found room").clone();
                    let response = ServerMessage {
                        from: Target::Platform,
                        to: from,
                        payload: Ok(ServerCommand::ResponseGetRoom { room_id, tx }),
                        timestamp: Utc::now()
                    };
                    
                    self.route_and_send(response).await?;
                }
                ServerCommand::RoomTerminal => {
                    if let Target::Room { room_id } = from {
                        self.rooms.remove(&room_id);
                    }
                }
                ServerCommand::ListRooms => {
                    let response = ServerMessage {
                        from: Target::Platform,
                        to: from,
                        payload: Ok(ServerCommand::ResponseListRooms { 
                            rooms_id: self.rooms.keys().cloned().collect() 
                        }),
                        timestamp: Utc::now()
                    };

                    self.route_and_send(response).await?;
                }

                _ => {

                }
            }
        } else {
            return Err("platform recv error msg".to_string());
        }
        Ok(())
    }

    async fn route_and_send(&self, server_msg: ServerMessage) -> Result<(), String> {
        debug!("Platform routing message to {:?}", server_msg.to);

        let to = &server_msg.to;

        match to {
            Target::Client { user_id } => {
                let tx = self.users.get(user_id).ok_or("no found user")?;
                tx.send(server_msg).await.map_err(|e| e.to_string())?;
            }
            Target::User { user_id } => {
                let tx = self.users.get(user_id).ok_or("no found user")?;
                tx.send(server_msg).await.map_err(|e| e.to_string())?;
            }
            Target::Room { room_id } => {
                let tx = self.rooms.get(room_id).ok_or("no found user")?;
                tx.send(server_msg).await.map_err(|e| e.to_string())?;
            }
            _ => {}
        };

        Ok(())
    }

}

pub struct Room {
    room_id: Uuid,
    rx: Receiver<ServerMessage>,
    replicated_tx: Sender<ServerMessage>,
    platform_tx: Sender<ServerMessage>,
    users: HashMap<Uuid, Sender<ServerMessage>>,
    users_ready: HashMap<Uuid, bool>,
    game_tx : Option<Sender<ServerMessage>>,
    game_name: Option<String>,
}

impl Room {
    fn new(platform_tx: Sender<ServerMessage>) -> (Uuid, Sender<ServerMessage>) {
        let (tx, rx) = mpsc::channel(16);
        let room_id = Uuid::new_v4();
        debug!("Room {} created", room_id);
        let mut room = Room {
            room_id,
            rx,
            replicated_tx: tx.clone(),
            platform_tx,
            users: HashMap::new(),
            game_tx: None,
            game_name: None,
            users_ready: HashMap::new(),
        };
        
        tokio::spawn(async move {
            if let Err(e) = room.run().await {
                error!("Room {} run error: {}", room_id, e);
            }
        });
        (room_id, tx)
    }

    async fn run(&mut self) -> Result<(), String> {
        debug!("Room {} running", self.room_id);
        self.process_inputs().await;
        self.terminal_room().await?;
        debug!("Room {} terminated", self.room_id);
        Ok(())
    }

    async fn process_inputs(&mut self) {
        debug!("Room {} start processing inputs", self.room_id);
        loop {
            let r = tokio::select! {
                Some(server_msg) = self.rx.recv() => self.process_input_from_server(server_msg).await, 
            };
            match r {
                Err(e) => { 
                    error!("Room {} error in process_inputs: {}", self.room_id, e);
                    break;
                }
                _ => {}
            }
        }
        debug!("Room {} stopped processing inputs", self.room_id);
    }

    async fn process_input_from_server(&mut self, server_msg: ServerMessage ) -> Result<(), String> {
        debug!("Room {} received server message: {:?}", self.room_id, server_msg);

        let ServerMessage {
            from,
            payload,
            ..
        } = server_msg;

        if let Ok(cmd) = payload {
            match cmd {
                ServerCommand::RegisterClient { user_id, tx } => {
                    self.users.insert(user_id, tx);
                    self.users_ready.insert(user_id, false);
                    let response = ServerMessage {
                        from: Target::Room { room_id: self.room_id },
                        to: from,
                        payload: Ok(ServerCommand::ResponseRegisterClient {}),
                        timestamp: Utc::now()
                    };
                    self.route_and_send(response).await?;
                }
                ServerCommand::UnregisterClient { user_id } => {
                    self.users.remove(&user_id);
                    self.users_ready.remove(&user_id);
                    let response = ServerMessage {
                        from: Target::Room { room_id: self.room_id },
                        to: from,
                        payload: Ok(ServerCommand::ResponseUnregisterClient {}),
                        timestamp: Utc::now()
                    };
                    self.route_and_send(response).await?;
                }
                ServerCommand::RequestRoomInfo { } => {
                    let response = ServerMessage {
                        from: Target::Room { room_id: self.room_id },
                        to: from,
                        payload: Ok(ServerCommand::ResponseRequestRoomInfo {}),
                        timestamp: Utc::now()
                    };
                    self.route_and_send(response).await?;
                }
                ServerCommand::ChooseGame { name } => {
                    self.game_name = Some(name);

                    let response = ServerMessage {
                        from: Target::Room { room_id: self.room_id },
                        to: from,
                        payload: Ok(ServerCommand::ResponseChooseGame { }),
                        timestamp: Utc::now()
                    };
                    self.route_and_send(response).await?;
                }
                ServerCommand::RoomReady { yes } => {
                    debug!("received room ready request");
                    if let Target::Client { user_id } = from {
                       if let Some(ready) = self.users_ready.get_mut(&user_id) {
                            *ready = yes;

                            debug!("users_ready: {:?}", self.users_ready);
                            
                            let response = ServerMessage {
                                from: Target::Room { room_id: self.room_id },
                                to: from,
                                payload: Ok(ServerCommand::ResponseRoomReady {}),
                                timestamp: Utc::now()
                            };
                            self.route_and_send(response).await?;

                            if self.users_ready.values().all(|ready| *ready == true) {
                                debug!("try to run game.");
                                self.run_game().await?;

                                for user_id in self.users.keys().cloned() {
                                    let response = ServerMessage {
                                        from: Target::Room { room_id: self.room_id },
                                        to: Target::Client { user_id },
                                        payload: Ok(ServerCommand::GameStart {  }),
                                        timestamp: Utc::now(),
                                    };
                                    self.route_and_send(response).await?;
                                }
                            }



                       } else {
                           return Err("Not exist user for ready".to_string());
                       }
                    }
                }
                ServerCommand::GameAction { user_id, data } => {
                    let response = ServerMessage {
                        from: Target::Room { room_id: self.room_id },
                        to: Target::Game { room_id: self.room_id },
                        payload: Ok(ServerCommand::GameAction { user_id, data }),
                        timestamp: Utc::now()
                    };
                    self.route_and_send(response).await?;
                }
                ServerCommand::ResponseGameAction { user_id, data } => {
                    let response = ServerMessage {
                        from: Target::Room { room_id: self.room_id },
                        to: Target::Client { user_id, },
                        payload: Ok(ServerCommand::ResponseGameAction { user_id, data }),
                        timestamp: Utc::now()
                    };
                    self.route_and_send(response).await?;
                }

                ServerCommand::GameFinish {  } => {
                    for user_id in self.users.keys() {
                        let server_msg = ServerMessage {
                            from: Target::Room { room_id: self.room_id },
                            to: Target::Client { user_id: user_id.clone(), },
                            payload: Ok(ServerCommand::GameFinish { }),
                            timestamp: Utc::now()
                        };
                        self.route_and_send(server_msg).await?;
                    }
                    self.game_tx = None;
                }
                _ => {

                }
            }
        } else {
            return Err("platform recv error msg".to_string());
        }
        Ok(())
    }

    async fn route_and_send(&self, server_msg: ServerMessage) -> Result<(), String> {
        debug!("Room {} routing message to {:?}", self.room_id, server_msg.to);

        let to = &server_msg.to;

        match to {
            Target::Client { user_id } => {
                let tx = self.users.get(user_id).take().expect("no found user?");
                tx.send(server_msg).await.map_err(|e| e.to_string())?;
            }
            Target::User { user_id } => {
                let tx = self.users.get(user_id).take().expect("no found user?");
                tx.send(server_msg).await.map_err(|e| e.to_string())?;
            }
            Target::Platform => {
                let tx = &self.platform_tx;
                tx.send(server_msg).await.map_err(|e| e.to_string())?;
            }
            Target::Game { room_id: _ } => {
                if let Some(tx) = &self.game_tx {
                    tx.send(server_msg).await.map_err(|e| e.to_string())?;
                }
            }
            _ => {}
        };

        Ok(())
    }

    async fn terminal_room(&mut self) -> Result<(), String> {
        debug!("Room {} terminal_room called", self.room_id);

        let terminal_msg = ServerMessage {
            from: Target::Room { room_id: self.room_id },
            to: Target::Platform,
            payload: Ok(ServerCommand::RoomTerminal ),
            timestamp: Utc::now(),
        };
        self.route_and_send(terminal_msg).await?;
        Ok(())
    }

    async fn run_game(&mut self) -> Result<(), String> {
        debug!("Room {} run_game called, game_name: {:?}", self.room_id, self.game_name);

        if let Some(game_name) = &self.game_name {
            let game_tx = GameFactior::new(game_name, self.room_id, self.replicated_tx.clone())?;
            self.game_tx = Some(game_tx);
        }
        Ok(())
    }

}

pub struct GameFactior {}
impl GameFactior {
    fn new(game_name: &String, room_id: Uuid, room_tx: Sender<ServerMessage>) -> Result<Sender<ServerMessage>, String> {
        debug!("GameFactior creating game: {} for room {}", game_name, room_id);

        let game_name = game_name.as_str();
        let game_tx = match game_name {
            "dummy_game" => {
                DummyGame::new(room_id, room_tx)
            }
            _ => {
                return Err("no match game name".to_string());
            }
        };
        
        Ok(game_tx)
    }
}


#[async_trait]
trait Game {
    fn new(room_id: Uuid, room_tx: mpsc::Sender<ServerMessage>) -> mpsc::Sender<ServerMessage>;
    async fn run(&mut self) -> Result<(), String>;
}


enum GameStateInfo {
    FinishGame,
}

pub struct DummyGame {
    room_id: Uuid,
    rx: mpsc::Receiver<ServerMessage>,
    room_tx: mpsc::Sender<ServerMessage>,
    interact_times: usize,
    finish_game_rx: oneshot::Receiver<GameStateInfo>,
    finish_game_tx: Option<oneshot::Sender<GameStateInfo>>, 
}

#[async_trait]
impl Game for DummyGame {
    fn new(room_id: Uuid, room_tx: mpsc::Sender<ServerMessage>) -> mpsc::Sender<ServerMessage> {
        let (tx, rx) = mpsc::channel(8);
        let (stx, srx) = oneshot::channel();
        debug!("DummyGame created for room {}", room_id);
        let mut game = Self {
            room_id,
            rx,
            room_tx,
            interact_times: 0,
            finish_game_rx: srx,
            finish_game_tx: Some(stx),
        };

        tokio::spawn(async move {
            if let Err(e) = game.run().await {
                error!("DummyGame for room {} run error: {}", room_id, e);
            }
        });
        tx
    }

    async fn run(&mut self) -> Result<(), String> {
        debug!("DummyGame for room {} running", self.room_id);
        loop {
            let r = tokio::select! {
                Some(a) = self.rx.recv() => self.process_inputs(a).await,
                _ = &mut self.finish_game_rx => {
                    self.finish_game().await?;
                    return Ok(());
                }
            };
        }
    }
}

impl DummyGame {
    async fn process_inputs(&mut self, server_msg: ServerMessage) -> Result<(), String> {
        debug!("DummyGame for room {} received server message: {:?}", self.room_id, server_msg);

    
        let cmd = server_msg.payload?;
        match cmd {
            ServerCommand::GameAction { user_id, data } => {
                let response = self.process_action(user_id, data);
                self.room_tx.send(response).await.map_err(|e| e.to_string())?;
                self.interact_times += 1;
            }
            _ => {
            }
        }

        

        if self.interact_times > 3 {
            if let Some(tx) = self.finish_game_tx.take() {
                let _ = tx.send(GameStateInfo::FinishGame);
            }
        }

        Ok(())
    }

    async fn finish_game(&mut self) -> Result<(), String> {
        let game_finish = ServerMessage {
            from: Target::Game { room_id: self.room_id },
            to: Target::Room { room_id: self.room_id },
            payload: Ok(ServerCommand::GameFinish {  }),
            timestamp: Utc::now(),
        };
        self.room_tx.send(game_finish).await.map_err(|e| e.to_string())?;
        Ok(())
    }

    fn process_action(&mut self, user_id: Uuid, data: Value ) -> ServerMessage {
        let response = ServerMessage {
            from: Target::Game { room_id: self.room_id },
            to: Target::Room { room_id: self.room_id },
            payload: Ok(ServerCommand::ResponseGameAction { user_id, data }),
            timestamp: Utc::now(),
        };
        response
    }

}   

// WebSocket处理函数
pub async fn handle_socket(socket: WebSocket, platform_tx: mpsc::Sender<ServerMessage>) {
    let (ws_tx, ws_rx) = socket.split();

    debug!("New websocket connection, spawning client");
    let mut client = Client::new(ws_tx, ws_rx, platform_tx.clone());
    let _ = client.run().await;
}

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(platform_tx): State<mpsc::Sender<ServerMessage>>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, platform_tx))
}


pub struct App {
    platform_tx: Sender<ServerMessage>,
}

impl App {
    pub fn new() -> Self {
        let mut platform = Platform::new();
        let tx = platform.replicated_tx.clone();

        tokio::spawn(async move {
            platform.run().await;
        });

        Self { platform_tx: tx }
    }

    pub async fn run(&mut self) {
        let platform_tx = self.platform_tx.clone();
        let app = Router::new()
            .route("/ws", any(ws_handler))
            .with_state(platform_tx);
        let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();

        info!("ws server start running, bind on 0.0.0.0:3000");

        axum::serve(listener, app).await.unwrap();
    } 
}