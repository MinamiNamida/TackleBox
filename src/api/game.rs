use std::{collections::HashMap, time::Duration};

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::{sync::{mpsc::{self, Receiver, Sender}, oneshot}, task::JoinHandle, time::timeout};
use tracing::{debug, error};

use crate::api::{command::{Endpoint, GameInfo, Room, RoomInfo, ServerMessage, ServerPayload, SystemCommand, SystemMessage, SystemResponse, UserCommand, UserMessage, UserResponse}, utils::MsgGen};


#[async_trait]
pub trait Game : Send {
    async fn run(&mut self);
}

pub struct GameFactory {}

impl GameFactory {
    pub fn new(room: Room, platform_tx: Sender<ServerMessage>) -> Result<(Sender<ServerMessage>, JoinHandle<()>), String> {
        let (tx, rx) = mpsc::channel(8);
        let game_name = &room.game_info.game_name;

        let mut game: Box<dyn Game> = match game_name.as_str() {
            "dummy_game" => {
                Box::new(DummyGame::new(rx, tx.clone(), platform_tx, room)?)
            }
            _ => {
                return Err("error game info".to_string());
            }
        };

        let handle = tokio::spawn(async move {
            game.run().await;
        });

        Ok((tx, handle))
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct DummyGameSetting {
    max_try_times: usize,
}

#[derive(Deserialize, Serialize, Debug)]
pub enum DummyGameCommand {
    Str(String),
    Number(usize),
}

pub struct DummyGame {
    room_name: String,
    rx: Receiver<ServerMessage>,
    replicated_tx: Sender<ServerMessage>,
    finish_tx: Option<oneshot::Sender<()>>,
    finish_rx: oneshot::Receiver<()>,
    try_times: usize,
    max_try_times: usize,
    platform_tx: Sender<ServerMessage>,
    users_tx: HashMap<String, Sender<ServerMessage>>,
    users: HashMap<String, bool>,
    msg_gen: MsgGen,
}


#[async_trait]
impl Game for DummyGame  {
    async fn run(&mut self) {
        self._wait_users_init().await;
        loop {
            let ret = tokio::select! {
                Some(msg) = self.rx.recv() => {
                    let Endpoint::Client { username: Some(username) } = msg.from else {
                        continue;
                    };
                    let payload = msg.payload;
                    let ServerPayload::User(UserMessage::Command(UserCommand::SendGameData { room_name: _, data })) = payload else {
                        continue;
                    };
                    self._process_user_data(username, data).await
                }
                _ = &mut self.finish_rx => {
                    self._game_over().await;
                    break;
                }
            };
            if let Err(r) = ret {
                error!(r);
            }
        }
    }
}

impl DummyGame {
    fn new(rx: Receiver<ServerMessage>, tx: Sender<ServerMessage>, platform_tx: Sender<ServerMessage>, room: Room) -> Result<Self, String> {
        let GameInfo { game_name, settings ,..} = room.game_info;
        let RoomInfo { room_name, users , ..} = room.room_info;
        assert!(game_name == "dummy_game".to_string());

        let (finish_tx, finish_rx) = oneshot::channel();

        let max_try_times = match settings {
            Some(settings) => DummyGame::_parse(settings)?.max_try_times,
            None => 5
        };

        let msg_gen = MsgGen::new(Endpoint::Game { room_name: room_name.clone() });

        let dummy_game = Self {
            room_name, 
            rx,
            replicated_tx: tx,
            finish_tx: Some(finish_tx),
            finish_rx,
            try_times: 0,
            max_try_times,
            platform_tx,
            users_tx: HashMap::new(),
            users: users.into_iter().map(|(username, _)| (username, false)).collect(),
            msg_gen,
        };

        Ok(dummy_game)

    }

    fn _parse(setting: Value) -> Result<DummyGameSetting ,String> {
        let setting: DummyGameSetting = serde_json::from_value(setting).map_err(|e| e.to_string())?;
        Ok(setting)
    }

    async fn _wait_users_init(&mut self) -> Result<(), String> {
        if let Err(_) = timeout(Duration::from_secs(30), async {
            while !self.users.values().all(|&v| v) {
                if let Some(msg) = self.rx.recv().await {
                    let payload = msg.payload;
                    
                    match payload {
                        ServerPayload::System(SystemMessage::Command(SystemCommand::EnterGame { username, tx })) => {
                            let Some(ready) = self.users.get_mut(&username) else {
                                continue;
                            };
                            *ready = true;
                            debug!("{} entered game", username);
                            self.users_tx.insert(username, tx);
                        }
                        _ => continue,
                    } 
                } else {
                    break;
                }
            }
        }).await {
            if let Some(tx) = self.finish_tx.take() {
                let _ = tx.send(()); 
            }
            return Err("Timeout but have user not entered".to_string());
        }

        Ok(())
    }


    async fn _process_user_data(&mut self, username: String, data: Value) -> Result<(), String> {
        let cmd: DummyGameCommand = serde_json::from_value(data).map_err(|e| e.to_string())?;
        match cmd {
            DummyGameCommand::Str(s) => {
                let ps = s + " hello";
                let resp_data = serde_json::to_value(ps).unwrap();
                let msg = self.msg_gen.user_response(
                    Endpoint::Client { username: Some(username.clone()) }, 
                    UserResponse::GameData {  data: resp_data }
                );
                self._notication(&username, msg).await;
                self.try_times += 1;
            }
            DummyGameCommand::Number(n) => {
                let pn = n + 1;
                let resp_data = serde_json::to_value(pn).unwrap();
                let msg = self.msg_gen.user_response(
                    Endpoint::Client { username: Some(username.clone()) }, 
                    UserResponse::GameData {  data: resp_data }
                );
                self._notication(&username, msg).await;
                self.try_times += 1;
            }
        };

        if self.try_times >= self.max_try_times {
            self.finish_tx.take().unwrap().send(());
        }

        Ok(())
    }


    async fn _game_over(&mut self) -> Result<(), String> {
        
        let msg = self.msg_gen.sys_response(Endpoint::Platform, SystemResponse::GameOver);
        self.platform_tx.send(msg).await.map_err(|e| e.to_string())?;
        
        for username in self.users.keys() {
            let msg = self.msg_gen.user_response(
                Endpoint::Client { username: Some(username.clone()) }, 
                UserResponse::GameEnded { room_name: self.room_name.clone() 
                });
            self._notication(username, msg).await?;
        }

        Ok(())
    }

    async fn _broadcast(&self, msg: ServerMessage) -> Result<(), String> {
        for username in self.users.keys() {
            self._notication(username, msg.clone()).await?;
        }
        Ok(())
    }

    async fn _notication(&self, username: &String, msg: ServerMessage) -> Result<(), String> {
        let Some(tx) = self.users_tx.get(username) else {
            return Err("no found this user".to_string());
        };
        tx.send(msg).await.map_err(|e| e.to_string())?;
        Ok(())
    }

}