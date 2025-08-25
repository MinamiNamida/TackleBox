use std::{collections::HashMap, time::Duration};

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::{sync::{mpsc::{self, Receiver, Sender}, oneshot}, task::JoinHandle, time::timeout};
use tracing::{debug, error};

use crate::api::{command::{Endpoint, GameInfo, ResponseError, Room, RoomInfo, ServerMessage, ServerPayload, SystemCommand, SystemMessage, SystemResponse, UserCommand, UserMessage, UserResponse}, entities::GameState, utils::MsgGen};


pub struct GameFactory {}

impl GameFactory {
    pub fn new(room: Room, platform_tx: Sender<ServerMessage>) -> Result<(Sender<ServerMessage>, JoinHandle<()>), String> {
        let (tx, rx) = mpsc::channel(8);
        let game_name = &room.game_info.game_name;

        let mut game_runner = match game_name.as_str() {
            "dummy_game" => {
                let logic = DummyLogic::new(room.game_info.settings.clone()).unwrap();
                let game_runner = GameRunner::new(rx, tx.clone(), platform_tx, room, logic);
                game_runner
            }
            _ => {
                return Err("error game info".to_string());
            }
        };

        let handle = tokio::spawn(async move {
            game_runner.run().await;
        });

        Ok((tx, handle))
    }
}

pub enum GameInput {
    UserInput { username: String, data: Value,} ,
    // Setting { setting: Value }
}

pub enum GameOutput {
    ToUser { username: String, data: Value },
    Broadcast { data: Value },
    GameState { data: Value },
    Finish,
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
    async fn on_event(&mut self, input: GameInput) -> Result<Vec<GameOutput>, String>;
}


pub struct GameRunner<L: GameLogic> {
    room_name: String,
    users: HashMap<String, bool>,
    channels: GameChannels,
    logic: L,
    msg_gen: MsgGen,
}

impl<L: GameLogic> GameRunner<L> {
    fn new(rx: Receiver<ServerMessage>, tx: Sender<ServerMessage>, platform_tx: Sender<ServerMessage>, room: Room, logic: L) -> Self {
        let RoomInfo { room_name, users , ..} = room.room_info;
        let (finish_tx, finish_rx) = oneshot::channel();

        Self { 
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
            msg_gen: MsgGen::new(Endpoint::Game { room_name }) 
        }
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
                        continue;
                    };
                    match self.logic.on_event(GameInput::UserInput { username: username.clone(), data }).await {
                        Ok(outputs) => {
                            for output in outputs {
                                self._process_game_output(output).await;
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

    async fn _process_game_output(&mut self, output: GameOutput) -> Result<(), String> {
        match output {
            GameOutput::ToUser { username, data } => {
                let payload = ServerPayload::User(UserMessage::Response(UserResponse::GameData { data }));
                self._send_user(username, payload).await?;
            }
            GameOutput::Broadcast { data } => {
                let payload = ServerPayload::User(UserMessage::Response(UserResponse::GameData { data }));
                self._broadcast(payload).await?;
            }
            GameOutput::Finish => {
                if let Some(finish_tx) = self.channels.finish_tx.take() {
                    finish_tx.send(());
                } else {
                    return Err("unexpect error for finish tx not exist".to_string())
                }
            }
            GameOutput::GameState { data: _ }  => {}
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

    async fn _update_loggers(&self) -> Result<(), String> {
        
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
    async fn on_event(&mut self, input: GameInput) -> Result<Vec<GameOutput>, String> {
        let mut outputs = vec![];
        match input {
            GameInput::UserInput { username, data } => {
                let cmd = serde_json::from_value::<DummyGameCommand>(data).map_err(|e| e.to_string())?;
                match cmd {
                    DummyGameCommand::Str(s) => {
                        let resp = serde_json::json!(s + " hello");
                        outputs.push(GameOutput::ToUser { username, data: resp });
                        self.try_times += 1;
                    }
                    DummyGameCommand::Number(n) => {
                        let resp = serde_json::json!(n + 1);
                        outputs.push(GameOutput::ToUser { username, data: resp });
                        self.try_times += 1;
                    }
                }
            }
            // GameInput::System(cmd) => {
            //     debug!("system event: {:?}", cmd);
            // }
        }

        if self.try_times >= self.max_try_times {
            outputs.push(GameOutput::Finish);
        }

        Ok(outputs)
    }
}
