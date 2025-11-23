//! Background services for Signal You Messenger
//!
//! This module provides background services for real-time communication:
//! - WebSocket: Real-time message push from Signal servers
//! - Sync: Message synchronization and event handling
//! - Notifications: Desktop notification integration

mod notifications;
mod sync;
mod websocket;

pub use notifications::NotificationService;
pub use sync::SyncService;
pub use websocket::{
    IncomingMessage, ProvisioningMessage, ProvisioningSocket, SignalEndpoints,
    WebSocketCredentials, WebSocketRequest, WebSocketResponse, WebSocketService,
};
