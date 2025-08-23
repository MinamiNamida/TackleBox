
use std::collections::HashMap;

use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};
use serde_json::Value;
use tokio::sync::mpsc::Sender;


#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ResponseError {
    // pub code: u16,
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub enum Endpoint {
    User, 
    Client { username: Option<String> },
    Platform,
    Room { room_name: String },
    Game { room_name: String },
}


#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum UserCommand {
    Ping,
    Register { username: String, password: String },
    Login { username: String, password: String } ,
    Logout,

    UpdateUserInfo(UserInfo),
    GetUserInfo,
    SendMessage { username: String, msg: String },

    JoinRoom { room_name: String } ,
    LeaveRoom { room_name: String },
    CreateRoom { room_name: String },
    GetRooms,
    // GetUserList { room_name: String },
    SetRoomInfo(RoomInfo),
    GetRoomInfo { room_name: String },
    // KickUser { room_name: String, username: String },

    SetGameInfo { room_name: String, game_info: GameInfo } ,
    GetGameInfo { room_name: String },
    SetGameReady { room_name: String, ready: bool },
    SendGameData { room_name: String, data: Value },
    GetGameData { room_name: String },
}


#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum UserResponse {
    Inited,
    Close,
    Pong,
    Registration { username: String },
    Login(UserInfo),
    Logout,

    UserInfo(UserInfo),
    RecvMessage { username: String, msg: String },

    JoinedRoom(RoomInfo),
    LeftRoom { room_name: String },
    RoomCreated(RoomInfo),
    RoomList { rooms: Vec<RoomInfo> },
    RoomInfo(RoomInfo),
    UserKicked { room_name: String, username: String },

    GameInfo(GameInfo),
    GameReady { room_name: String, ready: bool },
    GameData { data: Value },
    GameStarted { room_name: String },
    GameEnded { room_name: String },
}

#[derive(Debug, Clone)]
pub enum SystemCommand {
    Login { username: String, password: String, tx: Sender<ServerMessage> } ,
    Register { username: String, password: String, tx: Sender<ServerMessage> },
    EnterGame { username: String, tx: Sender<ServerMessage> },
    // Unregister { username: String },
}

#[derive(Debug, Clone)]
pub enum SystemResponse {
    GameStart { room_name: String, game_tx: Sender<ServerMessage> },
    GameEnded
}

#[derive(Debug, Clone)]
pub struct ServerMessage {
    pub from: Endpoint,
    pub to: Endpoint,
    pub payload: ServerPayload,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub enum SystemMessage {
    Command(SystemCommand),
    Response(SystemResponse),
    Error(ResponseError),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UserMessage {
    Command(UserCommand),
    Response(UserResponse),
    Error(ResponseError),
}

#[derive(Debug, Clone)]
pub enum ServerPayload {
    System(SystemMessage),
    User(UserMessage),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserInfo {
    pub username: String,
    pub avatar: Option<String>,
    pub stats: UserStatus,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub enum UserStatus {
    Offline,       // 不在线
    Online,        // 已连接但未加入房间
    InRoom,        // 已加入房间，但未进入游戏
    InGame,        // 正在游戏中
    Custom(String),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Room {
    pub room_info: RoomInfo,
    pub game_info: GameInfo,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RoomInfo {
    pub room_name: String,
    pub max_user_count: usize,
    pub password_hash: Option<String>,
    pub users: HashMap<String, bool>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GameInfo {
    pub room_name: String,
    pub game_name: String,
    pub game_status: GameStatus,
    pub settings: Option<Value> ,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub enum GameStatus {
    Waiting,
    Running,
    Ended,
}