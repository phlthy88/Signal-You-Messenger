//! Signal Protocol implementation
//!
//! This module provides the high-level Signal Protocol interface for
//! establishing sessions, encrypting/decrypting messages, and managing keys.

use anyhow::{anyhow, Result};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use x25519_dalek::PublicKey as X25519PublicKey;

use super::crypto::{
    calculate_fingerprint, DhKeyPair, IdentityKeyPair, IdentityPublicKey, PreKey, PreKeyBundle,
    SignedPreKey,
};
use super::ratchet::{RatchetMessage, SessionState};
use super::x3dh::{x3dh_initiate, x3dh_respond, InitialMessage};

/// Number of pre-keys to generate at a time
const PRE_KEY_BATCH_SIZE: u32 = 100;

/// Maximum pre-key ID before wrapping
const MAX_PRE_KEY_ID: u32 = 0x00FFFFFF;

/// Signal Protocol address (user + device)
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ProtocolAddress {
    pub name: String,
    pub device_id: u32,
}

impl ProtocolAddress {
    pub fn new(name: impl Into<String>, device_id: u32) -> Self {
        Self {
            name: name.into(),
            device_id,
        }
    }

    /// Convert to string representation
    pub fn to_string(&self) -> String {
        format!("{}.{}", self.name, self.device_id)
    }

    /// Parse from string representation
    pub fn from_string(s: &str) -> Result<Self> {
        let parts: Vec<&str> = s.rsplitn(2, '.').collect();
        if parts.len() != 2 {
            return Err(anyhow!("Invalid address format"));
        }
        let device_id: u32 = parts[0].parse()?;
        let name = parts[1].to_string();
        Ok(Self { name, device_id })
    }
}

/// Signal Protocol wrapper with session management
pub struct SignalProtocol {
    /// Our identity key pair
    identity_key: IdentityKeyPair,
    /// Our registration ID
    registration_id: u32,
    /// Pre-keys (one-time keys)
    pre_keys: HashMap<u32, PreKey>,
    /// Next pre-key ID
    next_pre_key_id: u32,
    /// Signed pre-key
    signed_pre_key: Option<SignedPreKey>,
    /// Active sessions with other users
    sessions: Arc<RwLock<HashMap<ProtocolAddress, SessionState>>>,
    /// Trusted identity keys
    trusted_identities: HashMap<String, IdentityPublicKey>,
}

impl SignalProtocol {
    /// Create a new protocol instance with fresh identity
    pub fn new() -> Result<Self> {
        let identity_key = IdentityKeyPair::generate();
        let registration_id = rand::random::<u32>() & 0x3FFF; // 14-bit ID

        tracing::info!("Generated new identity key pair, registration_id={}", registration_id);

        Ok(Self {
            identity_key,
            registration_id,
            pre_keys: HashMap::new(),
            next_pre_key_id: 1,
            signed_pre_key: None,
            sessions: Arc::new(RwLock::new(HashMap::new())),
            trusted_identities: HashMap::new(),
        })
    }

    /// Create from existing identity key
    pub fn from_identity(private_key: &[u8; 32], registration_id: u32) -> Result<Self> {
        let identity_key = IdentityKeyPair::from_private_key(private_key)?;

        Ok(Self {
            identity_key,
            registration_id,
            pre_keys: HashMap::new(),
            next_pre_key_id: 1,
            signed_pre_key: None,
            sessions: Arc::new(RwLock::new(HashMap::new())),
            trusted_identities: HashMap::new(),
        })
    }

    /// Get our identity public key
    pub fn identity_public_key(&self) -> IdentityPublicKey {
        self.identity_key.public_key()
    }

    /// Get our identity private key bytes (for secure storage)
    pub fn identity_private_key(&self) -> [u8; 32] {
        self.identity_key.private_key_bytes()
    }

    /// Get our registration ID
    pub fn registration_id(&self) -> u32 {
        self.registration_id
    }

    /// Generate identity key pair
    pub fn generate_identity_key_pair(&self) -> Result<(Vec<u8>, Vec<u8>)> {
        let key_pair = IdentityKeyPair::generate();
        let public_key = key_pair.public_key().as_bytes().to_vec();
        let private_key = key_pair.private_key_bytes().to_vec();

        tracing::info!("Generated identity key pair");

        Ok((public_key, private_key))
    }

    /// Generate a batch of pre-keys
    pub fn generate_pre_keys(&mut self, count: u32) -> Result<Vec<(u32, Vec<u8>)>> {
        let start_id = self.next_pre_key_id;
        let mut result = Vec::with_capacity(count as usize);

        for i in 0..count {
            let id = (start_id + i) % MAX_PRE_KEY_ID;
            let pre_key = PreKey::generate(id);

            // Store the pre-key
            let public_key = pre_key.key_pair.public_key().as_bytes().to_vec();
            self.pre_keys.insert(id, pre_key);

            result.push((id, public_key));
        }

        self.next_pre_key_id = (start_id + count) % MAX_PRE_KEY_ID;

        tracing::info!("Generated {} pre-keys starting at {}", count, start_id);

        Ok(result)
    }

    /// Generate a signed pre-key
    pub fn generate_signed_pre_key(&mut self, id: u32) -> Result<(Vec<u8>, Vec<u8>)> {
        let signed_pre_key = SignedPreKey::generate(id, &self.identity_key);

        let public_key = signed_pre_key.key_pair.public_key().as_bytes().to_vec();
        let signature = signed_pre_key.signature.to_vec();

        self.signed_pre_key = Some(signed_pre_key);

        tracing::info!("Generated signed pre-key {}", id);

        Ok((public_key, signature))
    }

    /// Get the current signed pre-key for publishing
    pub fn get_signed_pre_key(&self) -> Option<&SignedPreKey> {
        self.signed_pre_key.as_ref()
    }

    /// Create our pre-key bundle for publishing to the server
    pub fn create_pre_key_bundle(&self, device_id: u32) -> Result<PreKeyBundle> {
        let signed_pre_key = self
            .signed_pre_key
            .as_ref()
            .ok_or_else(|| anyhow!("No signed pre-key available"))?;

        // Get a one-time pre-key if available
        let (pre_key_id, pre_key_public) = self
            .pre_keys
            .iter()
            .next()
            .map(|(id, pk)| (Some(*id), Some(*pk.key_pair.public_key())))
            .unwrap_or((None, None));

        Ok(PreKeyBundle {
            registration_id: self.registration_id,
            device_id,
            pre_key_id,
            pre_key_public,
            signed_pre_key_id: signed_pre_key.id,
            signed_pre_key_public: *signed_pre_key.key_pair.public_key(),
            signed_pre_key_signature: signed_pre_key.signature,
            identity_key: self.identity_key.public_key(),
        })
    }

    /// Process a pre-key bundle to establish a session
    pub async fn process_pre_key_bundle(
        &self,
        address: &ProtocolAddress,
        bundle: &PreKeyBundle,
    ) -> Result<()> {
        // Verify the bundle
        bundle.verify()?;

        // Check if we trust this identity
        if let Some(trusted) = self.trusted_identities.get(&address.name) {
            if trusted.as_bytes() != bundle.identity_key.as_bytes() {
                return Err(anyhow!("Identity key mismatch for {}", address.name));
            }
        }

        // Perform X3DH key agreement
        let x3dh_result = x3dh_initiate(&self.identity_key, bundle)?;

        // Initialize the Double Ratchet session
        let our_ratchet_key = DhKeyPair::generate();
        let session = SessionState::initialize_alice(
            &x3dh_result.shared_secret,
            our_ratchet_key,
            &bundle.signed_pre_key_public,
        )?;

        // Store the session
        let mut sessions = self.sessions.write().await;
        sessions.insert(address.clone(), session);

        tracing::info!("Established session with {}", address.to_string());

        Ok(())
    }

    /// Encrypt a message for a recipient
    pub async fn encrypt(&self, address: &ProtocolAddress, plaintext: &[u8]) -> Result<Vec<u8>> {
        let mut sessions = self.sessions.write().await;

        let session = sessions
            .get_mut(address)
            .ok_or_else(|| anyhow!("No session for {}", address.to_string()))?;

        let message = session.encrypt(plaintext)?;

        tracing::debug!("Encrypted message for {}", address.to_string());

        Ok(message.serialize())
    }

    /// Encrypt initial message (when no session exists, includes X3DH data)
    pub async fn encrypt_initial(
        &self,
        address: &ProtocolAddress,
        bundle: &PreKeyBundle,
        plaintext: &[u8],
    ) -> Result<Vec<u8>> {
        // Perform X3DH key agreement
        let x3dh_result = x3dh_initiate(&self.identity_key, bundle)?;

        // Initialize session
        let our_ratchet_key = DhKeyPair::generate();
        let mut session = SessionState::initialize_alice(
            &x3dh_result.shared_secret,
            our_ratchet_key,
            &bundle.signed_pre_key_public,
        )?;

        // Encrypt the plaintext
        let encrypted_message = session.encrypt(plaintext)?;

        // Store the session
        {
            let mut sessions = self.sessions.write().await;
            sessions.insert(address.clone(), session);
        }

        // Create initial message with X3DH data
        let initial = InitialMessage::new(
            self.identity_key.public_key(),
            x3dh_result.ephemeral_public_key,
            x3dh_result.used_pre_key_id,
            bundle.signed_pre_key_id,
            encrypted_message.serialize(),
        );

        tracing::info!(
            "Created initial encrypted message for {}",
            address.to_string()
        );

        Ok(initial.serialize())
    }

    /// Decrypt a message from a sender
    pub async fn decrypt(&self, address: &ProtocolAddress, ciphertext: &[u8]) -> Result<Vec<u8>> {
        let mut sessions = self.sessions.write().await;

        let session = sessions
            .get_mut(address)
            .ok_or_else(|| anyhow!("No session for {}", address.to_string()))?;

        let message = RatchetMessage::deserialize(ciphertext)?;
        let plaintext = session.decrypt(&message)?;

        tracing::debug!("Decrypted message from {}", address.to_string());

        Ok(plaintext)
    }

    /// Decrypt an initial message (first message in a conversation)
    pub async fn decrypt_initial(
        &mut self,
        address: &ProtocolAddress,
        ciphertext: &[u8],
    ) -> Result<Vec<u8>> {
        let initial = InitialMessage::deserialize(ciphertext)?;

        // Get our signed pre-key
        let signed_pre_key = self
            .signed_pre_key
            .as_ref()
            .ok_or_else(|| anyhow!("No signed pre-key"))?;

        if signed_pre_key.id != initial.signed_pre_key_id {
            return Err(anyhow!("Unknown signed pre-key ID"));
        }

        // Get our one-time pre-key if used
        let one_time_pre_key = if let Some(id) = initial.pre_key_id {
            self.pre_keys.get(&id).map(|pk| &pk.key_pair)
        } else {
            None
        };

        // Perform X3DH key agreement (Bob's side)
        let shared_secret = x3dh_respond(
            &self.identity_key,
            &signed_pre_key.key_pair,
            one_time_pre_key,
            &initial.identity_key,
            &initial.ephemeral_key,
        )?;

        // Initialize session (Bob's side)
        let our_ratchet_key = DhKeyPair::generate();
        let mut session = SessionState::initialize_bob(&shared_secret, our_ratchet_key);

        // Decrypt the initial message
        let ratchet_message = RatchetMessage::deserialize(&initial.encrypted_message)?;
        let plaintext = session.decrypt(&ratchet_message)?;

        // Store session and trust identity
        {
            let mut sessions = self.sessions.write().await;
            sessions.insert(address.clone(), session);
        }
        self.trusted_identities
            .insert(address.name.clone(), initial.identity_key.clone());

        // Remove used one-time pre-key
        if let Some(id) = initial.pre_key_id {
            self.pre_keys.remove(&id);
        }

        tracing::info!(
            "Processed initial message from {}, session established",
            address.to_string()
        );

        Ok(plaintext)
    }

    /// Check if we have a session with an address
    pub async fn has_session(&self, address: &ProtocolAddress) -> bool {
        let sessions = self.sessions.read().await;
        sessions.contains_key(address)
    }

    /// Get session state for serialization
    pub async fn get_session(&self, address: &ProtocolAddress) -> Option<Vec<u8>> {
        let sessions = self.sessions.read().await;
        sessions
            .get(address)
            .and_then(|s| s.serialize().ok())
    }

    /// Restore a session from serialized state
    pub async fn restore_session(&self, address: &ProtocolAddress, data: &[u8]) -> Result<()> {
        let session = SessionState::deserialize(data)?;
        let mut sessions = self.sessions.write().await;
        sessions.insert(address.clone(), session);
        Ok(())
    }

    /// Get safety number for verification
    pub fn get_safety_number(&self, local_id: &str, remote_id: &str) -> Result<String> {
        let remote_identity = self
            .trusted_identities
            .get(remote_id)
            .ok_or_else(|| anyhow!("No trusted identity for {}", remote_id))?;

        let fingerprint = calculate_fingerprint(
            &self.identity_key.public_key(),
            local_id,
            remote_identity,
            remote_id,
        );

        Ok(fingerprint)
    }

    /// Trust an identity key
    pub fn trust_identity(&mut self, name: &str, identity_key: IdentityPublicKey) {
        self.trusted_identities.insert(name.to_string(), identity_key);
    }

    /// Check if an identity is trusted
    pub fn is_identity_trusted(&self, name: &str, identity_key: &IdentityPublicKey) -> bool {
        if let Some(trusted) = self.trusted_identities.get(name) {
            trusted.as_bytes() == identity_key.as_bytes()
        } else {
            false
        }
    }

    /// Get number of available pre-keys
    pub fn pre_key_count(&self) -> usize {
        self.pre_keys.len()
    }

    /// Ensure we have enough pre-keys
    pub fn refill_pre_keys_if_needed(&mut self) -> Result<Option<Vec<(u32, Vec<u8>)>>> {
        if self.pre_keys.len() < 10 {
            let new_keys = self.generate_pre_keys(PRE_KEY_BATCH_SIZE)?;
            Ok(Some(new_keys))
        } else {
            Ok(None)
        }
    }
}

impl Default for SignalProtocol {
    fn default() -> Self {
        Self::new().expect("Failed to create protocol instance")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_session_establishment() {
        // Create Alice and Bob
        let mut alice = SignalProtocol::new().unwrap();
        let mut bob = SignalProtocol::new().unwrap();

        // Bob generates keys
        bob.generate_pre_keys(10).unwrap();
        bob.generate_signed_pre_key(1).unwrap();

        // Bob creates pre-key bundle
        let bob_bundle = bob.create_pre_key_bundle(1).unwrap();

        // Alice establishes session with Bob
        let bob_address = ProtocolAddress::new("bob", 1);
        alice
            .process_pre_key_bundle(&bob_address, &bob_bundle)
            .await
            .unwrap();

        // Alice sends a message
        let plaintext = b"Hello, Bob!";
        let ciphertext = alice
            .encrypt_initial(&bob_address, &bob_bundle, plaintext)
            .await
            .unwrap();

        // Bob decrypts the message
        let alice_address = ProtocolAddress::new("alice", 1);
        let decrypted = bob.decrypt_initial(&alice_address, &ciphertext).await.unwrap();

        assert_eq!(plaintext.as_slice(), decrypted.as_slice());
    }

    #[tokio::test]
    async fn test_bidirectional_messaging() {
        // Create Alice and Bob
        let mut alice = SignalProtocol::new().unwrap();
        let mut bob = SignalProtocol::new().unwrap();

        // Generate keys for both
        alice.generate_pre_keys(10).unwrap();
        alice.generate_signed_pre_key(1).unwrap();
        bob.generate_pre_keys(10).unwrap();
        bob.generate_signed_pre_key(1).unwrap();

        // Exchange bundles and establish sessions
        let alice_bundle = alice.create_pre_key_bundle(1).unwrap();
        let bob_bundle = bob.create_pre_key_bundle(1).unwrap();

        let alice_address = ProtocolAddress::new("alice", 1);
        let bob_address = ProtocolAddress::new("bob", 1);

        // Alice initiates with Bob
        let msg1 = alice
            .encrypt_initial(&bob_address, &bob_bundle, b"Hello Bob!")
            .await
            .unwrap();
        let decrypted1 = bob.decrypt_initial(&alice_address, &msg1).await.unwrap();
        assert_eq!(b"Hello Bob!", decrypted1.as_slice());

        // Bob responds to Alice
        let msg2 = bob
            .encrypt_initial(&alice_address, &alice_bundle, b"Hi Alice!")
            .await
            .unwrap();
        let decrypted2 = alice.decrypt_initial(&bob_address, &msg2).await.unwrap();
        assert_eq!(b"Hi Alice!", decrypted2.as_slice());

        // Continue conversation with established sessions
        let msg3 = alice.encrypt(&bob_address, b"How are you?").await.unwrap();
        let decrypted3 = bob.decrypt(&alice_address, &msg3).await.unwrap();
        assert_eq!(b"How are you?", decrypted3.as_slice());
    }
}
