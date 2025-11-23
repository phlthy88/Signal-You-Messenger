//! Signal data types

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Represents a Signal user identity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignalIdentity {
    pub uuid: Uuid,
    pub phone_number: Option<String>,
    pub device_id: u32,
    pub registration_id: u32,
}

/// Represents a chat conversation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Conversation {
    pub id: String,
    pub recipient: SignalIdentity,
    pub is_group: bool,
    pub group_id: Option<String>,
    pub name: String,
    pub last_message: Option<Message>,
    pub unread_count: u32,
    pub archived: bool,
    pub muted_until: Option<i64>,
}

/// Represents a message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub id: String,
    pub conversation_id: String,
    pub sender: SignalIdentity,
    pub timestamp: i64,
    pub received_timestamp: Option<i64>,
    pub content: MessageContent,
    pub status: MessageStatus,
    pub quote: Option<Box<Message>>,
    pub reactions: Vec<Reaction>,
    pub expires_at: Option<i64>,
}

/// Message content types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum MessageContent {
    Text { body: String },
    Image { attachment: Attachment, caption: Option<String> },
    Video { attachment: Attachment, caption: Option<String> },
    Audio { attachment: Attachment },
    File { attachment: Attachment },
    Voice { attachment: Attachment, duration_ms: u32 },
    Sticker { pack_id: String, sticker_id: u32 },
    Contact { contact: ContactInfo },
    Location { latitude: f64, longitude: f64, name: Option<String> },
}

/// Message delivery status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MessageStatus {
    Sending,
    Sent,
    Delivered,
    Read,
    Failed,
}

/// File attachment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Attachment {
    pub id: String,
    pub content_type: String,
    pub file_name: Option<String>,
    pub size: u64,
    pub digest: Vec<u8>,
    pub key: Vec<u8>,
    pub cdn_number: u32,
    pub upload_timestamp: i64,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub thumbnail: Option<Vec<u8>>,
}

/// Message reaction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Reaction {
    pub emoji: String,
    pub sender: SignalIdentity,
    pub timestamp: i64,
}

/// Contact information for sharing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContactInfo {
    pub name: String,
    pub phone_numbers: Vec<String>,
    pub emails: Vec<String>,
}

/// Group information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Group {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub avatar: Option<Attachment>,
    pub members: Vec<GroupMember>,
    pub admins: Vec<Uuid>,
    pub pending_members: Vec<GroupMember>,
    pub disappearing_messages_timer: Option<u32>,
    pub access_control: GroupAccessControl,
}

/// Group member
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupMember {
    pub uuid: Uuid,
    pub role: GroupRole,
    pub joined_at: i64,
}

/// Group member role
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GroupRole {
    Member,
    Administrator,
}

/// Group access control settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupAccessControl {
    pub members_can_add_members: bool,
    pub members_can_edit_group_info: bool,
}

/// Device linking provisioning data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProvisioningData {
    pub uuid: Uuid,
    pub phone_number: String,
    pub provisioning_code: String,
    pub provisioning_cipher: Vec<u8>,
}

/// Sync message types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum SyncMessage {
    SentMessage { message: Message, destination: SignalIdentity },
    ReadMessages { messages: Vec<(String, i64)> },
    Contacts { contacts: Vec<SignalIdentity> },
    Groups { groups: Vec<Group> },
    Blocked { identities: Vec<SignalIdentity> },
    Configuration { read_receipts: bool, typing_indicators: bool },
}

/// Typing indicator status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TypingAction {
    Started,
    Stopped,
}

/// Signal server endpoints
pub struct SignalServers {
    pub service: &'static str,
    pub storage: &'static str,
    pub cdn: &'static str,
    pub cdn2: &'static str,
    pub cdn3: &'static str,
}

impl Default for SignalServers {
    fn default() -> Self {
        Self {
            service: "https://chat.signal.org",
            storage: "https://storage.signal.org",
            cdn: "https://cdn.signal.org",
            cdn2: "https://cdn2.signal.org",
            cdn3: "https://cdn3.signal.org",
        }
    }
}
