use axum::{
    extract::{ws::{WebSocket, WebSocketUpgrade}, State}, response::{IntoResponse}, routing::any, Router
};
use tokio::sync::mpsc::Sender;
use futures_util::{StreamExt};
use tracing::{debug, info};
use crate::api::{client::Client, command::ServerMessage, platform::Platform};



pub async fn handle_socket(socket: WebSocket, platform_tx: Sender<ServerMessage>) {
    let (ws_tx, ws_rx) = socket.split();

    debug!("New websocket connection, spawning client");
    let mut client = Client::new(ws_tx, ws_rx, platform_tx.clone());
    let _ = client.run().await;
    debug!("web socket connection closed");
}

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(platform_tx): State<Sender<ServerMessage>>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, platform_tx))
}

pub struct App {
    platform_tx: Sender<ServerMessage>
}

impl App {

    pub async fn new() -> Result<Self, String> {
        let mut platform = Platform::new().await?;
        let tx = platform.replicated_tx().clone();

        tokio::spawn(async move {
            platform.run().await;
        });

        Ok(Self { platform_tx: tx })
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