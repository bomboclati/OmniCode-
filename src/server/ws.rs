use axum::{
    extract::{
        ws::{Message, WebSocket},
        State, WebSocketUpgrade,
    },
    response::IntoResponse,
};
use futures::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WsMessage {
    #[serde(rename = "type")]
    pub message_type: String,
    pub content: String,
    pub timestamp: Option<String>,
}

pub struct WsState {
    pub clients: Vec<tokio::sync::mpsc::UnboundedSender<String>>,
}

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<Mutex<WsState>>>,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_socket(socket, state))
}

async fn handle_socket(
    socket: WebSocket,
    state: Arc<Mutex<WsState>>,
) {
    let (mut sender, mut receiver) = socket.split();
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<String>();

    {
        let mut s = state.lock().await;
        s.clients.push(tx);
    }

    let send_task = tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if sender
                .send(Message::Text(msg))
                .await
                .is_err()
            {
                break;
            }
        }
    });

    let recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            if let Message::Text(text) = msg {
                tracing::info!("Received WS message: {}", text);
            }
        }
    });

    tokio::select! {
        _ = send_task => {}
        _ = recv_task => {}
    }
}

pub async fn broadcast_message(state: &Arc<Mutex<WsState>>, message: &str) {
    let s = state.lock().await;
    let msg = message.to_string();
    for client in &s.clients {
        let _ = client.send(msg.clone());
    }
}
