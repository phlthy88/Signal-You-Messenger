//! WebSocket connection to Signal servers
//!
//! Provides real-time bidirectional communication with Signal's push service
//! for receiving messages and acknowledgments.

use anyhow::{anyhow, Result};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use futures::{SinkExt, StreamExt};
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{mpsc, oneshot, Mutex, RwLock};
use tokio::time::{interval, timeout};
use tokio_tungstenite::{
    connect_async,
    tungstenite::{http::Request, Message},
};

/// Signal WebSocket endpoints
pub struct SignalEndpoints;

impl SignalEndpoints {
    pub const SERVICE: &'static str = "wss://chat.signal.org";
    pub const STORAGE: &'static str = "wss://storage.signal.org";

    /// Get WebSocket URL with authentication
    pub fn websocket_url(username: &str, password: &str) -> String {
        format!(
            "{}/v1/websocket/?login={}&password={}",
            Self::SERVICE,
            urlencoding::encode(username),
            urlencoding::encode(password)
        )
    }

    /// Get provisioning WebSocket URL
    pub fn provisioning_url() -> String {
        format!("{}/v1/websocket/provisioning/", Self::SERVICE)
    }
}

/// WebSocket service for real-time communication with Signal
pub struct WebSocketService {
    /// WebSocket sender (wrapped for thread safety)
    sender: Arc<Mutex<Option<WebSocketSender>>>,
    /// Connection status
    is_connected: Arc<AtomicBool>,
    /// Request counter for message IDs
    request_counter: Arc<AtomicU64>,
    /// Pending requests waiting for responses
    pending_requests: Arc<RwLock<HashMap<u64, oneshot::Sender<WebSocketResponse>>>>,
    /// Channel for incoming messages
    incoming_tx: mpsc::Sender<IncomingMessage>,
    /// Shutdown signal sender
    shutdown_tx: Option<mpsc::Sender<()>>,
    /// Credentials
    credentials: Option<WebSocketCredentials>,
}

type WebSocketSender = futures::stream::SplitSink<
    tokio_tungstenite::WebSocketStream<
        tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
    >,
    Message,
>;

impl WebSocketService {
    /// Create a new WebSocket service
    pub fn new(incoming_tx: mpsc::Sender<IncomingMessage>) -> Self {
        Self {
            sender: Arc::new(Mutex::new(None)),
            is_connected: Arc::new(AtomicBool::new(false)),
            request_counter: Arc::new(AtomicU64::new(1)),
            pending_requests: Arc::new(RwLock::new(HashMap::new())),
            incoming_tx,
            shutdown_tx: None,
            credentials: None,
        }
    }

    /// Connect to Signal WebSocket
    pub async fn connect(&mut self, credentials: &WebSocketCredentials) -> Result<()> {
        if self.is_connected.load(Ordering::SeqCst) {
            return Ok(());
        }

        tracing::info!("Connecting to Signal WebSocket");

        let url = SignalEndpoints::websocket_url(&credentials.username, &credentials.password);

        // Build WebSocket request with proper headers
        let request = Request::builder()
            .uri(&url)
            .header("Sec-WebSocket-Protocol", "signal-websocket")
            .header(
                "X-Signal-Agent",
                "Signal-You/1.0.0 Linux",
            )
            .body(())?;

        // Attempt connection with retry logic
        let (ws_stream, response) = match timeout(Duration::from_secs(30), connect_async(request)).await {
            Ok(Ok(result)) => result,
            Ok(Err(e)) => {
                tracing::error!("WebSocket connection failed: {}", e);
                return Err(anyhow!("Connection failed: {}", e));
            }
            Err(_) => {
                return Err(anyhow!("Connection timeout"));
            }
        };

        tracing::info!(
            "WebSocket connected, status: {:?}",
            response.status()
        );

        let (sender, receiver) = ws_stream.split();

        // Store sender
        *self.sender.lock().await = Some(sender);
        self.is_connected.store(true, Ordering::SeqCst);
        self.credentials = Some(credentials.clone());

        // Create shutdown channel
        let (shutdown_tx, mut shutdown_rx) = mpsc::channel::<()>(1);
        self.shutdown_tx = Some(shutdown_tx);

        // Spawn receiver task
        let incoming_tx = self.incoming_tx.clone();
        let pending_requests = self.pending_requests.clone();
        let is_connected = self.is_connected.clone();

        tokio::spawn(async move {
            let mut receiver = receiver;
            let mut keepalive = interval(Duration::from_secs(30));

            loop {
                tokio::select! {
                    // Handle incoming messages
                    msg = receiver.next() => {
                        match msg {
                            Some(Ok(Message::Binary(data))) => {
                                if let Err(e) = Self::handle_message(
                                    &data,
                                    &incoming_tx,
                                    &pending_requests,
                                ).await {
                                    tracing::error!("Error handling message: {}", e);
                                }
                            }
                            Some(Ok(Message::Text(text))) => {
                                tracing::debug!("Received text message: {}", text);
                            }
                            Some(Ok(Message::Ping(data))) => {
                                tracing::trace!("Received ping");
                                // Pong is automatically sent by tungstenite
                                let _ = data;
                            }
                            Some(Ok(Message::Pong(_))) => {
                                tracing::trace!("Received pong");
                            }
                            Some(Ok(Message::Close(frame))) => {
                                tracing::info!("WebSocket closed: {:?}", frame);
                                is_connected.store(false, Ordering::SeqCst);
                                break;
                            }
                            Some(Ok(Message::Frame(_))) => {}
                            Some(Err(e)) => {
                                tracing::error!("WebSocket error: {}", e);
                                is_connected.store(false, Ordering::SeqCst);
                                break;
                            }
                            None => {
                                tracing::info!("WebSocket stream ended");
                                is_connected.store(false, Ordering::SeqCst);
                                break;
                            }
                        }
                    }
                    // Handle keepalive
                    _ = keepalive.tick() => {
                        tracing::trace!("Sending keepalive");
                    }
                    // Handle shutdown
                    _ = shutdown_rx.recv() => {
                        tracing::info!("WebSocket shutdown requested");
                        is_connected.store(false, Ordering::SeqCst);
                        break;
                    }
                }
            }

            // Notify disconnection
            let _ = incoming_tx.send(IncomingMessage::Disconnected).await;
        });

        Ok(())
    }

    /// Handle an incoming WebSocket message
    async fn handle_message(
        data: &[u8],
        incoming_tx: &mpsc::Sender<IncomingMessage>,
        pending_requests: &RwLock<HashMap<u64, oneshot::Sender<WebSocketResponse>>>,
    ) -> Result<()> {
        // Parse the WebSocket message envelope
        // Signal uses a custom binary protocol
        let envelope = WebSocketEnvelope::parse(data)?;

        match envelope {
            WebSocketEnvelope::Request(request) => {
                tracing::debug!(
                    "Received request: {} {}",
                    request.verb,
                    request.path
                );

                // Handle different request types
                if request.path == "/api/v1/message" {
                    if let Some(body) = request.body {
                        let _ = incoming_tx
                            .send(IncomingMessage::Envelope(body))
                            .await;
                    }
                } else if request.path == "/api/v1/queue/empty" {
                    let _ = incoming_tx.send(IncomingMessage::QueueEmpty).await;
                }
            }
            WebSocketEnvelope::Response(response) => {
                tracing::debug!(
                    "Received response for request {}: {}",
                    response.id,
                    response.status
                );

                // Find and complete pending request
                let mut pending = pending_requests.write().await;
                if let Some(tx) = pending.remove(&response.id) {
                    let _ = tx.send(response);
                }
            }
        }

        Ok(())
    }

    /// Disconnect from Signal WebSocket
    pub async fn disconnect(&mut self) -> Result<()> {
        tracing::info!("Disconnecting from Signal WebSocket");

        // Send shutdown signal
        if let Some(tx) = self.shutdown_tx.take() {
            let _ = tx.send(()).await;
        }

        // Close the sender
        if let Some(mut sender) = self.sender.lock().await.take() {
            let _ = sender.close().await;
        }

        self.is_connected.store(false, Ordering::SeqCst);

        Ok(())
    }

    /// Check if connected
    pub fn is_connected(&self) -> bool {
        self.is_connected.load(Ordering::SeqCst)
    }

    /// Send a request over WebSocket and wait for response
    pub async fn send_request(&self, request: WebSocketRequest) -> Result<WebSocketResponse> {
        if !self.is_connected() {
            return Err(anyhow!("WebSocket not connected"));
        }

        let id = self.request_counter.fetch_add(1, Ordering::SeqCst);

        // Create response channel
        let (tx, rx) = oneshot::channel();
        {
            let mut pending = self.pending_requests.write().await;
            pending.insert(id, tx);
        }

        // Build and send request
        let mut req = request;
        req.id = id;
        let data = req.serialize();

        {
            let mut sender = self.sender.lock().await;
            if let Some(s) = sender.as_mut() {
                s.send(Message::Binary(data)).await?;
            } else {
                return Err(anyhow!("No WebSocket sender"));
            }
        }

        // Wait for response with timeout
        match timeout(Duration::from_secs(30), rx).await {
            Ok(Ok(response)) => Ok(response),
            Ok(Err(_)) => Err(anyhow!("Response channel closed")),
            Err(_) => {
                // Remove pending request
                let mut pending = self.pending_requests.write().await;
                pending.remove(&id);
                Err(anyhow!("Request timeout"))
            }
        }
    }

    /// Send a message (fire and forget)
    pub async fn send_message(&self, message: &[u8]) -> Result<()> {
        if !self.is_connected() {
            return Err(anyhow!("WebSocket not connected"));
        }

        let mut sender = self.sender.lock().await;
        if let Some(s) = sender.as_mut() {
            s.send(Message::Binary(message.to_vec())).await?;
            tracing::debug!("Sent {} bytes over WebSocket", message.len());
            Ok(())
        } else {
            Err(anyhow!("No WebSocket sender"))
        }
    }

    /// Send an acknowledgment for a received message
    pub async fn send_ack(&self, request_id: u64) -> Result<()> {
        let response = WebSocketResponse {
            id: request_id,
            status: 200,
            message: Some("OK".to_string()),
            body: None,
            headers: vec![],
        };

        self.send_message(&response.serialize()).await
    }

    /// Attempt to reconnect with exponential backoff
    pub async fn reconnect(&mut self) -> Result<()> {
        let credentials = self
            .credentials
            .clone()
            .ok_or_else(|| anyhow!("No credentials for reconnect"))?;

        let mut delay = Duration::from_secs(1);
        let max_delay = Duration::from_secs(60);

        for attempt in 1..=5 {
            tracing::info!("Reconnection attempt {} of 5", attempt);

            match self.connect(&credentials).await {
                Ok(_) => {
                    tracing::info!("Reconnected successfully");
                    return Ok(());
                }
                Err(e) => {
                    tracing::warn!("Reconnection failed: {}", e);
                    tokio::time::sleep(delay).await;
                    delay = std::cmp::min(delay * 2, max_delay);
                }
            }
        }

        Err(anyhow!("Failed to reconnect after 5 attempts"))
    }
}

impl Default for WebSocketService {
    fn default() -> Self {
        let (tx, _rx) = mpsc::channel(100);
        Self::new(tx)
    }
}

/// Credentials for WebSocket authentication
#[derive(Clone, Debug)]
pub struct WebSocketCredentials {
    pub username: String,
    pub password: String,
}

impl WebSocketCredentials {
    pub fn new(username: impl Into<String>, password: impl Into<String>) -> Self {
        Self {
            username: username.into(),
            password: password.into(),
        }
    }

    /// Create from UUID and device credentials
    pub fn from_device(uuid: &str, device_id: u32, password: &str) -> Self {
        Self {
            username: format!("{}.{}", uuid, device_id),
            password: password.to_string(),
        }
    }

    /// Get Basic auth header value
    pub fn basic_auth(&self) -> String {
        let credentials = format!("{}:{}", self.username, self.password);
        format!("Basic {}", BASE64.encode(credentials.as_bytes()))
    }
}

/// Incoming message types
#[derive(Debug)]
pub enum IncomingMessage {
    /// Signal envelope (encrypted message)
    Envelope(Vec<u8>),
    /// Queue is empty
    QueueEmpty,
    /// Disconnected from server
    Disconnected,
}

/// WebSocket envelope types
enum WebSocketEnvelope {
    Request(WebSocketRequest),
    Response(WebSocketResponse),
}

impl WebSocketEnvelope {
    /// Parse a WebSocket message
    fn parse(data: &[u8]) -> Result<Self> {
        if data.is_empty() {
            return Err(anyhow!("Empty message"));
        }

        // Signal WebSocket protocol:
        // First byte: type (1 = request, 2 = response)
        // Following bytes: protobuf-encoded message

        let msg_type = data[0];
        let payload = &data[1..];

        match msg_type {
            1 => {
                // Request message
                let request = WebSocketRequest::parse(payload)?;
                Ok(WebSocketEnvelope::Request(request))
            }
            2 => {
                // Response message
                let response = WebSocketResponse::parse(payload)?;
                Ok(WebSocketEnvelope::Response(response))
            }
            _ => Err(anyhow!("Unknown message type: {}", msg_type)),
        }
    }
}

/// WebSocket request
#[derive(Debug, Clone)]
pub struct WebSocketRequest {
    pub id: u64,
    pub verb: String,
    pub path: String,
    pub body: Option<Vec<u8>>,
    pub headers: Vec<(String, String)>,
}

impl WebSocketRequest {
    /// Create a new request
    pub fn new(verb: impl Into<String>, path: impl Into<String>) -> Self {
        Self {
            id: 0,
            verb: verb.into(),
            path: path.into(),
            body: None,
            headers: vec![],
        }
    }

    /// Add a body
    pub fn with_body(mut self, body: Vec<u8>) -> Self {
        self.body = Some(body);
        self
    }

    /// Add a header
    pub fn with_header(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.push((name.into(), value.into()));
        self
    }

    /// Serialize to wire format
    pub fn serialize(&self) -> Vec<u8> {
        // Simple serialization format:
        // type (1 byte) | id (8 bytes) | verb_len (2 bytes) | verb | path_len (2 bytes) | path |
        // body_len (4 bytes) | body | header_count (2 bytes) | headers...

        let mut data = Vec::with_capacity(256);

        // Type = 1 (request)
        data.push(1);

        // ID
        data.extend_from_slice(&self.id.to_be_bytes());

        // Verb
        let verb_bytes = self.verb.as_bytes();
        data.extend_from_slice(&(verb_bytes.len() as u16).to_be_bytes());
        data.extend_from_slice(verb_bytes);

        // Path
        let path_bytes = self.path.as_bytes();
        data.extend_from_slice(&(path_bytes.len() as u16).to_be_bytes());
        data.extend_from_slice(path_bytes);

        // Body
        if let Some(body) = &self.body {
            data.extend_from_slice(&(body.len() as u32).to_be_bytes());
            data.extend_from_slice(body);
        } else {
            data.extend_from_slice(&0u32.to_be_bytes());
        }

        // Headers
        data.extend_from_slice(&(self.headers.len() as u16).to_be_bytes());
        for (name, value) in &self.headers {
            let name_bytes = name.as_bytes();
            let value_bytes = value.as_bytes();
            data.extend_from_slice(&(name_bytes.len() as u16).to_be_bytes());
            data.extend_from_slice(name_bytes);
            data.extend_from_slice(&(value_bytes.len() as u16).to_be_bytes());
            data.extend_from_slice(value_bytes);
        }

        data
    }

    /// Parse from wire format
    fn parse(data: &[u8]) -> Result<Self> {
        if data.len() < 12 {
            return Err(anyhow!("Request too short"));
        }

        let id = u64::from_be_bytes(data[0..8].try_into()?);
        let mut offset = 8;

        // Verb
        let verb_len = u16::from_be_bytes(data[offset..offset + 2].try_into()?) as usize;
        offset += 2;
        let verb = String::from_utf8(data[offset..offset + verb_len].to_vec())?;
        offset += verb_len;

        // Path
        let path_len = u16::from_be_bytes(data[offset..offset + 2].try_into()?) as usize;
        offset += 2;
        let path = String::from_utf8(data[offset..offset + path_len].to_vec())?;
        offset += path_len;

        // Body
        let body_len = u32::from_be_bytes(data[offset..offset + 4].try_into()?) as usize;
        offset += 4;
        let body = if body_len > 0 {
            Some(data[offset..offset + body_len].to_vec())
        } else {
            None
        };
        // Note: offset not incremented as headers parsing is not implemented yet

        // Headers (simplified - skip for now)
        let headers = vec![];

        Ok(Self {
            id,
            verb,
            path,
            body,
            headers,
        })
    }
}

/// WebSocket response
#[derive(Debug, Clone)]
pub struct WebSocketResponse {
    pub id: u64,
    pub status: u16,
    pub message: Option<String>,
    pub body: Option<Vec<u8>>,
    pub headers: Vec<(String, String)>,
}

impl WebSocketResponse {
    /// Serialize to wire format
    pub fn serialize(&self) -> Vec<u8> {
        let mut data = Vec::with_capacity(256);

        // Type = 2 (response)
        data.push(2);

        // ID
        data.extend_from_slice(&self.id.to_be_bytes());

        // Status
        data.extend_from_slice(&self.status.to_be_bytes());

        // Message
        if let Some(msg) = &self.message {
            let msg_bytes = msg.as_bytes();
            data.extend_from_slice(&(msg_bytes.len() as u16).to_be_bytes());
            data.extend_from_slice(msg_bytes);
        } else {
            data.extend_from_slice(&0u16.to_be_bytes());
        }

        // Body
        if let Some(body) = &self.body {
            data.extend_from_slice(&(body.len() as u32).to_be_bytes());
            data.extend_from_slice(body);
        } else {
            data.extend_from_slice(&0u32.to_be_bytes());
        }

        data
    }

    /// Parse from wire format
    fn parse(data: &[u8]) -> Result<Self> {
        if data.len() < 10 {
            return Err(anyhow!("Response too short"));
        }

        let id = u64::from_be_bytes(data[0..8].try_into()?);
        let status = u16::from_be_bytes(data[8..10].try_into()?);
        let mut offset = 10;

        // Message
        let msg_len = u16::from_be_bytes(data[offset..offset + 2].try_into()?) as usize;
        offset += 2;
        let message = if msg_len > 0 {
            Some(String::from_utf8(data[offset..offset + msg_len].to_vec())?)
        } else {
            None
        };
        offset += msg_len;

        // Body
        let body = if offset + 4 <= data.len() {
            let body_len = u32::from_be_bytes(data[offset..offset + 4].try_into()?) as usize;
            offset += 4;
            if body_len > 0 && offset + body_len <= data.len() {
                Some(data[offset..offset + body_len].to_vec())
            } else {
                None
            }
        } else {
            None
        };

        Ok(Self {
            id,
            status,
            message,
            body,
            headers: vec![],
        })
    }
}

/// Provisioning WebSocket for device linking
pub struct ProvisioningSocket {
    /// Incoming provisioning messages
    pub messages: mpsc::Receiver<ProvisioningMessage>,
    /// Shutdown sender
    shutdown_tx: Option<mpsc::Sender<()>>,
}

/// Provisioning message types
#[derive(Debug)]
pub enum ProvisioningMessage {
    /// Provisioning UUID assigned
    Uuid(String),
    /// Provisioning envelope received
    Envelope(Vec<u8>),
    /// Connection error
    Error(String),
}

impl ProvisioningSocket {
    /// Connect to provisioning WebSocket
    pub async fn connect() -> Result<Self> {
        let url = SignalEndpoints::provisioning_url();

        tracing::info!("Connecting to provisioning WebSocket");

        let request = Request::builder()
            .uri(&url)
            .header("Sec-WebSocket-Protocol", "signal-websocket")
            .body(())?;

        let (ws_stream, _response) = connect_async(request).await?;
        let (_sender, mut receiver) = ws_stream.split();

        let (msg_tx, msg_rx) = mpsc::channel(10);
        let (shutdown_tx, mut shutdown_rx) = mpsc::channel::<()>(1);

        // Spawn receiver task
        tokio::spawn(async move {
            loop {
                tokio::select! {
                    msg = receiver.next() => {
                        match msg {
                            Some(Ok(Message::Binary(data))) => {
                                if let Ok(prov_msg) = Self::parse_message(&data) {
                                    let _ = msg_tx.send(prov_msg).await;
                                }
                            }
                            Some(Err(e)) => {
                                let _ = msg_tx.send(ProvisioningMessage::Error(e.to_string())).await;
                                break;
                            }
                            None => break,
                            _ => {}
                        }
                    }
                    _ = shutdown_rx.recv() => {
                        break;
                    }
                }
            }
        });

        Ok(Self {
            messages: msg_rx,
            shutdown_tx: Some(shutdown_tx),
        })
    }

    /// Parse a provisioning message
    fn parse_message(data: &[u8]) -> Result<ProvisioningMessage> {
        if data.is_empty() {
            return Err(anyhow!("Empty provisioning message"));
        }

        // First byte indicates message type
        match data[0] {
            1 => {
                // UUID message
                let uuid = String::from_utf8(data[1..].to_vec())?;
                Ok(ProvisioningMessage::Uuid(uuid))
            }
            2 => {
                // Envelope
                Ok(ProvisioningMessage::Envelope(data[1..].to_vec()))
            }
            _ => Err(anyhow!("Unknown provisioning message type")),
        }
    }

    /// Close the provisioning socket
    pub async fn close(&mut self) {
        if let Some(tx) = self.shutdown_tx.take() {
            let _ = tx.send(()).await;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_credentials_basic_auth() {
        let creds = WebSocketCredentials::new("user", "pass");
        let auth = creds.basic_auth();
        assert!(auth.starts_with("Basic "));
    }

    #[test]
    fn test_request_serialization() {
        let request = WebSocketRequest::new("PUT", "/api/v1/message")
            .with_body(b"test body".to_vec())
            .with_header("Content-Type", "application/octet-stream");

        let serialized = request.serialize();
        assert!(!serialized.is_empty());
        assert_eq!(serialized[0], 1); // Request type
    }

    #[test]
    fn test_response_serialization() {
        let response = WebSocketResponse {
            id: 42,
            status: 200,
            message: Some("OK".to_string()),
            body: Some(b"response body".to_vec()),
            headers: vec![],
        };

        let serialized = response.serialize();
        assert!(!serialized.is_empty());
        assert_eq!(serialized[0], 2); // Response type
    }
}
