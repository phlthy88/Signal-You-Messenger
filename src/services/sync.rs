//! Message synchronization service

use anyhow::Result;
use tokio::sync::mpsc;

use crate::signal::{SignalClient, SignalEvent};

/// Service for synchronizing messages with Signal servers
pub struct SyncService {
    client: std::sync::Arc<tokio::sync::Mutex<SignalClient>>,
    shutdown_tx: Option<mpsc::Sender<()>>,
}

impl SyncService {
    pub fn new(client: std::sync::Arc<tokio::sync::Mutex<SignalClient>>) -> Self {
        Self {
            client,
            shutdown_tx: None,
        }
    }

    /// Start the sync service
    pub async fn start(&mut self) -> Result<()> {
        let (shutdown_tx, mut shutdown_rx) = mpsc::channel::<()>(1);
        self.shutdown_tx = Some(shutdown_tx);

        let client = self.client.clone();

        tokio::spawn(async move {
            tracing::info!("Sync service started");

            loop {
                tokio::select! {
                    _ = shutdown_rx.recv() => {
                        tracing::info!("Sync service shutting down");
                        break;
                    }
                    event = async {
                        let mut guard = client.lock().await;
                        guard.events().recv().await
                    } => {
                        if let Some(event) = event {
                            Self::handle_event(event).await;
                        }
                    }
                }
            }
        });

        Ok(())
    }

    /// Stop the sync service
    pub async fn stop(&mut self) -> Result<()> {
        if let Some(tx) = self.shutdown_tx.take() {
            let _ = tx.send(()).await;
        }
        Ok(())
    }

    /// Handle incoming Signal events
    async fn handle_event(event: SignalEvent) {
        match event {
            SignalEvent::MessageReceived(message) => {
                tracing::info!("Received message: {:?}", message.id);
                // TODO: Store message and notify UI
            }
            SignalEvent::MessageStatusChanged { message_id, status } => {
                tracing::info!("Message {} status changed to {:?}", message_id, status);
                // TODO: Update message status in store
            }
            SignalEvent::TypingIndicator { conversation_id, sender, action } => {
                tracing::debug!("Typing indicator in {} from {:?}: {:?}", conversation_id, sender.uuid, action);
                // TODO: Notify UI of typing status
            }
            SignalEvent::ReadReceipt { conversation_id, read_at } => {
                tracing::debug!("Read receipt in {} at {}", conversation_id, read_at);
                // TODO: Update read status
            }
            SignalEvent::ContactUpdated(contact) => {
                tracing::info!("Contact updated: {:?}", contact.uuid);
                // TODO: Update contact in store
            }
            SignalEvent::GroupUpdated(group) => {
                tracing::info!("Group updated: {}", group.id);
                // TODO: Update group in store
            }
            SignalEvent::SyncReceived(sync_message) => {
                tracing::info!("Sync message received");
                // TODO: Process sync message
            }
            SignalEvent::ConnectionChanged(status) => {
                tracing::info!("Connection status: {:?}", status);
                // TODO: Notify UI of connection status
            }
            SignalEvent::DeviceLinked(identity) => {
                tracing::info!("Device linked: {:?}", identity.uuid);
                // TODO: Complete linking process
            }
            SignalEvent::Error(error) => {
                tracing::error!("Signal error: {}", error);
                // TODO: Handle error
            }
        }
    }

    /// Request a full sync from primary device
    pub async fn request_full_sync(&self) -> Result<()> {
        let client = self.client.lock().await;
        client.request_sync().await
    }
}
