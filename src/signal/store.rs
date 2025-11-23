//! Encrypted storage for Signal data
//!
//! Uses SQLCipher for encrypted database storage of messages,
//! conversations, cryptographic keys, and session state.

use anyhow::{anyhow, Result};
use rusqlite::{params, Connection, OptionalExtension};
use std::path::Path;
use std::sync::Arc;
use tokio::sync::Mutex;

use super::protocol::ProtocolAddress;
use super::types::*;

/// Database schema version for migrations
const SCHEMA_VERSION: u32 = 1;

/// Encrypted Signal data store
pub struct SignalStore {
    /// SQLCipher connection (wrapped for async safety)
    db: Arc<Mutex<Connection>>,
    /// Data directory path
    data_dir: std::path::PathBuf,
}

impl SignalStore {
    /// Create or open the encrypted store
    pub async fn new(data_dir: &Path) -> Result<Self> {
        std::fs::create_dir_all(data_dir)?;

        let db_path = data_dir.join("signal.db");
        let db = Connection::open(&db_path)?;

        tracing::info!("Opening Signal store at {:?}", data_dir);

        let store = Self {
            db: Arc::new(Mutex::new(db)),
            data_dir: data_dir.to_path_buf(),
        };

        // Run migrations
        store.migrate().await?;

        Ok(store)
    }

    /// Create with encryption key
    pub async fn new_encrypted(data_dir: &Path, key: &str) -> Result<Self> {
        std::fs::create_dir_all(data_dir)?;

        let db_path = data_dir.join("signal.db");
        let db = Connection::open(&db_path)?;

        // Set SQLCipher encryption key
        db.execute_batch(&format!("PRAGMA key = '{}';", key))?;

        // Verify encryption is working
        db.query_row("SELECT count(*) FROM sqlite_master;", [], |_| Ok(()))
            .map_err(|_| anyhow!("Failed to verify database encryption"))?;

        tracing::info!("Opening encrypted Signal store at {:?}", data_dir);

        let store = Self {
            db: Arc::new(Mutex::new(db)),
            data_dir: data_dir.to_path_buf(),
        };

        store.migrate().await?;

        Ok(store)
    }

    /// Run database migrations
    pub async fn migrate(&self) -> Result<()> {
        let db = self.db.lock().await;

        // Check current schema version
        let current_version: u32 = db
            .query_row(
                "SELECT value FROM metadata WHERE key = 'schema_version'",
                [],
                |row| row.get(0),
            )
            .unwrap_or(0);

        if current_version >= SCHEMA_VERSION {
            return Ok(());
        }

        tracing::info!(
            "Running database migrations from v{} to v{}",
            current_version,
            SCHEMA_VERSION
        );

        // Create tables
        db.execute_batch(
            r#"
            -- Metadata table for schema versioning
            CREATE TABLE IF NOT EXISTS metadata (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL
            );

            -- Identity keys (our own and trusted contacts)
            CREATE TABLE IF NOT EXISTS identities (
                address TEXT PRIMARY KEY,
                public_key BLOB NOT NULL,
                private_key BLOB,
                registration_id INTEGER,
                trusted INTEGER DEFAULT 0,
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL
            );

            -- Pre-keys (one-time keys)
            CREATE TABLE IF NOT EXISTS pre_keys (
                id INTEGER PRIMARY KEY,
                public_key BLOB NOT NULL,
                private_key BLOB NOT NULL,
                created_at INTEGER NOT NULL
            );

            -- Signed pre-keys
            CREATE TABLE IF NOT EXISTS signed_pre_keys (
                id INTEGER PRIMARY KEY,
                public_key BLOB NOT NULL,
                private_key BLOB NOT NULL,
                signature BLOB NOT NULL,
                timestamp INTEGER NOT NULL,
                created_at INTEGER NOT NULL
            );

            -- Sessions (Double Ratchet state)
            CREATE TABLE IF NOT EXISTS sessions (
                address TEXT PRIMARY KEY,
                session_data BLOB NOT NULL,
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL
            );

            -- Conversations
            CREATE TABLE IF NOT EXISTS conversations (
                id TEXT PRIMARY KEY,
                recipient_uuid TEXT NOT NULL,
                recipient_device_id INTEGER DEFAULT 1,
                is_group INTEGER DEFAULT 0,
                group_id TEXT,
                name TEXT NOT NULL,
                archived INTEGER DEFAULT 0,
                muted_until INTEGER,
                unread_count INTEGER DEFAULT 0,
                last_message_id TEXT,
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL
            );

            CREATE INDEX IF NOT EXISTS idx_conversations_updated
                ON conversations(updated_at DESC);

            -- Messages
            CREATE TABLE IF NOT EXISTS messages (
                id TEXT PRIMARY KEY,
                conversation_id TEXT NOT NULL,
                sender_uuid TEXT NOT NULL,
                sender_device_id INTEGER DEFAULT 1,
                timestamp INTEGER NOT NULL,
                received_timestamp INTEGER,
                content_type TEXT NOT NULL,
                content_json TEXT NOT NULL,
                status TEXT NOT NULL,
                quote_id TEXT,
                expires_at INTEGER,
                created_at INTEGER NOT NULL,
                FOREIGN KEY (conversation_id) REFERENCES conversations(id)
            );

            CREATE INDEX IF NOT EXISTS idx_messages_conversation
                ON messages(conversation_id, timestamp DESC);

            -- Attachments
            CREATE TABLE IF NOT EXISTS attachments (
                id TEXT PRIMARY KEY,
                message_id TEXT NOT NULL,
                content_type TEXT NOT NULL,
                file_name TEXT,
                size INTEGER NOT NULL,
                digest BLOB,
                key BLOB,
                cdn_number INTEGER,
                local_path TEXT,
                thumbnail BLOB,
                width INTEGER,
                height INTEGER,
                created_at INTEGER NOT NULL,
                FOREIGN KEY (message_id) REFERENCES messages(id)
            );

            CREATE INDEX IF NOT EXISTS idx_attachments_message
                ON attachments(message_id);

            -- Reactions
            CREATE TABLE IF NOT EXISTS reactions (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                message_id TEXT NOT NULL,
                emoji TEXT NOT NULL,
                sender_uuid TEXT NOT NULL,
                timestamp INTEGER NOT NULL,
                FOREIGN KEY (message_id) REFERENCES messages(id),
                UNIQUE(message_id, sender_uuid)
            );

            -- Contacts
            CREATE TABLE IF NOT EXISTS contacts (
                uuid TEXT PRIMARY KEY,
                phone_number TEXT,
                name TEXT,
                profile_name TEXT,
                profile_key BLOB,
                avatar_path TEXT,
                blocked INTEGER DEFAULT 0,
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL
            );

            -- Groups
            CREATE TABLE IF NOT EXISTS groups (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                description TEXT,
                avatar_path TEXT,
                disappearing_timer INTEGER,
                access_members INTEGER DEFAULT 1,
                access_info INTEGER DEFAULT 1,
                revision INTEGER DEFAULT 0,
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL
            );

            -- Group members
            CREATE TABLE IF NOT EXISTS group_members (
                group_id TEXT NOT NULL,
                member_uuid TEXT NOT NULL,
                role TEXT NOT NULL,
                joined_at INTEGER NOT NULL,
                PRIMARY KEY (group_id, member_uuid),
                FOREIGN KEY (group_id) REFERENCES groups(id)
            );

            -- Sender key distribution (for group messaging)
            CREATE TABLE IF NOT EXISTS sender_keys (
                address TEXT NOT NULL,
                distribution_id TEXT NOT NULL,
                key_data BLOB NOT NULL,
                created_at INTEGER NOT NULL,
                PRIMARY KEY (address, distribution_id)
            );
            "#,
        )?;

        // Update schema version
        db.execute(
            "INSERT OR REPLACE INTO metadata (key, value) VALUES ('schema_version', ?)",
            params![SCHEMA_VERSION.to_string()],
        )?;

        tracing::info!("Database migrations complete");

        Ok(())
    }

    // ==================== Identity Operations ====================

    /// Store our identity keys
    pub async fn store_local_identity(
        &self,
        public_key: &[u8],
        private_key: &[u8],
        registration_id: u32,
    ) -> Result<()> {
        let db = self.db.lock().await;
        let now = chrono::Utc::now().timestamp();

        db.execute(
            r#"INSERT OR REPLACE INTO identities
               (address, public_key, private_key, registration_id, trusted, created_at, updated_at)
               VALUES ('local', ?, ?, ?, 1, ?, ?)"#,
            params![public_key, private_key, registration_id, now, now],
        )?;

        tracing::info!("Stored local identity");
        Ok(())
    }

    /// Get our local identity
    pub async fn get_local_identity(&self) -> Result<Option<(Vec<u8>, Vec<u8>, u32)>> {
        let db = self.db.lock().await;

        let result = db
            .query_row(
                "SELECT public_key, private_key, registration_id FROM identities WHERE address = 'local'",
                [],
                |row| Ok((row.get::<_, Vec<u8>>(0)?, row.get::<_, Vec<u8>>(1)?, row.get::<_, u32>(2)?)),
            )
            .optional()?;

        Ok(result)
    }

    /// Store identity (our own or trusted contact)
    pub async fn store_identity(&self, identity: &SignalIdentity, keys: &[u8]) -> Result<()> {
        let db = self.db.lock().await;
        let now = chrono::Utc::now().timestamp();
        let address = format!("{}.{}", identity.uuid, identity.device_id);

        db.execute(
            r#"INSERT OR REPLACE INTO identities
               (address, public_key, registration_id, trusted, created_at, updated_at)
               VALUES (?, ?, ?, 1, ?, ?)"#,
            params![address, keys, identity.registration_id, now, now],
        )?;

        tracing::info!("Stored identity for {:?}", identity.uuid);
        Ok(())
    }

    /// Get stored identity
    pub async fn get_identity(&self) -> Result<Option<SignalIdentity>> {
        let db = self.db.lock().await;

        let result = db
            .query_row(
                "SELECT public_key, registration_id FROM identities WHERE address = 'local'",
                [],
                |row| {
                    Ok(SignalIdentity {
                        uuid: uuid::Uuid::nil(),
                        phone_number: None,
                        device_id: 1,
                        registration_id: row.get(1)?,
                    })
                },
            )
            .optional()?;

        Ok(result)
    }

    /// Check if an identity is trusted
    pub async fn is_identity_trusted(&self, address: &ProtocolAddress) -> Result<bool> {
        let db = self.db.lock().await;
        let addr_str = address.to_string();

        let trusted: bool = db
            .query_row(
                "SELECT trusted FROM identities WHERE address = ?",
                params![addr_str],
                |row| row.get(0),
            )
            .unwrap_or(false);

        Ok(trusted)
    }

    // ==================== Pre-Key Operations ====================

    /// Store pre-keys
    pub async fn store_pre_keys(&self, keys: &[(u32, Vec<u8>, Vec<u8>)]) -> Result<()> {
        let db = self.db.lock().await;
        let now = chrono::Utc::now().timestamp();

        for (id, public_key, private_key) in keys {
            db.execute(
                "INSERT OR REPLACE INTO pre_keys (id, public_key, private_key, created_at) VALUES (?, ?, ?, ?)",
                params![id, public_key, private_key, now],
            )?;
        }

        tracing::info!("Stored {} pre-keys", keys.len());
        Ok(())
    }

    /// Get pre-key by ID
    pub async fn get_pre_key(&self, id: u32) -> Result<Option<(Vec<u8>, Vec<u8>)>> {
        let db = self.db.lock().await;

        let result = db
            .query_row(
                "SELECT public_key, private_key FROM pre_keys WHERE id = ?",
                params![id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .optional()?;

        Ok(result)
    }

    /// Remove used pre-key
    pub async fn remove_pre_key(&self, id: u32) -> Result<()> {
        let db = self.db.lock().await;
        db.execute("DELETE FROM pre_keys WHERE id = ?", params![id])?;
        tracing::info!("Removed pre-key {}", id);
        Ok(())
    }

    /// Get pre-key count
    pub async fn pre_key_count(&self) -> Result<usize> {
        let db = self.db.lock().await;
        let count: usize = db.query_row("SELECT COUNT(*) FROM pre_keys", [], |row| row.get(0))?;
        Ok(count)
    }

    /// Store signed pre-key
    pub async fn store_signed_pre_key(
        &self,
        id: u32,
        public_key: &[u8],
        private_key: &[u8],
        signature: &[u8],
        timestamp: i64,
    ) -> Result<()> {
        let db = self.db.lock().await;
        let now = chrono::Utc::now().timestamp();

        db.execute(
            r#"INSERT OR REPLACE INTO signed_pre_keys
               (id, public_key, private_key, signature, timestamp, created_at)
               VALUES (?, ?, ?, ?, ?, ?)"#,
            params![id, public_key, private_key, signature, timestamp, now],
        )?;

        tracing::info!("Stored signed pre-key {}", id);
        Ok(())
    }

    /// Get signed pre-key
    pub async fn get_signed_pre_key(
        &self,
        id: u32,
    ) -> Result<Option<(Vec<u8>, Vec<u8>, Vec<u8>, i64)>> {
        let db = self.db.lock().await;

        let result = db
            .query_row(
                "SELECT public_key, private_key, signature, timestamp FROM signed_pre_keys WHERE id = ?",
                params![id],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
            )
            .optional()?;

        Ok(result)
    }

    // ==================== Session Operations ====================

    /// Store session
    pub async fn store_session(&self, address: &ProtocolAddress, session_data: &[u8]) -> Result<()> {
        let db = self.db.lock().await;
        let now = chrono::Utc::now().timestamp();
        let addr_str = address.to_string();

        db.execute(
            r#"INSERT OR REPLACE INTO sessions (address, session_data, created_at, updated_at)
               VALUES (?, ?, ?, ?)"#,
            params![addr_str, session_data, now, now],
        )?;

        tracing::debug!("Stored session for {}", addr_str);
        Ok(())
    }

    /// Get session
    pub async fn get_session(&self, address: &ProtocolAddress) -> Result<Option<Vec<u8>>> {
        let db = self.db.lock().await;
        let addr_str = address.to_string();

        let result = db
            .query_row(
                "SELECT session_data FROM sessions WHERE address = ?",
                params![addr_str],
                |row| row.get(0),
            )
            .optional()?;

        Ok(result)
    }

    /// Check if session exists
    pub async fn has_session(&self, address: &ProtocolAddress) -> Result<bool> {
        let db = self.db.lock().await;
        let addr_str = address.to_string();

        let count: i32 = db.query_row(
            "SELECT COUNT(*) FROM sessions WHERE address = ?",
            params![addr_str],
            |row| row.get(0),
        )?;

        Ok(count > 0)
    }

    /// Delete session
    pub async fn delete_session(&self, address: &ProtocolAddress) -> Result<()> {
        let db = self.db.lock().await;
        let addr_str = address.to_string();

        db.execute("DELETE FROM sessions WHERE address = ?", params![addr_str])?;

        tracing::info!("Deleted session for {}", addr_str);
        Ok(())
    }

    // ==================== Conversation Operations ====================

    /// Store a conversation
    pub async fn store_conversation(&self, conversation: &Conversation) -> Result<()> {
        let db = self.db.lock().await;
        let now = chrono::Utc::now().timestamp();

        db.execute(
            r#"INSERT OR REPLACE INTO conversations
               (id, recipient_uuid, recipient_device_id, is_group, group_id, name,
                archived, muted_until, unread_count, last_message_id, created_at, updated_at)
               VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"#,
            params![
                conversation.id,
                conversation.recipient.uuid.to_string(),
                conversation.recipient.device_id,
                conversation.is_group,
                conversation.group_id,
                conversation.name,
                conversation.archived,
                conversation.muted_until,
                conversation.unread_count,
                conversation.last_message.as_ref().map(|m| &m.id),
                now,
                now,
            ],
        )?;

        tracing::info!("Stored conversation {}", conversation.id);
        Ok(())
    }

    /// Get all conversations
    pub async fn get_conversations(&self) -> Result<Vec<Conversation>> {
        let db = self.db.lock().await;

        let mut stmt = db.prepare(
            r#"SELECT id, recipient_uuid, recipient_device_id, is_group, group_id, name,
                      archived, muted_until, unread_count
               FROM conversations ORDER BY updated_at DESC"#,
        )?;

        let conversations = stmt
            .query_map([], |row| {
                Ok(Conversation {
                    id: row.get(0)?,
                    recipient: SignalIdentity {
                        uuid: row
                            .get::<_, String>(1)?
                            .parse()
                            .unwrap_or(uuid::Uuid::nil()),
                        phone_number: None,
                        device_id: row.get(2)?,
                        registration_id: 0,
                    },
                    is_group: row.get(3)?,
                    group_id: row.get(4)?,
                    name: row.get(5)?,
                    last_message: None,
                    unread_count: row.get(8)?,
                    archived: row.get(6)?,
                    muted_until: row.get(7)?,
                })
            })?
            .filter_map(|r| r.ok())
            .collect();

        Ok(conversations)
    }

    /// Get a specific conversation
    pub async fn get_conversation(&self, id: &str) -> Result<Option<Conversation>> {
        let db = self.db.lock().await;

        let result = db
            .query_row(
                r#"SELECT id, recipient_uuid, recipient_device_id, is_group, group_id, name,
                          archived, muted_until, unread_count
                   FROM conversations WHERE id = ?"#,
                params![id],
                |row| {
                    Ok(Conversation {
                        id: row.get(0)?,
                        recipient: SignalIdentity {
                            uuid: row
                                .get::<_, String>(1)?
                                .parse()
                                .unwrap_or(uuid::Uuid::nil()),
                            phone_number: None,
                            device_id: row.get(2)?,
                            registration_id: 0,
                        },
                        is_group: row.get(3)?,
                        group_id: row.get(4)?,
                        name: row.get(5)?,
                        last_message: None,
                        unread_count: row.get(8)?,
                        archived: row.get(6)?,
                        muted_until: row.get(7)?,
                    })
                },
            )
            .optional()?;

        Ok(result)
    }

    // ==================== Message Operations ====================

    /// Store a message
    pub async fn store_message(&self, message: &Message) -> Result<()> {
        let db = self.db.lock().await;
        let now = chrono::Utc::now().timestamp();

        let content_type = match &message.content {
            MessageContent::Text { .. } => "text",
            MessageContent::Image { .. } => "image",
            MessageContent::Video { .. } => "video",
            MessageContent::Audio { .. } => "audio",
            MessageContent::File { .. } => "file",
            MessageContent::Voice { .. } => "voice",
            MessageContent::Sticker { .. } => "sticker",
            MessageContent::Contact { .. } => "contact",
            MessageContent::Location { .. } => "location",
        };

        let content_json = serde_json::to_string(&message.content)?;
        let status = format!("{:?}", message.status);

        db.execute(
            r#"INSERT OR REPLACE INTO messages
               (id, conversation_id, sender_uuid, sender_device_id, timestamp,
                received_timestamp, content_type, content_json, status, quote_id,
                expires_at, created_at)
               VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"#,
            params![
                message.id,
                message.conversation_id,
                message.sender.uuid.to_string(),
                message.sender.device_id,
                message.timestamp,
                message.received_timestamp,
                content_type,
                content_json,
                status,
                message.quote.as_ref().map(|q| &q.id),
                message.expires_at,
                now,
            ],
        )?;

        // Update conversation's last message
        db.execute(
            "UPDATE conversations SET last_message_id = ?, updated_at = ? WHERE id = ?",
            params![message.id, now, message.conversation_id],
        )?;

        tracing::debug!("Stored message {}", message.id);
        Ok(())
    }

    /// Get messages for a conversation
    pub async fn get_messages(&self, conversation_id: &str, limit: usize) -> Result<Vec<Message>> {
        let db = self.db.lock().await;

        let mut stmt = db.prepare(
            r#"SELECT id, conversation_id, sender_uuid, sender_device_id, timestamp,
                      received_timestamp, content_type, content_json, status, expires_at
               FROM messages WHERE conversation_id = ?
               ORDER BY timestamp DESC LIMIT ?"#,
        )?;

        let messages = stmt
            .query_map(params![conversation_id, limit], |row| {
                let content_json: String = row.get(7)?;
                let content: MessageContent =
                    serde_json::from_str(&content_json).unwrap_or(MessageContent::Text {
                        body: "[Error loading message]".to_string(),
                    });

                let status_str: String = row.get(8)?;
                let status = match status_str.as_str() {
                    "Sending" => MessageStatus::Sending,
                    "Sent" => MessageStatus::Sent,
                    "Delivered" => MessageStatus::Delivered,
                    "Read" => MessageStatus::Read,
                    _ => MessageStatus::Failed,
                };

                Ok(Message {
                    id: row.get(0)?,
                    conversation_id: row.get(1)?,
                    sender: SignalIdentity {
                        uuid: row
                            .get::<_, String>(2)?
                            .parse()
                            .unwrap_or(uuid::Uuid::nil()),
                        phone_number: None,
                        device_id: row.get(3)?,
                        registration_id: 0,
                    },
                    timestamp: row.get(4)?,
                    received_timestamp: row.get(5)?,
                    content,
                    status,
                    quote: None,
                    reactions: Vec::new(),
                    expires_at: row.get(9)?,
                })
            })?
            .filter_map(|r| r.ok())
            .collect();

        Ok(messages)
    }

    /// Update message status
    pub async fn update_message_status(&self, message_id: &str, status: MessageStatus) -> Result<()> {
        let db = self.db.lock().await;
        let status_str = format!("{:?}", status);

        db.execute(
            "UPDATE messages SET status = ? WHERE id = ?",
            params![status_str, message_id],
        )?;

        tracing::debug!("Updated message {} status to {:?}", message_id, status);
        Ok(())
    }

    // ==================== Contact Operations ====================

    /// Store a contact
    pub async fn store_contact(&self, contact: &SignalIdentity) -> Result<()> {
        let db = self.db.lock().await;
        let now = chrono::Utc::now().timestamp();

        db.execute(
            r#"INSERT OR REPLACE INTO contacts
               (uuid, phone_number, created_at, updated_at)
               VALUES (?, ?, ?, ?)"#,
            params![
                contact.uuid.to_string(),
                contact.phone_number,
                now,
                now,
            ],
        )?;

        tracing::info!("Stored contact {:?}", contact.uuid);
        Ok(())
    }

    /// Get all contacts
    pub async fn get_contacts(&self) -> Result<Vec<SignalIdentity>> {
        let db = self.db.lock().await;

        let mut stmt = db.prepare(
            "SELECT uuid, phone_number FROM contacts WHERE blocked = 0 ORDER BY name",
        )?;

        let contacts = stmt
            .query_map([], |row| {
                Ok(SignalIdentity {
                    uuid: row
                        .get::<_, String>(0)?
                        .parse()
                        .unwrap_or(uuid::Uuid::nil()),
                    phone_number: row.get(1)?,
                    device_id: 1,
                    registration_id: 0,
                })
            })?
            .filter_map(|r| r.ok())
            .collect();

        Ok(contacts)
    }

    // ==================== Group Operations ====================

    /// Store a group
    pub async fn store_group(&self, group: &Group) -> Result<()> {
        let db = self.db.lock().await;
        let now = chrono::Utc::now().timestamp();

        db.execute(
            r#"INSERT OR REPLACE INTO groups
               (id, name, description, disappearing_timer, access_members, access_info,
                created_at, updated_at)
               VALUES (?, ?, ?, ?, ?, ?, ?, ?)"#,
            params![
                group.id,
                group.name,
                group.description,
                group.disappearing_messages_timer,
                group.access_control.members_can_add_members,
                group.access_control.members_can_edit_group_info,
                now,
                now,
            ],
        )?;

        // Store members
        for member in &group.members {
            db.execute(
                r#"INSERT OR REPLACE INTO group_members (group_id, member_uuid, role, joined_at)
                   VALUES (?, ?, ?, ?)"#,
                params![
                    group.id,
                    member.uuid.to_string(),
                    format!("{:?}", member.role),
                    member.joined_at,
                ],
            )?;
        }

        tracing::info!("Stored group {}", group.id);
        Ok(())
    }

    /// Get a group
    pub async fn get_group(&self, id: &str) -> Result<Option<Group>> {
        let db = self.db.lock().await;

        let group = db
            .query_row(
                r#"SELECT id, name, description, disappearing_timer, access_members, access_info
                   FROM groups WHERE id = ?"#,
                params![id],
                |row| {
                    Ok(Group {
                        id: row.get(0)?,
                        name: row.get(1)?,
                        description: row.get(2)?,
                        avatar: None,
                        members: Vec::new(),
                        admins: Vec::new(),
                        pending_members: Vec::new(),
                        disappearing_messages_timer: row.get(3)?,
                        access_control: GroupAccessControl {
                            members_can_add_members: row.get(4)?,
                            members_can_edit_group_info: row.get(5)?,
                        },
                    })
                },
            )
            .optional()?;

        if let Some(mut group) = group {
            // Load members
            let mut stmt = db.prepare(
                "SELECT member_uuid, role, joined_at FROM group_members WHERE group_id = ?",
            )?;

            group.members = stmt
                .query_map(params![id], |row| {
                    let role_str: String = row.get(1)?;
                    let role = if role_str == "Administrator" {
                        GroupRole::Administrator
                    } else {
                        GroupRole::Member
                    };

                    Ok(GroupMember {
                        uuid: row
                            .get::<_, String>(0)?
                            .parse()
                            .unwrap_or(uuid::Uuid::nil()),
                        role,
                        joined_at: row.get(2)?,
                    })
                })?
                .filter_map(|r| r.ok())
                .collect();

            return Ok(Some(group));
        }

        Ok(None)
    }

    // ==================== Utility Operations ====================

    /// Clear all data (for account unlinking)
    pub async fn clear(&self) -> Result<()> {
        let db = self.db.lock().await;

        tracing::warn!("Clearing all Signal data");

        db.execute_batch(
            r#"
            DELETE FROM sender_keys;
            DELETE FROM group_members;
            DELETE FROM groups;
            DELETE FROM contacts;
            DELETE FROM reactions;
            DELETE FROM attachments;
            DELETE FROM messages;
            DELETE FROM conversations;
            DELETE FROM sessions;
            DELETE FROM signed_pre_keys;
            DELETE FROM pre_keys;
            DELETE FROM identities;
            "#,
        )?;

        Ok(())
    }

    /// Get database path
    pub fn data_dir(&self) -> &Path {
        &self.data_dir
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_store_and_get_conversation() {
        let temp_dir = TempDir::new().unwrap();
        let store = SignalStore::new(temp_dir.path()).await.unwrap();

        let conversation = Conversation {
            id: "test-conv-1".to_string(),
            recipient: SignalIdentity {
                uuid: uuid::Uuid::new_v4(),
                phone_number: Some("+1234567890".to_string()),
                device_id: 1,
                registration_id: 12345,
            },
            is_group: false,
            group_id: None,
            name: "Test Contact".to_string(),
            last_message: None,
            unread_count: 0,
            archived: false,
            muted_until: None,
        };

        store.store_conversation(&conversation).await.unwrap();

        let retrieved = store.get_conversation("test-conv-1").await.unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().name, "Test Contact");
    }

    #[tokio::test]
    async fn test_store_and_get_message() {
        let temp_dir = TempDir::new().unwrap();
        let store = SignalStore::new(temp_dir.path()).await.unwrap();

        // Create conversation first
        let conversation = Conversation {
            id: "test-conv-1".to_string(),
            recipient: SignalIdentity {
                uuid: uuid::Uuid::new_v4(),
                phone_number: None,
                device_id: 1,
                registration_id: 0,
            },
            is_group: false,
            group_id: None,
            name: "Test".to_string(),
            last_message: None,
            unread_count: 0,
            archived: false,
            muted_until: None,
        };
        store.store_conversation(&conversation).await.unwrap();

        let message = Message {
            id: "msg-1".to_string(),
            conversation_id: "test-conv-1".to_string(),
            sender: conversation.recipient.clone(),
            timestamp: chrono::Utc::now().timestamp_millis(),
            received_timestamp: None,
            content: MessageContent::Text {
                body: "Hello, World!".to_string(),
            },
            status: MessageStatus::Sent,
            quote: None,
            reactions: Vec::new(),
            expires_at: None,
        };

        store.store_message(&message).await.unwrap();

        let messages = store.get_messages("test-conv-1", 10).await.unwrap();
        assert_eq!(messages.len(), 1);

        if let MessageContent::Text { body } = &messages[0].content {
            assert_eq!(body, "Hello, World!");
        } else {
            panic!("Expected text message");
        }
    }

    #[tokio::test]
    async fn test_session_storage() {
        let temp_dir = TempDir::new().unwrap();
        let store = SignalStore::new(temp_dir.path()).await.unwrap();

        let address = ProtocolAddress::new("alice", 1);
        let session_data = b"test session data".to_vec();

        store.store_session(&address, &session_data).await.unwrap();

        let retrieved = store.get_session(&address).await.unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap(), session_data);
    }
}
