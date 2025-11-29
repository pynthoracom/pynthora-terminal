//! WebSocket streaming support for real-time data ingestion (v0.3.0)
use anyhow::{Context, Result};
use futures_util::{SinkExt, StreamExt};
use serde_json::Value;
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio_tungstenite::{connect_async, tungstenite::Message};
use tracing::{debug, error, info};
use url::Url;

use crate::core::config::Config;

/// WebSocket client for real-time data streaming
pub struct WebSocketClient {
    config: Arc<Config>,
    reconnect_interval: u64,
}

impl WebSocketClient {
    /// Create a new WebSocket client
    pub fn new(config: Config) -> Self {
        Self {
            config: Arc::new(config),
            reconnect_interval: 5,
        }
    }

    /// Connect to WebSocket endpoint and stream events
    pub async fn connect_and_stream<F>(&self, mut on_event: F) -> Result<()>
    where
        F: FnMut(Value) -> Result<()> + Send + 'static,
    {
        let ws_url = self.build_ws_url()?;
        info!("Connecting to WebSocket: {}", ws_url);

        loop {
            match self.connect_once(&ws_url, &mut on_event).await {
                Ok(_) => {
                    info!("WebSocket connection closed normally");
                    break;
                }
                Err(e) => {
                    error!("WebSocket error: {}. Reconnecting in {}s...", e, self.reconnect_interval);
                    tokio::time::sleep(tokio::time::Duration::from_secs(self.reconnect_interval)).await;
                }
            }
        }

        Ok(())
    }

    async fn connect_once<F>(&self, url: &str, on_event: &mut F) -> Result<()>
    where
        F: FnMut(Value) -> Result<()>,
    {
        let url = Url::parse(url)?;
        let (ws_stream, _) = connect_async(url).await?;
        let (mut write, mut read) = ws_stream.split();

        // Send authentication message
        let auth_msg = serde_json::json!({
            "type": "auth",
            "api_key": self.config.api_key,
            "workspace": self.config.workspace,
        });
        write.send(Message::Text(auth_msg.to_string())).await?;
        info!("WebSocket authenticated");

        // Listen for messages
        while let Some(msg) = read.next().await {
            match msg {
                Ok(Message::Text(text)) => {
                    debug!("Received WebSocket message: {}", text);
                    match serde_json::from_str::<Value>(&text) {
                        Ok(value) => {
                            if let Err(e) = on_event(value) {
                                error!("Error processing event: {}", e);
                            }
                        }
                        Err(e) => {
                            error!("Failed to parse WebSocket message: {}", e);
                        }
                    }
                }
                Ok(Message::Close(_)) => {
                    info!("WebSocket connection closed");
                    break;
                }
                Ok(Message::Ping(data)) => {
                    write.send(Message::Pong(data)).await?;
                }
                Err(e) => {
                    return Err(anyhow::anyhow!("WebSocket error: {}", e));
                }
                _ => {}
            }
        }

        Ok(())
    }

    fn build_ws_url(&self) -> Result<String> {
        let base = self.config.ingest_url.replace("https://", "wss://").replace("http://", "ws://");
        Ok(format!("{}/ws/stream", base))
    }
}

/// Stream events via WebSocket with automatic reconnection
pub async fn stream_websocket(config: Config, _events: Vec<Value>) -> Result<()> {
    let client = WebSocketClient::new(config);
    
    client.connect_and_stream(|event| {
        debug!("Processing WebSocket event: {:?}", event);
        Ok(())
    }).await?;

    Ok(())
}

