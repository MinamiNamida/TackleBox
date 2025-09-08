use std::{collections::HashMap, error::Error, time::Duration};

use async_trait::async_trait;
use chrono::{NaiveDateTime, Utc};
use entity::{GameAclModel, GameInputAclModel, GameParticipantAclModel, GameStateAclModel};
use sea_orm::{ActiveModelTrait, ActiveValue::Set, DatabaseConnection, EntityTrait};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::{sync::{mpsc::{self, Receiver, Sender}, oneshot}, task::JoinHandle, time::timeout};
use tracing::{debug, error};
use uuid::Uuid;

use crate::api::{
    command::{
        Endpoint, GameInfo, ResponseError, Room, RoomInfo, 
        ServerMessage, ServerPayload, SystemCommand, SystemMessage, 
        SystemResponse, UserCommand, UserMessage, UserResponse
    }, 
    utils::MsgGen
};



pub struct GameFactory {}

impl GameFactory {
    pub async fn new(room: Room, platform_tx: Sender<ServerMessage>, db: DatabaseConnection) -> Result<(Sender<ServerMessage>, JoinHandle<()>), Box<dyn Error>> {
        let (tx, rx) = mpsc::channel(8);
        let game_name = &room.game_info.game_name;

        let mut game_runner = match game_name.as_str() {
            "dummy_game" => {
                let logic = DummyLogic::new(room.game_info.settings.clone())?;
                let game_runner = GameRunner::new(rx, tx.clone(), platform_tx, room, logic, db).await?;
                game_runner
            }
            _ => {
                return Err("error game info".into());
            }
        };

        let handle = tokio::spawn(async move {
            game_runner.run().await;
        });

        Ok((tx, handle))
    }
}

#[derive(Clone, Serialize)]
pub struct GameInput {
    username: String, 
    data: Value,
}


pub struct GameOutput {
    game_state: GameState,
    game_message: Vec<GameMessage>,
}

pub enum GameMessage {
    ToUser { username: String, data: Value },
    Broadcast { data: Value },
    Finish,
}

#[derive(Serialize)]
pub struct GameState {
    state: Value
}

struct GameChannels {
    rx: Receiver<ServerMessage>,
    replicated_tx: Sender<ServerMessage>,
    finish_tx: Option<oneshot::Sender<()>>,
    finish_rx: oneshot::Receiver<()>,
    platform_tx: Sender<ServerMessage>,
    users_tx: HashMap<String, Sender<ServerMessage>>,
}

#[async_trait]
pub trait GameLogic: Send {
    fn game_type(&self) -> String;
    async fn on_event(&mut self, input: GameInput) -> Result<GameOutput, String>;
    async fn state(&self) -> GameState;
    
}


pub struct GameRunner<L: GameLogic> {
    id: Uuid, 
    room_name: String,
    users: HashMap<String, bool>,
    channels: GameChannels,
    logic: L,
    msg_gen: MsgGen,
    // logger: GameLogger,
    db: DatabaseConnection
}

impl<L: GameLogic> GameRunner<L> {


    async fn new(
        rx: Receiver<ServerMessage>, 
        tx: Sender<ServerMessage>, 
        platform_tx: Sender<ServerMessage>, 
        room: Room, 
        logic: L, 
        db: DatabaseConnection
    ) -> Result<Self, Box<dyn Error>> {

        let RoomInfo { room_name, users , ..} = room.room_info;
        let (finish_tx, finish_rx) = oneshot::channel();
        let id = Uuid::new_v4();

        // game have a unique uuid for database insert.
        let game_data = GameAclModel {
            id: Set(id),
            game_type: Set(logic.game_type()),
            created_at: Set(Utc::now()),
        };
        game_data.insert(&db).await?;

        for user in users.keys() {
            let participant = GameParticipantAclModel {
                id: Set(Uuid::new_v4()),
                game_id: Set(id),
                username: Set(user.clone()),
                joined_at: Set(Utc::now())
            };
            participant.insert(&db).await?;
        }

        let game = Self { 
            id,
            room_name: room_name.clone(), 
            users: users.into_iter().map(|(user, _)| (user, false)).collect(), 
            channels: GameChannels { 
                rx, 
                replicated_tx: 
                tx.clone(), 
                finish_tx: Some(finish_tx), 
                finish_rx, 
                platform_tx, 
                users_tx: HashMap::new() 
            }, 
            logic, 
            msg_gen: MsgGen::new(Endpoint::Game { room_name }),
            db,
        };

        Ok(game)

    }


    async fn _wait_users_init(&mut self) -> Result<(), String> {
        if let Err(_) = timeout(Duration::from_secs(30), async {
            while !self.users.values().all(|&v| v) {
                if let Some(msg) = self.channels.rx.recv().await {
                    let payload = msg.payload;
                    
                    match payload {
                        ServerPayload::System(SystemMessage::Command(SystemCommand::EnterGame { username, tx })) => {
                            let Some(ready) = self.users.get_mut(&username) else {
                                continue;
                            };
                            *ready = true;
                            debug!("{} entered game", username);
                            self.channels.users_tx.insert(username, tx);
                        }
                        _ => continue,
                    } 
                } else {
                    break;
                }
            }
        }).await {
            if let Some(tx) = self.channels.finish_tx.take() {
                let _ = tx.send(()); 
            }
            return Err("Timeout but have user not entered".to_string());
        }

        Ok(())
    }

    async fn run(&mut self) {
        self._wait_users_init().await;
        loop {
            let ret = tokio::select! {
                Some(msg) = self.channels.rx.recv() => {
                    let Endpoint::Client { username: Some(username) } = &msg.from else {
                        continue;
                    };
                    let payload = msg.payload;
                    let ServerPayload::User(UserMessage::Command(UserCommand::SendGameData { room_name: _, data })) = payload else {
                        // TODO: need tells User that give a error input to game.
                        continue;
                    };
                    let game_input = GameInput { username: username.clone(), data: data.clone() };

                    match self.logic.on_event(game_input).await {
                        Ok(GameOutput{ game_state , game_message }) => {
                            let input_id = Uuid::new_v4();
                            let input_data = GameInputAclModel {
                                id: Set(input_id),
                                game_id: Set(self.id),
                                username: Set(username.clone()),
                                data: Set(data),
                                created_at: Set(Utc::now())
                            };         
                            let GameState { state: game_state } = game_state;               
                            let state_data = GameStateAclModel {
                                id: Set(Uuid::new_v4()),
                                input_id: Set(input_id),
                                game_id: Set(self.id),
                                data: Set(game_state),
                                created_at: Set(Utc::now())
                            };
                            let _ = input_data.insert(&self.db).await;
                            let _ = state_data.insert(&self.db).await;
                            for msg in game_message {
                                self._process_game_message(msg).await;
                            }

                        }
                        Err(e) => {
                            let payload = ServerPayload::User(UserMessage::Error( ResponseError { message: e } ));
                            self._send_user(username.clone(), payload).await;
                        }
                    }
                }
                _ = &mut self.channels.finish_rx => {
                    self._game_ended().await;
                    break;
                }
            };
        }
    }

    async fn _process_game_message(&mut self, message: GameMessage) -> Result<(), String> {
        match message {
            GameMessage::ToUser { username, data } => {
                let payload = ServerPayload::User(UserMessage::Response(UserResponse::GameData { data }));
                self._send_user(username, payload).await?;
            }
            GameMessage::Broadcast { data } => {
                let payload = ServerPayload::User(UserMessage::Response(UserResponse::GameData { data }));
                self._broadcast(payload).await?;
            }
            GameMessage::Finish => {
                if let Some(finish_tx) = self.channels.finish_tx.take() {
                    finish_tx.send(());
                } else {
                    return Err("unexpect error for finish tx not exist".to_string())
                }
            }
        }
        Ok(())
    }

    async fn _game_ended(&mut self) -> Result<(), String> {
        
        let msg = self.msg_gen.sys_response(Endpoint::Platform, SystemResponse::GameEnded);
        self.channels.platform_tx.send(msg).await.map_err(|e| e.to_string())?;
        
        let payload = ServerPayload::User(
            UserMessage::Response(UserResponse::GameEnded { 
                room_name: self.room_name.clone()
            }));
        self._broadcast(payload).await?;
        
        Ok(())
    }

    async fn _broadcast(&self, payload: ServerPayload) -> Result<(), String> {
        for (username, tx) in &self.channels.users_tx {
            let to = Endpoint::Client { username: Some(username.clone()) };
            let game_msg = self.msg_gen.from_payload(to, payload.clone());
            tx.send(game_msg).await.map_err(|e| e.to_string())?;
        }
        Ok(())
    }

    async fn _send_user(&self, username: String, payload: ServerPayload) -> Result<(), String> {
        let Some(tx) = self.channels.users_tx.get(&username) else {
            return Err("unfound user in game".to_string());
        };
        let to = Endpoint::Client { username: Some(username) };
        let game_msg = self.msg_gen.from_payload(to, payload);
        tx.send(game_msg).await.map_err(|e| e.to_string())?;
        Ok(())
    }

}



#[derive(Deserialize, Serialize, Debug)]
pub enum DummyGameCommand {
    Str(String),
    Number(usize),
}

pub struct DummyLogic {
    try_times: usize,
    max_try_times: usize,
}

impl DummyLogic {
    fn new(settings: Option<Value>) -> Result<Self, String> {
        let logic = Self {
            try_times: 0,
            max_try_times: 5,
        };
        Ok(logic)
    }
}

#[async_trait]
impl GameLogic for DummyLogic {
    async fn on_event(&mut self, input: GameInput) -> Result<GameOutput, String> {
        let mut game_message = vec![];
        let GameInput { username, data } = input;

        let cmd = serde_json::from_value::<DummyGameCommand>(data).map_err(|e| e.to_string())?;
        match cmd {
            DummyGameCommand::Str(s) => {
                let resp = serde_json::json!(s + " hello");
                game_message.push(GameMessage::ToUser { username, data: resp });
                self.try_times += 1;
            }
            DummyGameCommand::Number(n) => {
                let resp = serde_json::json!(n + 1);
                game_message.push(GameMessage::ToUser { username, data: resp });
                self.try_times += 1;
            }   
        }

        if self.try_times >= self.max_try_times {
            game_message.push(GameMessage::Finish);
        }
        let game_state = self.state().await;
        
        Ok(GameOutput { game_message, game_state })
    }

    async fn state(&self) -> GameState {
        GameState { state: serde_json::json!({
            "try_times": self.try_times
        }) }
    }
    
    fn game_type(&self) -> String {
        "dummy_game".to_string()
    }
}
