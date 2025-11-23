//! Double Ratchet Algorithm implementation
//!
//! Implements the Signal Protocol's Double Ratchet algorithm for
//! forward secrecy and post-compromise security in end-to-end encryption.
//!
//! Reference: https://signal.org/docs/specifications/doubleratchet/

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use x25519_dalek::PublicKey as X25519PublicKey;
use zeroize::{Zeroize, ZeroizeOnDrop};

use super::crypto::{DhKeyPair, MessageKeys, SignalCipher, SignalHkdf, NONCE_SIZE};

/// Maximum number of skipped message keys to store
const MAX_SKIP: u32 = 1000;

/// Session state for the Double Ratchet
#[derive(Clone, Serialize, Deserialize)]
pub struct SessionState {
    /// Our current DH ratchet key pair
    #[serde(with = "dh_keypair_serde")]
    dh_self: DhKeyPair,
    /// Their current DH ratchet public key
    #[serde(with = "x25519_pubkey_serde")]
    dh_remote: Option<X25519PublicKey>,
    /// Root key for deriving chain keys
    #[serde(with = "array32_serde")]
    root_key: [u8; 32],
    /// Sending chain key
    #[serde(with = "option_array32_serde")]
    sending_chain_key: Option<[u8; 32]>,
    /// Receiving chain key
    #[serde(with = "option_array32_serde")]
    receiving_chain_key: Option<[u8; 32]>,
    /// Sending message counter
    sending_counter: u32,
    /// Receiving message counter
    receiving_counter: u32,
    /// Previous sending chain counter (for header)
    previous_counter: u32,
    /// Skipped message keys (for out-of-order delivery)
    skipped_keys: HashMap<(Vec<u8>, u32), SkippedKey>,
}

/// Skipped message key for out-of-order message handling
#[derive(Clone, Serialize, Deserialize, Zeroize, ZeroizeOnDrop)]
struct SkippedKey {
    #[serde(with = "array32_serde")]
    cipher_key: [u8; 32],
    #[serde(with = "array32_serde")]
    mac_key: [u8; 32],
    #[serde(with = "array16_serde")]
    iv: [u8; 16],
    timestamp: i64,
}

impl SessionState {
    /// Initialize a new session as the initiator (Alice)
    /// Called after X3DH key agreement
    pub fn initialize_alice(
        shared_secret: &[u8; 32],
        our_ratchet_key: DhKeyPair,
        their_ratchet_key: &X25519PublicKey,
    ) -> Result<Self> {
        // Initial root key from X3DH shared secret
        let root_key = *shared_secret;

        // Perform initial DH ratchet step
        let dh_output = our_ratchet_key.dh_agreement(their_ratchet_key);
        let (new_root_key, sending_chain_key) = SignalHkdf::derive_root_key(&root_key, &dh_output)?;

        Ok(Self {
            dh_self: our_ratchet_key,
            dh_remote: Some(*their_ratchet_key),
            root_key: new_root_key,
            sending_chain_key: Some(sending_chain_key),
            receiving_chain_key: None,
            sending_counter: 0,
            receiving_counter: 0,
            previous_counter: 0,
            skipped_keys: HashMap::new(),
        })
    }

    /// Initialize a new session as the responder (Bob)
    /// Called after receiving initial message with X3DH data
    pub fn initialize_bob(shared_secret: &[u8; 32], our_ratchet_key: DhKeyPair) -> Self {
        Self {
            dh_self: our_ratchet_key,
            dh_remote: None,
            root_key: *shared_secret,
            sending_chain_key: None,
            receiving_chain_key: None,
            sending_counter: 0,
            receiving_counter: 0,
            previous_counter: 0,
            skipped_keys: HashMap::new(),
        }
    }

    /// Encrypt a message
    pub fn encrypt(&mut self, plaintext: &[u8]) -> Result<RatchetMessage> {
        // Ensure we have a sending chain
        if self.sending_chain_key.is_none() {
            return Err(anyhow!("No sending chain key available"));
        }

        // Derive message keys
        let chain_key = self.sending_chain_key.as_ref().unwrap();
        let message_keys = SignalHkdf::derive_message_keys(chain_key)?;

        // Update chain key
        self.sending_chain_key = Some(message_keys.next_chain_key);

        // Generate nonce from IV
        let mut nonce = [0u8; NONCE_SIZE];
        nonce.copy_from_slice(&message_keys.iv[..NONCE_SIZE]);

        // Encrypt the plaintext
        let ciphertext = SignalCipher::encrypt(&message_keys.cipher_key, &nonce, plaintext)?;

        // Create message header
        let header = MessageHeader {
            dh_ratchet_key: *self.dh_self.public_key(),
            previous_counter: self.previous_counter,
            message_counter: self.sending_counter,
        };

        // Increment counter
        self.sending_counter += 1;

        Ok(RatchetMessage {
            header,
            ciphertext,
        })
    }

    /// Decrypt a message
    pub fn decrypt(&mut self, message: &RatchetMessage) -> Result<Vec<u8>> {
        // Check if this is a skipped message
        let key_id = (
            message.header.dh_ratchet_key.as_bytes().to_vec(),
            message.header.message_counter,
        );

        if let Some(skipped) = self.skipped_keys.remove(&key_id) {
            let mut nonce = [0u8; NONCE_SIZE];
            nonce.copy_from_slice(&skipped.iv[..NONCE_SIZE]);
            return SignalCipher::decrypt(&skipped.cipher_key, &nonce, &message.ciphertext);
        }

        // Check if we need to perform a DH ratchet step
        let their_key = message.header.dh_ratchet_key;
        if self.dh_remote.is_none() || *self.dh_remote.as_ref().unwrap() != their_key {
            // Skip any remaining messages in current receiving chain
            if self.receiving_chain_key.is_some() {
                self.skip_message_keys(message.header.previous_counter)?;
            }

            // Perform DH ratchet
            self.dh_ratchet(&their_key)?;
        }

        // Skip any messages before this one in the current chain
        self.skip_message_keys(message.header.message_counter)?;

        // Derive message keys
        let chain_key = self.receiving_chain_key.as_ref().ok_or_else(|| {
            anyhow!("No receiving chain key available")
        })?;
        let message_keys = SignalHkdf::derive_message_keys(chain_key)?;

        // Update chain key
        self.receiving_chain_key = Some(message_keys.next_chain_key);
        self.receiving_counter = message.header.message_counter + 1;

        // Generate nonce from IV
        let mut nonce = [0u8; NONCE_SIZE];
        nonce.copy_from_slice(&message_keys.iv[..NONCE_SIZE]);

        // Decrypt the ciphertext
        SignalCipher::decrypt(&message_keys.cipher_key, &nonce, &message.ciphertext)
    }

    /// Perform a DH ratchet step
    fn dh_ratchet(&mut self, their_key: &X25519PublicKey) -> Result<()> {
        self.previous_counter = self.sending_counter;
        self.sending_counter = 0;
        self.receiving_counter = 0;

        self.dh_remote = Some(*their_key);

        // Derive receiving chain key from DH
        let dh_output = self.dh_self.dh_agreement(their_key);
        let (new_root_key, receiving_chain_key) =
            SignalHkdf::derive_root_key(&self.root_key, &dh_output)?;
        self.root_key = new_root_key;
        self.receiving_chain_key = Some(receiving_chain_key);

        // Generate new DH key pair
        self.dh_self = DhKeyPair::generate();

        // Derive sending chain key from new DH
        let dh_output = self.dh_self.dh_agreement(their_key);
        let (new_root_key, sending_chain_key) =
            SignalHkdf::derive_root_key(&self.root_key, &dh_output)?;
        self.root_key = new_root_key;
        self.sending_chain_key = Some(sending_chain_key);

        Ok(())
    }

    /// Skip and store message keys for out-of-order delivery
    fn skip_message_keys(&mut self, until: u32) -> Result<()> {
        if self.receiving_chain_key.is_none() {
            return Ok(());
        }

        let current = self.receiving_counter;
        if until < current {
            return Ok(());
        }

        if until - current > MAX_SKIP {
            return Err(anyhow!("Too many skipped messages"));
        }

        let dh_key = self.dh_remote.as_ref().map(|k| k.as_bytes().to_vec());
        if dh_key.is_none() {
            return Ok(());
        }
        let dh_key = dh_key.unwrap();

        for i in current..until {
            let chain_key = self.receiving_chain_key.as_ref().unwrap();
            let message_keys = SignalHkdf::derive_message_keys(chain_key)?;

            self.skipped_keys.insert(
                (dh_key.clone(), i),
                SkippedKey {
                    cipher_key: message_keys.cipher_key,
                    mac_key: message_keys.mac_key,
                    iv: message_keys.iv,
                    timestamp: chrono::Utc::now().timestamp(),
                },
            );

            self.receiving_chain_key = Some(message_keys.next_chain_key);
        }

        Ok(())
    }

    /// Get our current ratchet public key
    pub fn our_ratchet_key(&self) -> &X25519PublicKey {
        self.dh_self.public_key()
    }

    /// Serialize the session state for storage
    pub fn serialize(&self) -> Result<Vec<u8>> {
        serde_json::to_vec(self).map_err(|e| anyhow!("Serialization failed: {}", e))
    }

    /// Deserialize session state from storage
    pub fn deserialize(data: &[u8]) -> Result<Self> {
        serde_json::from_slice(data).map_err(|e| anyhow!("Deserialization failed: {}", e))
    }

    /// Clean up old skipped keys
    pub fn cleanup_skipped_keys(&mut self, max_age_seconds: i64) {
        let now = chrono::Utc::now().timestamp();
        self.skipped_keys.retain(|_, key| now - key.timestamp < max_age_seconds);
    }
}

/// Message header containing ratchet public key and counters
#[derive(Clone, Debug)]
pub struct MessageHeader {
    /// Current DH ratchet public key
    pub dh_ratchet_key: X25519PublicKey,
    /// Previous sending chain message counter
    pub previous_counter: u32,
    /// Current message counter
    pub message_counter: u32,
}

impl MessageHeader {
    /// Serialize the header
    pub fn serialize(&self) -> Vec<u8> {
        let mut data = Vec::with_capacity(40);
        data.extend_from_slice(self.dh_ratchet_key.as_bytes());
        data.extend_from_slice(&self.previous_counter.to_be_bytes());
        data.extend_from_slice(&self.message_counter.to_be_bytes());
        data
    }

    /// Deserialize the header
    pub fn deserialize(data: &[u8]) -> Result<Self> {
        if data.len() < 40 {
            return Err(anyhow!("Header data too short"));
        }
        let key_bytes: [u8; 32] = data[0..32].try_into()?;
        let dh_ratchet_key = X25519PublicKey::from(key_bytes);
        let previous_counter = u32::from_be_bytes(data[32..36].try_into()?);
        let message_counter = u32::from_be_bytes(data[36..40].try_into()?);

        Ok(Self {
            dh_ratchet_key,
            previous_counter,
            message_counter,
        })
    }
}

/// Complete ratchet message with header and ciphertext
#[derive(Clone, Debug)]
pub struct RatchetMessage {
    pub header: MessageHeader,
    pub ciphertext: Vec<u8>,
}

impl RatchetMessage {
    /// Serialize the complete message
    pub fn serialize(&self) -> Vec<u8> {
        let header = self.header.serialize();
        let mut data = Vec::with_capacity(4 + header.len() + self.ciphertext.len());
        data.extend_from_slice(&(header.len() as u32).to_be_bytes());
        data.extend_from_slice(&header);
        data.extend_from_slice(&self.ciphertext);
        data
    }

    /// Deserialize a complete message
    pub fn deserialize(data: &[u8]) -> Result<Self> {
        if data.len() < 4 {
            return Err(anyhow!("Message data too short"));
        }
        let header_len = u32::from_be_bytes(data[0..4].try_into()?) as usize;
        if data.len() < 4 + header_len {
            return Err(anyhow!("Message data too short for header"));
        }
        let header = MessageHeader::deserialize(&data[4..4 + header_len])?;
        let ciphertext = data[4 + header_len..].to_vec();

        Ok(Self { header, ciphertext })
    }
}

// Serde helpers for DhKeyPair
mod dh_keypair_serde {
    use super::*;
    use serde::{Deserializer, Serializer};

    pub fn serialize<S>(key: &DhKeyPair, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let bytes = key.private_key_bytes();
        serializer.serialize_bytes(&bytes)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<DhKeyPair, D::Error>
    where
        D: Deserializer<'de>,
    {
        let bytes: Vec<u8> = Deserialize::deserialize(deserializer)?;
        if bytes.len() != 32 {
            return Err(serde::de::Error::custom("Invalid key length"));
        }
        let mut arr = [0u8; 32];
        arr.copy_from_slice(&bytes);
        Ok(DhKeyPair::from_private_key(arr))
    }
}

// Serde helpers for X25519PublicKey
mod x25519_pubkey_serde {
    use super::*;
    use serde::{Deserializer, Serializer};

    pub fn serialize<S>(key: &Option<X25519PublicKey>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match key {
            Some(k) => serializer.serialize_some(&k.as_bytes().to_vec()),
            None => serializer.serialize_none(),
        }
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<X25519PublicKey>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let opt: Option<Vec<u8>> = Deserialize::deserialize(deserializer)?;
        match opt {
            Some(bytes) if bytes.len() == 32 => {
                let mut arr = [0u8; 32];
                arr.copy_from_slice(&bytes);
                Ok(Some(X25519PublicKey::from(arr)))
            }
            Some(_) => Err(serde::de::Error::custom("Invalid key length")),
            None => Ok(None),
        }
    }
}

// Serde helpers for fixed-size arrays
mod array32_serde {
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(arr: &[u8; 32], serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_bytes(arr)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<[u8; 32], D::Error>
    where
        D: Deserializer<'de>,
    {
        let bytes: Vec<u8> = Deserialize::deserialize(deserializer)?;
        if bytes.len() != 32 {
            return Err(serde::de::Error::custom("Invalid array length"));
        }
        let mut arr = [0u8; 32];
        arr.copy_from_slice(&bytes);
        Ok(arr)
    }
}

mod option_array32_serde {
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(arr: &Option<[u8; 32]>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match arr {
            Some(a) => serializer.serialize_some(&a.to_vec()),
            None => serializer.serialize_none(),
        }
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<[u8; 32]>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let opt: Option<Vec<u8>> = Deserialize::deserialize(deserializer)?;
        match opt {
            Some(bytes) if bytes.len() == 32 => {
                let mut arr = [0u8; 32];
                arr.copy_from_slice(&bytes);
                Ok(Some(arr))
            }
            Some(_) => Err(serde::de::Error::custom("Invalid array length")),
            None => Ok(None),
        }
    }
}

mod array16_serde {
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(arr: &[u8; 16], serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_bytes(arr)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<[u8; 16], D::Error>
    where
        D: Deserializer<'de>,
    {
        let bytes: Vec<u8> = Deserialize::deserialize(deserializer)?;
        if bytes.len() != 16 {
            return Err(serde::de::Error::custom("Invalid array length"));
        }
        let mut arr = [0u8; 16];
        arr.copy_from_slice(&bytes);
        Ok(arr)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ratchet_encrypt_decrypt() {
        // Simulate X3DH shared secret
        let shared_secret = [0x42u8; 32];

        // Alice initializes with Bob's ratchet key
        let bob_ratchet = DhKeyPair::generate();
        let alice_ratchet = DhKeyPair::generate();

        let mut alice_session = SessionState::initialize_alice(
            &shared_secret,
            alice_ratchet.clone(),
            bob_ratchet.public_key(),
        )
        .unwrap();

        // Bob initializes with the shared secret
        let mut bob_session = SessionState::initialize_bob(&shared_secret, bob_ratchet);

        // Alice sends a message
        let plaintext = b"Hello, Bob!";
        let message = alice_session.encrypt(plaintext).unwrap();

        // Bob receives and decrypts
        let decrypted = bob_session.decrypt(&message).unwrap();
        assert_eq!(plaintext.as_slice(), decrypted.as_slice());
    }

    #[test]
    fn test_session_serialization() {
        let shared_secret = [0x42u8; 32];
        let bob_ratchet = DhKeyPair::generate();
        let alice_ratchet = DhKeyPair::generate();

        let session = SessionState::initialize_alice(
            &shared_secret,
            alice_ratchet,
            bob_ratchet.public_key(),
        )
        .unwrap();

        let serialized = session.serialize().unwrap();
        let deserialized = SessionState::deserialize(&serialized).unwrap();

        assert_eq!(session.sending_counter, deserialized.sending_counter);
    }
}
