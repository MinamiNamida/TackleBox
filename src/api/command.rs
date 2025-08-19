
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};
use serde_json::Value;
use tokio::sync::mpsc::Sender;
use uuid::Uuid;


pub type Response<T> = Result<T, String>;

// ======= 用户层面 =======

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum Target {
    /// UserWs层面
    User { user_id: Uuid },
    /// Client层面
    Client { user_id: Uuid },
    /// 针对平台级操作（比如匹配、查看房间列表）
    Platform,
    /// 针对某个房间
    Room { room_id: Uuid },
    /// 针对某个游戏
    Game { room_id: Uuid },
}

#[derive(Debug, Serialize, Deserialize)]
pub enum UserCommand {
    // ===== 消息层 =====
    Talk { user_id: Uuid, msg: String },
    Ping,
    Pong,
    Close,
    GetClientInfo,

    // ===== 平台级 =====
    JoinRoom { room_id: Uuid } ,
    LeaveRoom { room_id: Uuid },
    CreateRoom,
    ListRooms,
    HasJoinedRooms,

    // ===== 房间级 =====

    ChooseGame { room_id: Uuid, name: String } ,
    RoomInfo { room_id: Uuid},
    Ready { room_id: Uuid, yes: bool },

    // ===== 游戏级 =====
    GameAction { room_id: Uuid, data: Value },
}


#[derive(Debug, Serialize, Deserialize)]
pub struct UserResponse {
    pub from: Target,
    pub payload: Response<ResponsePayload>,
    pub timestamp: DateTime<Utc>
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ResponsePayload {
    // ===== 消息层 =====
    Talk { msg: String },
    Pong,
    RegisterSuccess,
    ClientInfo { info: ClientInfo } ,

    // ===== 平台级 =====
    JoinRoom,
    LeaveRoom,
    CreateRoom { room_id: Uuid },
    ListRooms { rooms_id: Vec<Uuid> },
    HasJoinedRooms { rooms_id: Vec<Uuid> },

    // ===== 房间级 =====
    ChooseGame,
    RoomInfo,
    Ready,

    // ===== 游戏级 =====
    GameAction { data: Value },
    GameStart,
    GameFinish
}

#[derive(Debug)]
pub struct ServerMessage {
    pub from: Target,
    pub to: Target,
    pub payload: Result<ServerCommand, String>,
    pub timestamp: DateTime<Utc>
}

#[derive(Debug)]
pub enum ServerCommand {
    RegisterClient { user_id: Uuid, tx: Sender<ServerMessage> },
    ResponseRegisterClient { } ,

    UnregisterClient { user_id: Uuid },
    ResponseUnregisterClient,
    
    CreateRoom,
    ResponseCreateRoom { room_id: Uuid, tx: Sender<ServerMessage> },

    ListRooms,
    ResponseListRooms { rooms_id: Vec<Uuid> },

    GetRoom { room_id: Uuid },
    ResponseGetRoom { room_id: Uuid, tx: Sender<ServerMessage> },

    UserMessage { msg: UserResponse },

    RoomTerminal,

    ChooseGame { name: String },
    ResponseChooseGame { },

    GameAction { user_id: Uuid, data: Value },
    ResponseGameAction { user_id: Uuid,  data: Value },

    RoomReady { yes: bool },
    ResponseRoomReady {},

    RequestRoomInfo { },
    ResponseRequestRoomInfo {},

    GameStart {},
    GameFinish {},


}

#[derive(Debug, Serialize, Deserialize)]
pub struct ClientInfo {
    pub user_id: Uuid,
    pub connected_rooms: Vec<Uuid>,
    pub create_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RoomInfo {
    pub room_id: Uuid,
    pub connected_players: Vec<Uuid>,
    pub create_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GameInfo {
    pub name: String,
}