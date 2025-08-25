use std::{collections::HashMap, time::Duration};

use sea_orm::{ActiveModelTrait, ActiveValue::Set, ColumnTrait, ConnectOptions, Database, DatabaseConnection, EntityTrait, QueryFilter};
use tokio::sync::{mpsc, oneshot};

use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2
};
use tracing::{debug, error};

use crate::api::{
    command::{Endpoint, GameInfo, GameStatus, ResponseError, Room, RoomInfo, 
        ServerMessage, ServerPayload, SystemCommand, SystemMessage, 
        SystemResponse, UserCommand, UserInfo, 
        UserMessage, UserResponse, UserStatus
    }, entities::{ User, UserCol, UserAclModel}, game::GameFactory, utils::MsgGen
    
};

pub struct PlatformChannels {
    rx: mpsc::Receiver<ServerMessage>,
    replicated_tx: mpsc::Sender<ServerMessage>,
    user_txs: HashMap<String, mpsc::Sender<ServerMessage>>,
    game_txs: HashMap<String, mpsc::Sender<ServerMessage>>,
    finish_tx: Option<oneshot::Sender<()>>,
    finish_rx: oneshot::Receiver<()>,
}

pub struct Platform {
    channels: PlatformChannels,

    users_to_rooms: HashMap<String, Vec<String>>,
    rooms: HashMap<String, Room>,
    msg_gen: MsgGen,
    // users: HashMap<String, UserInfo>,

    db_connector: DatabaseConnection,
}

impl Platform {

    pub async fn new() -> Result<Self, String> {
        let (tx, rx) = mpsc::channel(1024);
        let (finish_tx, finish_rx) = oneshot::channel();
        let msg_gen = MsgGen::new(Endpoint::Platform);

        let mut opt = ConnectOptions::new("postgres://postgres:password@localhost/tacklebox");
        opt.max_connections(100)                
            .min_connections(5)               
            .connect_timeout(Duration::from_secs(8))
            .idle_timeout(Duration::from_secs(600))
            .sqlx_logging(true);              
        let db_connector = Database::connect(opt).await.map_err(|e| e.to_string())?;

        let platform = Platform { 
            channels: PlatformChannels {
                rx,
                replicated_tx: tx,
                user_txs: HashMap::new(),
                game_txs: HashMap::new(),
                finish_tx: Some(finish_tx),
                finish_rx,
            },
            users_to_rooms: HashMap::new(),
            // users: HashMap::new(),
            rooms: HashMap::new(),
            msg_gen,
            db_connector,
            // argon2: Argon2::default(),
        };
        Ok(platform)
    }



    pub fn replicated_tx(&self) -> mpsc::Sender<ServerMessage> {
        self.channels.replicated_tx.clone()
    }

    pub async fn run(&mut self) {
        
        loop {
            let r= tokio::select! {
                Some(msg) = self.channels.rx.recv() => self._handle_server_msg(msg).await,
                _ = &mut self.channels.finish_rx => {
                    break;
                }
            };
            if let Err(e) = r {
                debug!(e);
            }
        };
    }

    async fn _handle_server_msg(&mut self, server_msg: ServerMessage) -> Result<(), String> {
        assert!(server_msg.to == Endpoint::Platform, "send local error");
        let from = server_msg.from;
        let payload = server_msg.payload;
        let msg: ServerMessage = match payload {
            ServerPayload::System(sys_msg) => {
                match sys_msg {
                    SystemMessage::Command(cmd) => {
                        match cmd {
                            SystemCommand::Login { username, password, tx } => {
                                
                                // let Some(user_info) = self.users.get(&username) else {
                                //     return Err("this usr did not register".to_string())
                                // };

                                let Ok(Some(user)) = User::find().filter(UserCol::Username.eq(&username)).one(&self.db_connector).await else {
                                    return Err("no exist username or did not register".to_string());
                                };

                                let parsed_hash = PasswordHash::new(&user.password_hash).map_err(|e|e .to_string())?;
                                if  Argon2::default().verify_password(password.as_bytes(), &parsed_hash).is_ok() {
                                    self.channels.user_txs.insert(username.clone(), tx);
                                    let user_info = UserInfo {
                                        username: username.clone(),
                                        stats: UserStatus::Online,
                                    };
                                    self.msg_gen.user_response(Endpoint::Client { username: Some(username) }, UserResponse::Login(user_info))
                                } else {
                                    return Err("no exist username".to_string());   
                                }
                            }
                            SystemCommand::Register { username, password, tx } => {
                                if let Ok(Some(_)) = User::find().filter(UserCol::Username.eq(&username)).one(&self.db_connector).await {
                                    return Err("exist same username".to_string())
                                };
                                let salt = SaltString::generate(&mut OsRng);
                                let argon2 = Argon2::default();
                                let password_hash = argon2.hash_password(password.as_bytes(), &salt).unwrap().to_string();

                                let new_user = UserAclModel {
                                    username: Set(username.clone()),
                                    password_hash: Set(password_hash),
                                };

                                let inserted = new_user.insert(&self.db_connector).await.map_err(|e| e.to_string())?;

                                // self.users.insert(
                                //     username.clone(), 
                                //     UserInfo { username: username.clone(), avatar: None, stats: UserStatus::Offline }
                                // );

                                let msg = self.msg_gen.user_response(from, UserResponse::Registration { username });
                                tx.send(msg).await.map_err(|e| e.to_string())?;
                                return Ok(())
                            }
                            _ => {
                                return Err("recv error sys response".to_string());
                            }   
                        }
                    },
                    SystemMessage::Response(resp) => {
                        match resp {
                            SystemResponse::GameEnded => {
                                let Endpoint::Game { room_name } = from else {
                                    return Err("error game over input".to_string());
                                };
                                self.channels.game_txs.remove(&room_name);
                                let Some(room) = self.rooms.get_mut(&room_name) else {
                                    return Err("error game over input".to_string());
                                };
                                room.room_info.users.iter_mut().for_each(|(_, ready)| *ready = false );
                                room.game_info.game_status = GameStatus::Waiting;
                                return Ok(())
                            }
                            _ => return Err("Error msg".to_string()),
                        }
                    }
                    _ =>  {
                        return Err("recv error sys response".to_string());
                    }
                }
            }
            ServerPayload::User(user_msg) => {
                let Endpoint::Client { username: Some(ref username) } = from else { return Err("no register".to_string()) };
                match user_msg {
                    UserMessage::Command(cmd) => {
                        match cmd {
                            UserCommand::Logout => {
                                let response = self.msg_gen.user_response(
                                    Endpoint::Client { username: Some(username.clone()) }, 
                                    UserResponse::Logout
                                );
                                self._send(response).await?;
                                self.channels.user_txs.remove(username);
                                return Ok(())
                            }
                            UserCommand::SendMessage { username: to_usr, msg } => {
                                let msg = self.msg_gen.user_response(
                                    Endpoint::Client { username: Some(to_usr) }, 
                                    UserResponse::RecvMessage { username: username.clone(), msg });
                                msg
                            }
                            UserCommand::UpdateUserInfo(user_info) => {
                                let Ok(Some(user)) = User::find().filter(UserCol::Username.eq(username)).one(&self.db_connector).await else {
                                    return Err("no exist username or did not register".to_string());
                                };

                                // let mut act_user: UserAtcModle = user.into();
                                // act_user.
                                

                                // if let Some(info) = self.users.get_mut(username) {
                                //     *info = user_info;
                                let response = self.msg_gen.user_response(
                                    Endpoint::Client { username: Some(username.clone()) }, 
                                    UserResponse::UserInfo(user_info)   
                                );
                                response
                                // } else {
                                //     return Err("nofound user info".to_string())
                                // }
                            }
                            UserCommand::GetUserInfo => {
                                let Ok(Some(user)) = User::find().filter(UserCol::Username.eq(username)).one(&self.db_connector).await else {
                                    return Err("no exist username or did not register".to_string());
                                };
                                self.msg_gen.user_response(Endpoint::Client { username: Some(username.clone()) },
                                    UserResponse::UserInfo(UserInfo { username: user.username, stats: UserStatus::Online })
                                )
                            }

                            UserCommand::JoinRoom { room_name } => {
                                let Some(room) = self.rooms.get_mut(&room_name) else {
                                    return Err("no found room info".to_string())
                                };
                                room.room_info.users.insert(username.clone(), false);
                                self.users_to_rooms.get_mut(username).ok_or("no found in user in rooms")?.push(room_name.clone());
                                self.msg_gen.user_response(from, UserResponse::JoinedRoom(room.room_info.clone()))
                            }
                            UserCommand::LeaveRoom { room_name } => {
                                let Some(room) = self.rooms.get_mut(&room_name) else {
                                    return Err("no found room info".to_string())
                                };
                                room.room_info.users.remove(username);
                                self.users_to_rooms.get_mut(username).ok_or("no found in user in rooms")?.retain(|u| u != username);
                                self.msg_gen.user_response(from, UserResponse::LeftRoom { room_name })
                            }
                            UserCommand::CreateRoom { room_name } => {
                                match self.rooms.get(&room_name) {
                                    Some(_) => {
                                        self.msg_gen.user_error(from, ResponseError { message: "already exist same room name".to_string() })
                                    }
                                    None => {
                                        let mut room_info  = RoomInfo { 
                                            room_name: room_name.clone(), max_user_count: 9, 
                                            password_hash: None, users: HashMap::new()
                                        };
                                        let game_info = GameInfo {
                                            room_name: room_name.clone(),
                                            game_name: "dummy_game".to_string(),
                                            game_status: GameStatus::Waiting,
                                            settings: None,
                                        };
                                        room_info.users.insert(username.clone(), false);
                                        let msg = self.msg_gen.user_response(from, UserResponse::RoomInfo(room_info.clone()));
                                        self.rooms.insert(room_name, Room { room_info, game_info });
                                        msg
                                    }
                                }
                            }
                            UserCommand::GetRooms => {
                                self.msg_gen.user_response(from, UserResponse::RoomList { rooms: self.rooms.values().map(|r| r.room_info.clone()).collect() })
                            }
                            // UserCommand::KickUser { room_name, username } => {

                            // }
                            // UserCommand::GetUserList { room_name } => {
                            //     self.channels.user_txs.keys().cloned().collect()
                            // }

                            UserCommand::GetRoomInfo { room_name } => {
                                let Some(room) = self.rooms.get(&room_name) else {
                                    return Err("no found room".to_string())
                                };
                                self.msg_gen.user_response(from, UserResponse::RoomInfo(room.room_info.clone()))
                            }
                            UserCommand::SetRoomInfo(room_info) => {
                                let room_name = &room_info.room_name;
                                let Some(room) = self.rooms.get_mut(room_name) else {
                                    return Err("no found room".to_string())
                                };
                                room.room_info = room_info;
                                self.msg_gen.user_response(from, UserResponse::RoomInfo(room.room_info.clone()))
                            }
                            UserCommand::GetGameInfo { room_name } => {
                                let Some(room) = self.rooms.get(&room_name) else {
                                    return Err("no found game or room".to_string());
                                };
                                self.msg_gen.user_response(from, UserResponse::GameInfo(room.game_info.clone()))
                            }
                            UserCommand::SetGameInfo { room_name, game_info } => {
                                let Some(room) = self.rooms.get_mut(&room_name) else {
                                    return Err("no found game or room".to_string());
                                };
                                room.game_info = game_info;
                                self.msg_gen.user_response(from, UserResponse::GameInfo(room.game_info.clone()))
                            }
                            UserCommand::SetGameReady { room_name, ready } => {
                                let Some(room) = self.rooms.get_mut(&room_name) else {
                                    return Err("no found game or room".to_string());
                                };
                                let Some(room_ready) = room.room_info.users.get_mut(username) else {
                                    return Err("not found user in room".to_string());
                                };
                                *room_ready = ready;
                                let msg = self.msg_gen.user_response(from, UserResponse::RoomInfo(room.room_info.clone()));

                                let game_start = room.room_info.users.values().all(|r| *r == true);

                                if game_start {
                                    room.game_info.game_status = GameStatus::Running;
                                    let platform_tx = self.replicated_tx();
                                    let room = self.rooms.get(&room_name).unwrap();
                                    let (tx, handle) = GameFactory::new(room.clone(), platform_tx.clone())?;
                                    self.channels.game_txs.insert(room_name.clone(), tx.clone());
                                    for username in room.room_info.users.keys() {
                                        let game_msg = self.msg_gen.sys_response(
                                            Endpoint::Client { username: Some(username.clone()) },
                                            SystemResponse::GameStart { room_name: room_name.clone(), game_tx: tx.clone() }
                                        );
                                        self._send(game_msg).await?;
                                    }
                                }
                                msg
                            }
                            _ => {
                                return Err("Recv error user cmd".to_string());
                            }
                        }
                    }
                    _ => {
                        return Err("recv error sys response".to_string());
                    }
                }
            }
        };
        
        self._send(msg).await?;
        Ok(())
    }

    async fn _send(&self, msg: ServerMessage) -> Result<(), String> {
        let to = &msg.to;
        match to {
            Endpoint::Client { username: Some(username) } => {

                match msg.payload {
                    ServerPayload::User(UserMessage::Response(_)) | ServerPayload::System(SystemMessage::Response(_)) => {
                        let tx = self.channels.user_txs.get(username).ok_or("no found user tx")?;
                        tx.send(msg).await.map_err(|e| e.to_string())?;
                    }
                    _ => return Err("send error message to ws handle".to_string())
                }
            }
            Endpoint::Game { room_name } => {
                if let Some(tx) = self.channels.game_txs.get(room_name) {
                    tx.send(msg).await.map_err(|e| e.to_string())?;
                } else {
                    return Err("no found game tx".to_string());
                }
            }
            Endpoint::Platform | Endpoint::Room { room_name: _ } | Endpoint::User => {}
            _ => {
                return Err("no found client".to_string());
            }
        };
        Ok(())
    }
}