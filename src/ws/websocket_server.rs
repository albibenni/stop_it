use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc;
use tokio_tungstenite::tungstenite::protocol::Message;

#[derive(Debug, Deserialize, Clone)]
pub struct TabUpdateMessage {
    #[serde(rename = "type")]
    // pub msg_type: String,
    pub url: String,
    pub title: String,
    pub domain: Option<String>,
    //TODO: add category by user choice, future impl with ui? by default impl base fields
    #[serde(default)]
    pub category: Option<String>, // e.g., "productivity", "social", "entertainment"
}

#[derive(Debug, Serialize)]
pub struct WebSocketResponse {
    pub success: bool,
    pub message: Option<String>,
}

pub type ActivitySender = mpsc::UnboundedSender<TabUpdateMessage>;
pub type ActivityReceiver = mpsc::UnboundedReceiver<TabUpdateMessage>;

pub fn create_activity_channel() -> (ActivitySender, ActivityReceiver) {
    mpsc::unbounded_channel()
}

pub async fn start_websocket_server(
    addr: SocketAddr,
    activity_tx: ActivitySender,
) -> Result<(), Box<dyn std::error::Error>> {
    let listener = TcpListener::bind(&addr).await?;
    println!("WebSocket server listening on: {}", addr);

    while let Ok((stream, peer_addr)) = listener.accept().await {
        println!("New WebSocket connection from: {}", peer_addr);
        let tx = activity_tx.clone();
        tokio::spawn(handle_connection(stream, peer_addr, tx));
    }

    Ok(())
}

async fn handle_connection(stream: TcpStream, peer_addr: SocketAddr, activity_tx: ActivitySender) {
    let ws_stream = match tokio_tungstenite::accept_async(stream).await {
        Ok(ws) => ws,
        Err(e) => {
            eprintln!("WebSocket handshake failed with {}: {}", peer_addr, e);
            return;
        }
    };

    println!("WebSocket handshake completed with {}", peer_addr);

    let (mut ws_sender, mut ws_receiver) = ws_stream.split();

    while let Some(msg) = ws_receiver.next().await {
        match msg {
            Ok(Message::Text(text)) => {
                match serde_json::from_str::<TabUpdateMessage>(&text) {
                    Ok(tab_message) => {
                        println!(
                            "[WebSocket] Received: url={}, title={}, domain={:?}, category={:?}",
                            tab_message.url,
                            tab_message.title,
                            tab_message.domain,
                            tab_message.category
                        );

                        // Send to activity tracker
                        if let Err(e) = activity_tx.send(tab_message) {
                            eprintln!("Failed to send activity message: {}", e);
                        }

                        // Send success response
                        let response = WebSocketResponse {
                            success: true,
                            message: Some("Message received".to_string()),
                        };

                        if let Ok(response_json) = serde_json::to_string(&response) {
                            if let Err(e) = ws_sender.send(Message::Text(response_json)).await {
                                eprintln!("Failed to send WebSocket response: {}", e);
                                break;
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("Failed to parse message: {}", e);
                        let response = WebSocketResponse {
                            success: false,
                            message: Some(format!("Parse error: {}", e)),
                        };
                        if let Ok(response_json) = serde_json::to_string(&response) {
                            let _ = ws_sender.send(Message::Text(response_json)).await;
                        }
                    }
                }
            }
            Ok(Message::Close(_)) => {
                println!("WebSocket connection closed by {}", peer_addr);
                break;
            }
            Ok(Message::Ping(data)) => {
                if let Err(e) = ws_sender.send(Message::Pong(data)).await {
                    eprintln!("Failed to send pong: {}", e);
                    break;
                }
            }
            Ok(_) => {}
            Err(e) => {
                eprintln!("WebSocket error from {}: {}", peer_addr, e);
                break;
            }
        }
    }

    println!("WebSocket connection with {} terminated", peer_addr);
}
