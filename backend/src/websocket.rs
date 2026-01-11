use axum::extract::ws::{Message, WebSocket};
use futures_util::{SinkExt, StreamExt};
use tokio::sync::broadcast;

pub struct WebSocketManager {
    tx: broadcast::Sender<String>,
}

impl WebSocketManager {
    pub fn new() -> Self {
        let (tx, _) = broadcast::channel(100);
        Self { tx }
    }

    pub fn broadcast(&self, event: &str) {
        let _ = self.tx.send(event.to_string());
    }

    pub async fn handle_socket(ws: WebSocket, mut rx: broadcast::Receiver<String>) {
        let (mut sender, mut receiver) = ws.split();

        let mut send_task = tokio::spawn(async move {
            while let Ok(msg) = rx.recv().await {
                if sender.send(Message::Text(msg)).await.is_err() {
                    break;
                }
            }
        });

        let mut recv_task = tokio::spawn(async move {
            while let Some(Ok(Message::Text(_))) = receiver.next().await {
                // Handle incoming messages
            }
        });

        tokio::select! {
            _ = (&mut send_task) => recv_task.abort(),
            _ = (&mut recv_task) => send_task.abort(),
        };
    }
}

