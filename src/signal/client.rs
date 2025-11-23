//! Signal client wrapper
//!
//! Provides high-level API for Signal operations including linking,
//! sending/receiving messages, and managing contacts.

use anyhow::{anyhow, Result};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use std::path::Path;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use uuid::Uuid;

use super::crypto::{DhKeyPair, IdentityKeyPair, SignalCipher};
use super::protocol::{ProtocolAddress, SignalProtocol};
use super::store::SignalStore;
use super::types::*;
use crate::services::{
    IncomingMessage, ProvisioningSocket, WebSocketCredentials, WebSocketService,
};

/// High-level Signal client
pub struct SignalClient {
    /// Signal protocol for cryptographic operations
    protocol: Arc<RwLock<SignalProtocol>>,
    /// Encrypted data store
    store: Arc<SignalStore>,
    /// WebSocket service for real-time messages
    websocket: Arc<RwLock<WebSocketService>>,
    /// Current identity
    identity: Option<SignalIdentity>,
    /// Device credentials (password)
    device_password: Option<String>,
    /// Whether device is linked
    is_linked: bool,
    /// Event sender for UI updates
    event_tx: mpsc::Sender<SignalEvent>,
    /// Incoming message receiver
    incoming_rx: Arc<RwLock<mpsc::Receiver<IncomingMessage>>>,
}

/// Events emitted by the Signal client
#[derive(Debug, Clone)]
pub enum SignalEvent {
    /// New message received
    MessageReceived(Message),
    /// Message delivery status updated
    MessageStatusChanged {
        message_id: String,
        status: MessageStatus,
    },
    /// Typing indicator received
    TypingIndicator {
        conversation_id: String,
        sender: SignalIdentity,
        action: TypingAction,
    },
    /// Read receipt received
    ReadReceipt {
        conversation_id: String,
        read_at: i64,
    },
    /// Contact updated
    ContactUpdated(SignalIdentity),
    /// Group updated
    GroupUpdated(Group),
    /// Sync message received
    SyncReceived(SyncMessage),
    /// Connection status changed
    ConnectionChanged(ConnectionStatus),
    /// Device linked successfully
    DeviceLinked(SignalIdentity),
    /// Error occurred
    Error(String),
}

/// Connection status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionStatus {
    Connecting,
    Connected,
    Disconnected,
    Reconnecting,
}

impl SignalClient {
    /// Create a new Signal client
    pub async fn new(data_dir: &Path) -> Result<Self> {
        let store = SignalStore::new(data_dir).await?;
        let (event_tx, _event_rx) = mpsc::channel(100);
        let (incoming_tx, incoming_rx) = mpsc::channel(100);

        let websocket = WebSocketService::new(incoming_tx);

        // Try to load existing identity
        let (protocol, identity, is_linked) =
            if let Some((pub_key, priv_key, reg_id)) = store.get_local_identity().await? {
                let mut key_bytes = [0u8; 32];
                key_bytes.copy_from_slice(&priv_key[..32]);
                let protocol = SignalProtocol::from_identity(&key_bytes, reg_id)?;

                // Load identity from store
                let identity = store.get_identity().await?;
                (protocol, identity, identity.is_some())
            } else {
                (SignalProtocol::new()?, None, false)
            };

        Ok(Self {
            protocol: Arc::new(RwLock::new(protocol)),
            store: Arc::new(store),
            websocket: Arc::new(RwLock::new(websocket)),
            identity,
            device_password: None,
            is_linked,
            event_tx,
            incoming_rx: Arc::new(RwLock::new(incoming_rx)),
        })
    }

    /// Check if this device is linked to a Signal account
    pub fn is_linked(&self) -> bool {
        self.is_linked
    }

    /// Get the current identity
    pub fn identity(&self) -> Option<&SignalIdentity> {
        self.identity.as_ref()
    }

    /// Get event sender for subscribing to events
    pub fn event_sender(&self) -> mpsc::Sender<SignalEvent> {
        self.event_tx.clone()
    }

    /// Generate a device linking URI for QR code
    ///
    /// The URI format is: sgnl://linkdevice?uuid=<prov_uuid>&pub_key=<base64_key>
    pub async fn generate_linking_uri(&self) -> Result<(String, LinkingSession)> {
        tracing::info!("Generating device linking URI");

        // Generate ephemeral key pair for provisioning
        let ephemeral_key = DhKeyPair::generate();
        let public_key_bytes = ephemeral_key.public_key().as_bytes();
        let public_key_base64 = BASE64.encode(public_key_bytes);

        // Generate provisioning UUID
        let prov_uuid = Uuid::new_v4().to_string();

        // Build URI
        let uri = format!(
            "sgnl://linkdevice?uuid={}&pub_key={}",
            urlencoding::encode(&prov_uuid),
            urlencoding::encode(&public_key_base64)
        );

        let session = LinkingSession {
            provisioning_uuid: prov_uuid,
            ephemeral_key,
        };

        Ok((uri, session))
    }

    /// Wait for device linking to complete
    pub async fn wait_for_linking(
        &mut self,
        session: LinkingSession,
    ) -> Result<SignalIdentity> {
        tracing::info!("Waiting for device linking");

        // Connect to provisioning socket
        let mut prov_socket = ProvisioningSocket::connect().await?;

        // Wait for provisioning message
        while let Some(msg) = prov_socket.messages.recv().await {
            match msg {
                crate::services::ProvisioningMessage::Uuid(uuid) => {
                    tracing::info!("Received provisioning UUID: {}", uuid);
                }
                crate::services::ProvisioningMessage::Envelope(envelope) => {
                    tracing::info!("Received provisioning envelope");

                    // Decrypt the provisioning message
                    let identity = self
                        .process_provisioning_message(&session, &envelope)
                        .await?;

                    // Close the provisioning socket
                    prov_socket.close().await;

                    // Store the identity
                    self.identity = Some(identity.clone());
                    self.is_linked = true;

                    // Emit event
                    let _ = self.event_tx.send(SignalEvent::DeviceLinked(identity.clone())).await;

                    return Ok(identity);
                }
                crate::services::ProvisioningMessage::Error(e) => {
                    return Err(anyhow!("Provisioning error: {}", e));
                }
            }
        }

        Err(anyhow!("Provisioning socket closed without completing"))
    }

    /// Process a provisioning message to complete device linking
    async fn process_provisioning_message(
        &mut self,
        session: &LinkingSession,
        envelope: &[u8],
    ) -> Result<SignalIdentity> {
        // The provisioning envelope is encrypted with the ephemeral key
        // Format: [32-byte public key][encrypted data]
        if envelope.len() < 48 {
            return Err(anyhow!("Provisioning envelope too short"));
        }

        let primary_public_key: [u8; 32] = envelope[0..32].try_into()?;
        let ciphertext = &envelope[32..];

        // Derive shared secret
        let shared_secret = session.ephemeral_key.dh_agreement(
            &x25519_dalek::PublicKey::from(primary_public_key),
        );

        // Decrypt the provisioning data
        // First 12 bytes of ciphertext are the nonce
        if ciphertext.len() < 12 {
            return Err(anyhow!("Ciphertext too short for nonce"));
        }
        let nonce: [u8; 12] = ciphertext[0..12].try_into()?;
        let encrypted_data = &ciphertext[12..];

        let plaintext = SignalCipher::decrypt(&shared_secret, &nonce, encrypted_data)?;

        // Parse the provisioning data (simplified JSON format for now)
        let prov_data: ProvisioningData = serde_json::from_slice(&plaintext)
            .map_err(|e| anyhow!("Failed to parse provisioning data: {}", e))?;

        // Create identity from provisioning data
        let identity = SignalIdentity {
            uuid: prov_data.uuid,
            phone_number: Some(prov_data.phone_number),
            device_id: 2, // Linked devices start at 2
            registration_id: {
                let protocol = self.protocol.read().await;
                protocol.registration_id()
            },
        };

        // Store the identity and keys
        {
            let protocol = self.protocol.read().await;
            self.store
                .store_local_identity(
                    &protocol.identity_public_key().as_bytes(),
                    &protocol.identity_private_key(),
                    protocol.registration_id(),
                )
                .await?;
        }

        // Generate and store pre-keys
        {
            let mut protocol = self.protocol.write().await;
            let pre_keys = protocol.generate_pre_keys(100)?;

            // Convert to storage format
            let storage_keys: Vec<(u32, Vec<u8>, Vec<u8>)> = pre_keys
                .iter()
                .map(|(id, pub_key)| (*id, pub_key.clone(), vec![]))
                .collect();

            self.store.store_pre_keys(&storage_keys).await?;

            // Generate signed pre-key
            let (spk_pub, spk_sig) = protocol.generate_signed_pre_key(1)?;
            self.store
                .store_signed_pre_key(
                    1,
                    &spk_pub,
                    &[], // Private key handled by protocol
                    &spk_sig,
                    chrono::Utc::now().timestamp(),
                )
                .await?;
        }

        // Store device password for WebSocket auth
        self.device_password = Some(prov_data.provisioning_code.clone());

        tracing::info!("Device linking complete: {:?}", identity.uuid);

        Ok(identity)
    }

    /// Connect to Signal servers
    pub async fn connect(&mut self) -> Result<()> {
        if !self.is_linked {
            return Err(anyhow!("Device not linked"));
        }

        let identity = self
            .identity
            .as_ref()
            .ok_or_else(|| anyhow!("No identity"))?;

        let password = self
            .device_password
            .as_ref()
            .ok_or_else(|| anyhow!("No device password"))?;

        tracing::info!("Connecting to Signal servers");

        let _ = self
            .event_tx
            .send(SignalEvent::ConnectionChanged(ConnectionStatus::Connecting))
            .await;

        // Create credentials
        let credentials = WebSocketCredentials::from_device(
            &identity.uuid.to_string(),
            identity.device_id,
            password,
        );

        // Connect WebSocket
        {
            let mut ws = self.websocket.write().await;
            ws.connect(&credentials).await?;
        }

        let _ = self
            .event_tx
            .send(SignalEvent::ConnectionChanged(ConnectionStatus::Connected))
            .await;

        // Start message receive loop
        self.start_message_loop();

        Ok(())
    }

    /// Start the background message processing loop
    fn start_message_loop(&self) {
        let incoming_rx = self.incoming_rx.clone();
        let event_tx = self.event_tx.clone();
        let protocol = self.protocol.clone();
        let store = self.store.clone();

        tokio::spawn(async move {
            let mut rx = incoming_rx.write().await;

            while let Some(msg) = rx.recv().await {
                match msg {
                    IncomingMessage::Envelope(envelope) => {
                        if let Err(e) =
                            Self::process_envelope(&envelope, &protocol, &store, &event_tx).await
                        {
                            tracing::error!("Failed to process envelope: {}", e);
                            let _ = event_tx.send(SignalEvent::Error(e.to_string())).await;
                        }
                    }
                    IncomingMessage::QueueEmpty => {
                        tracing::debug!("Message queue empty");
                    }
                    IncomingMessage::Disconnected => {
                        tracing::warn!("WebSocket disconnected");
                        let _ = event_tx
                            .send(SignalEvent::ConnectionChanged(ConnectionStatus::Disconnected))
                            .await;
                    }
                }
            }
        });
    }

    /// Process an incoming message envelope
    async fn process_envelope(
        envelope: &[u8],
        protocol: &Arc<RwLock<SignalProtocol>>,
        store: &Arc<SignalStore>,
        event_tx: &mpsc::Sender<SignalEvent>,
    ) -> Result<()> {
        // Parse envelope header
        // Format: [source_uuid:36][device_id:4][timestamp:8][type:1][content...]
        if envelope.len() < 49 {
            return Err(anyhow!("Envelope too short"));
        }

        let source_uuid_str = String::from_utf8(envelope[0..36].to_vec())?;
        let source_uuid: Uuid = source_uuid_str.parse()?;
        let device_id = u32::from_be_bytes(envelope[36..40].try_into()?);
        let timestamp = i64::from_be_bytes(envelope[40..48].try_into()?);
        let msg_type = envelope[48];
        let content = &envelope[49..];

        let sender_address = ProtocolAddress::new(source_uuid.to_string(), device_id);

        // Decrypt the content based on message type
        let plaintext = match msg_type {
            1 => {
                // Pre-key message (initial message)
                let mut proto = protocol.write().await;
                proto.decrypt_initial(&sender_address, content).await?
            }
            2 => {
                // Regular message
                let proto = protocol.read().await;
                proto.decrypt(&sender_address, content).await?
            }
            _ => {
                return Err(anyhow!("Unknown message type: {}", msg_type));
            }
        };

        // Parse the decrypted content as a message
        let content: MessageContent = serde_json::from_slice(&plaintext)
            .unwrap_or(MessageContent::Text {
                body: String::from_utf8_lossy(&plaintext).to_string(),
            });

        // Create message object
        let message = Message {
            id: Uuid::new_v4().to_string(),
            conversation_id: source_uuid.to_string(),
            sender: SignalIdentity {
                uuid: source_uuid,
                phone_number: None,
                device_id,
                registration_id: 0,
            },
            timestamp,
            received_timestamp: Some(chrono::Utc::now().timestamp_millis()),
            content,
            status: MessageStatus::Delivered,
            quote: None,
            reactions: Vec::new(),
            expires_at: None,
        };

        // Store the message
        store.store_message(&message).await?;

        // Emit event
        let _ = event_tx.send(SignalEvent::MessageReceived(message)).await;

        Ok(())
    }

    /// Disconnect from Signal servers
    pub async fn disconnect(&mut self) -> Result<()> {
        tracing::info!("Disconnecting from Signal servers");

        {
            let mut ws = self.websocket.write().await;
            ws.disconnect().await?;
        }

        let _ = self
            .event_tx
            .send(SignalEvent::ConnectionChanged(ConnectionStatus::Disconnected))
            .await;

        Ok(())
    }

    /// Send a text message
    pub async fn send_message(
        &self,
        recipient: &SignalIdentity,
        content: &str,
    ) -> Result<Message> {
        tracing::info!("Sending message to {:?}", recipient.uuid);

        let recipient_address =
            ProtocolAddress::new(recipient.uuid.to_string(), recipient.device_id);

        // Create message content
        let msg_content = MessageContent::Text {
            body: content.to_string(),
        };
        let content_bytes = serde_json::to_vec(&msg_content)?;

        // Encrypt the message
        let ciphertext = {
            let protocol = self.protocol.read().await;

            if protocol.has_session(&recipient_address).await {
                protocol.encrypt(&recipient_address, &content_bytes).await?
            } else {
                // Need to fetch pre-key bundle and establish session
                // For now, return error - real implementation would fetch from server
                return Err(anyhow!(
                    "No session with recipient. Pre-key bundle fetch not implemented."
                ));
            }
        };

        // Build envelope and send via WebSocket
        let timestamp = chrono::Utc::now().timestamp_millis();
        let mut envelope = Vec::with_capacity(49 + ciphertext.len());

        if let Some(identity) = &self.identity {
            envelope.extend_from_slice(identity.uuid.to_string().as_bytes());
            envelope.extend_from_slice(&identity.device_id.to_be_bytes());
        } else {
            return Err(anyhow!("No local identity"));
        }

        envelope.extend_from_slice(&timestamp.to_be_bytes());
        envelope.push(2); // Regular message type
        envelope.extend_from_slice(&ciphertext);

        {
            let ws = self.websocket.read().await;
            ws.send_message(&envelope).await?;
        }

        // Create message object
        let message = Message {
            id: Uuid::new_v4().to_string(),
            conversation_id: recipient.uuid.to_string(),
            sender: self.identity.clone().unwrap(),
            timestamp,
            received_timestamp: None,
            content: msg_content,
            status: MessageStatus::Sending,
            quote: None,
            reactions: Vec::new(),
            expires_at: None,
        };

        // Store the message
        self.store.store_message(&message).await?;

        Ok(message)
    }

    /// Send a message with attachment
    pub async fn send_attachment(
        &self,
        recipient: &SignalIdentity,
        file_path: &Path,
        caption: Option<&str>,
    ) -> Result<Message> {
        tracing::info!("Sending attachment to {:?}: {:?}", recipient.uuid, file_path);

        // Read file
        let file_data = tokio::fs::read(file_path).await?;
        let file_name = file_path
            .file_name()
            .map(|n| n.to_string_lossy().to_string());

        // Detect content type
        let content_type = mime_guess::from_path(file_path)
            .first()
            .map(|m| m.to_string())
            .unwrap_or_else(|| "application/octet-stream".to_string());

        // Generate encryption key for attachment
        let mut attachment_key = [0u8; 32];
        rand::RngCore::fill_bytes(&mut rand::rngs::OsRng, &mut attachment_key);

        // Encrypt attachment
        let nonce = SignalCipher::generate_nonce();
        let encrypted_data = SignalCipher::encrypt(&attachment_key, &nonce, &file_data)?;

        // Calculate digest
        let digest = sha2::Sha256::digest(&encrypted_data).to_vec();

        // TODO: Upload to CDN and get attachment ID
        // For now, create a local attachment reference
        let attachment = Attachment {
            id: Uuid::new_v4().to_string(),
            content_type,
            file_name,
            size: file_data.len() as u64,
            digest,
            key: attachment_key.to_vec(),
            cdn_number: 0,
            upload_timestamp: chrono::Utc::now().timestamp_millis(),
            width: None,
            height: None,
            thumbnail: None,
        };

        // Create message content
        let msg_content = if content_type.starts_with("image/") {
            MessageContent::Image {
                attachment,
                caption: caption.map(|s| s.to_string()),
            }
        } else if content_type.starts_with("video/") {
            MessageContent::Video {
                attachment,
                caption: caption.map(|s| s.to_string()),
            }
        } else if content_type.starts_with("audio/") {
            MessageContent::Audio { attachment }
        } else {
            MessageContent::File { attachment }
        };

        // For now, return error since we don't have CDN upload
        Err(anyhow!("Attachment upload not yet implemented"))
    }

    /// Send typing indicator
    pub async fn send_typing(
        &self,
        recipient: &SignalIdentity,
        action: TypingAction,
    ) -> Result<()> {
        tracing::debug!("Sending typing indicator to {:?}: {:?}", recipient.uuid, action);

        // Create typing message
        let typing_msg = serde_json::json!({
            "type": "typing",
            "action": if matches!(action, TypingAction::Started) { "started" } else { "stopped" },
            "timestamp": chrono::Utc::now().timestamp_millis()
        });

        // TODO: Send via WebSocket
        // For now, just log
        tracing::debug!("Typing message: {}", typing_msg);

        Ok(())
    }

    /// Mark messages as read
    pub async fn mark_read(&self, conversation_id: &str, up_to_timestamp: i64) -> Result<()> {
        tracing::info!(
            "Marking messages read in {} up to {}",
            conversation_id,
            up_to_timestamp
        );

        // Update local store
        // TODO: Send read receipt to sender

        Ok(())
    }

    /// Get all conversations
    pub async fn get_conversations(&self) -> Result<Vec<Conversation>> {
        self.store.get_conversations().await
    }

    /// Get messages for a conversation
    pub async fn get_messages(&self, conversation_id: &str, limit: usize) -> Result<Vec<Message>> {
        self.store.get_messages(conversation_id, limit).await
    }

    /// Get contacts
    pub async fn get_contacts(&self) -> Result<Vec<SignalIdentity>> {
        self.store.get_contacts().await
    }

    /// Sync with primary device
    pub async fn request_sync(&self) -> Result<()> {
        tracing::info!("Requesting sync from primary device");

        // Send sync request message
        // TODO: Implement sync request protocol

        Ok(())
    }

    /// Get safety number for a contact
    pub async fn get_safety_number(&self, contact_id: &str) -> Result<String> {
        let identity = self.identity.as_ref().ok_or_else(|| anyhow!("No identity"))?;

        let protocol = self.protocol.read().await;
        protocol.get_safety_number(&identity.uuid.to_string(), contact_id)
    }

    /// Unlink device and clear all data
    pub async fn unlink(&mut self) -> Result<()> {
        tracing::warn!("Unlinking device");

        // Disconnect
        self.disconnect().await?;

        // Clear store
        self.store.clear().await?;

        // Reset state
        self.identity = None;
        self.device_password = None;
        self.is_linked = false;

        Ok(())
    }
}

/// Device linking session data
pub struct LinkingSession {
    /// Provisioning UUID
    pub provisioning_uuid: String,
    /// Ephemeral key pair for provisioning encryption
    pub ephemeral_key: DhKeyPair,
}

use sha2::Digest;

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_client_creation() {
        let temp_dir = TempDir::new().unwrap();
        let client = SignalClient::new(temp_dir.path()).await.unwrap();

        assert!(!client.is_linked());
        assert!(client.identity().is_none());
    }

    #[tokio::test]
    async fn test_linking_uri_generation() {
        let temp_dir = TempDir::new().unwrap();
        let client = SignalClient::new(temp_dir.path()).await.unwrap();

        let (uri, _session) = client.generate_linking_uri().await.unwrap();

        assert!(uri.starts_with("sgnl://linkdevice?"));
        assert!(uri.contains("uuid="));
        assert!(uri.contains("pub_key="));
    }
}
