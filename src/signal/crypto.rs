//! Cryptographic primitives for Signal Protocol
//!
//! This module provides the core cryptographic operations needed for the Signal Protocol:
//! - X25519 Diffie-Hellman key exchange
//! - Ed25519 digital signatures
//! - HKDF key derivation
//! - AES-256-GCM authenticated encryption

use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use hkdf::Hkdf;
use hmac::{Hmac, Mac};
use rand::rngs::OsRng;
use sha2::{Digest, Sha256, Sha512};
use x25519_dalek::{EphemeralSecret, PublicKey as X25519PublicKey, StaticSecret};
use zeroize::{Zeroize, ZeroizeOnDrop};

use anyhow::{anyhow, Result};

/// Size of symmetric keys in bytes (256 bits)
pub const KEY_SIZE: usize = 32;
/// Size of AES-GCM nonce in bytes (96 bits)
pub const NONCE_SIZE: usize = 12;
/// Size of Ed25519 signature in bytes
pub const SIGNATURE_SIZE: usize = 64;

/// Signal Protocol info string for HKDF
const SIGNAL_HKDF_INFO: &[u8] = b"Signal Protocol";

/// Identity key pair (Ed25519)
#[derive(Clone, ZeroizeOnDrop)]
pub struct IdentityKeyPair {
    #[zeroize(skip)]
    signing_key: SigningKey,
}

impl IdentityKeyPair {
    /// Generate a new identity key pair
    pub fn generate() -> Self {
        let signing_key = SigningKey::generate(&mut OsRng);
        Self { signing_key }
    }

    /// Create from existing private key bytes
    pub fn from_private_key(bytes: &[u8; 32]) -> Result<Self> {
        let signing_key = SigningKey::from_bytes(bytes);
        Ok(Self { signing_key })
    }

    /// Get the public identity key
    pub fn public_key(&self) -> IdentityPublicKey {
        IdentityPublicKey {
            verifying_key: self.signing_key.verifying_key(),
        }
    }

    /// Get the private key bytes (for secure storage)
    pub fn private_key_bytes(&self) -> [u8; 32] {
        self.signing_key.to_bytes()
    }

    /// Sign a message
    pub fn sign(&self, message: &[u8]) -> [u8; SIGNATURE_SIZE] {
        let signature = self.signing_key.sign(message);
        signature.to_bytes()
    }

    /// Get the public key as X25519 for Diffie-Hellman
    /// (Signal derives DH key from Ed25519 key)
    pub fn dh_public_key(&self) -> X25519PublicKey {
        // Convert Ed25519 private key to X25519 via SHA-512 hash
        let hash = Sha512::digest(&self.signing_key.to_bytes());
        let mut x25519_private = [0u8; 32];
        x25519_private.copy_from_slice(&hash[..32]);
        // Clamp the scalar
        x25519_private[0] &= 248;
        x25519_private[31] &= 127;
        x25519_private[31] |= 64;

        let secret = StaticSecret::from(x25519_private);
        X25519PublicKey::from(&secret)
    }

    /// Perform DH key agreement with a peer's public key
    pub fn dh_agreement(&self, peer_public: &X25519PublicKey) -> [u8; 32] {
        let hash = Sha512::digest(&self.signing_key.to_bytes());
        let mut x25519_private = [0u8; 32];
        x25519_private.copy_from_slice(&hash[..32]);
        x25519_private[0] &= 248;
        x25519_private[31] &= 127;
        x25519_private[31] |= 64;

        let secret = StaticSecret::from(x25519_private);
        let shared_secret = secret.diffie_hellman(peer_public);
        *shared_secret.as_bytes()
    }
}

impl std::fmt::Debug for IdentityKeyPair {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("IdentityKeyPair")
            .field("public_key", &hex::encode(self.public_key().as_bytes()))
            .finish()
    }
}

/// Public identity key
#[derive(Clone, Debug)]
pub struct IdentityPublicKey {
    verifying_key: VerifyingKey,
}

impl IdentityPublicKey {
    /// Create from raw bytes
    pub fn from_bytes(bytes: &[u8; 32]) -> Result<Self> {
        let verifying_key = VerifyingKey::from_bytes(bytes)
            .map_err(|e| anyhow!("Invalid public key: {}", e))?;
        Ok(Self { verifying_key })
    }

    /// Get raw bytes
    pub fn as_bytes(&self) -> [u8; 32] {
        self.verifying_key.to_bytes()
    }

    /// Verify a signature
    pub fn verify(&self, message: &[u8], signature: &[u8; SIGNATURE_SIZE]) -> Result<()> {
        let sig = Signature::from_bytes(signature);
        self.verifying_key
            .verify(message, &sig)
            .map_err(|e| anyhow!("Signature verification failed: {}", e))
    }
}

/// X25519 key pair for Diffie-Hellman key exchange
#[derive(ZeroizeOnDrop)]
pub struct DhKeyPair {
    #[zeroize(skip)]
    secret: StaticSecret,
    #[zeroize(skip)]
    public: X25519PublicKey,
}

impl DhKeyPair {
    /// Generate a new DH key pair
    pub fn generate() -> Self {
        let secret = StaticSecret::random_from_rng(OsRng);
        let public = X25519PublicKey::from(&secret);
        Self { secret, public }
    }

    /// Create from existing private key bytes
    pub fn from_private_key(bytes: [u8; 32]) -> Self {
        let secret = StaticSecret::from(bytes);
        let public = X25519PublicKey::from(&secret);
        Self { secret, public }
    }

    /// Get the public key
    pub fn public_key(&self) -> &X25519PublicKey {
        &self.public
    }

    /// Get the private key bytes (for secure storage)
    pub fn private_key_bytes(&self) -> [u8; 32] {
        self.secret.to_bytes()
    }

    /// Perform DH key agreement
    pub fn dh_agreement(&self, peer_public: &X25519PublicKey) -> [u8; 32] {
        let shared_secret = self.secret.diffie_hellman(peer_public);
        *shared_secret.as_bytes()
    }
}

impl Clone for DhKeyPair {
    fn clone(&self) -> Self {
        Self::from_private_key(self.secret.to_bytes())
    }
}

impl std::fmt::Debug for DhKeyPair {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DhKeyPair")
            .field("public_key", &hex::encode(self.public.as_bytes()))
            .finish()
    }
}

/// Pre-key for Signal Protocol key exchange
#[derive(Clone)]
pub struct PreKey {
    pub id: u32,
    pub key_pair: DhKeyPair,
}

impl PreKey {
    /// Generate a new pre-key with the given ID
    pub fn generate(id: u32) -> Self {
        Self {
            id,
            key_pair: DhKeyPair::generate(),
        }
    }

    /// Create from stored data
    pub fn from_stored(id: u32, private_key: [u8; 32]) -> Self {
        Self {
            id,
            key_pair: DhKeyPair::from_private_key(private_key),
        }
    }

    /// Serialize for storage
    pub fn serialize(&self) -> Vec<u8> {
        let mut data = Vec::with_capacity(4 + 32 + 32);
        data.extend_from_slice(&self.id.to_be_bytes());
        data.extend_from_slice(&self.key_pair.private_key_bytes());
        data.extend_from_slice(self.key_pair.public_key().as_bytes());
        data
    }

    /// Deserialize from stored data
    pub fn deserialize(data: &[u8]) -> Result<Self> {
        if data.len() < 4 + 32 {
            return Err(anyhow!("Pre-key data too short"));
        }
        let id = u32::from_be_bytes(data[0..4].try_into()?);
        let private_key: [u8; 32] = data[4..36].try_into()?;
        Ok(Self::from_stored(id, private_key))
    }
}

/// Signed pre-key with Ed25519 signature from identity key
pub struct SignedPreKey {
    pub id: u32,
    pub key_pair: DhKeyPair,
    pub signature: [u8; SIGNATURE_SIZE],
    pub timestamp: i64,
}

impl SignedPreKey {
    /// Generate a new signed pre-key
    pub fn generate(id: u32, identity_key: &IdentityKeyPair) -> Self {
        let key_pair = DhKeyPair::generate();
        let timestamp = chrono::Utc::now().timestamp();

        // Sign the public key
        let signature = identity_key.sign(key_pair.public_key().as_bytes());

        Self {
            id,
            key_pair,
            signature,
            timestamp,
        }
    }

    /// Serialize for storage
    pub fn serialize(&self) -> Vec<u8> {
        let mut data = Vec::with_capacity(4 + 32 + 32 + 64 + 8);
        data.extend_from_slice(&self.id.to_be_bytes());
        data.extend_from_slice(&self.key_pair.private_key_bytes());
        data.extend_from_slice(self.key_pair.public_key().as_bytes());
        data.extend_from_slice(&self.signature);
        data.extend_from_slice(&self.timestamp.to_be_bytes());
        data
    }

    /// Deserialize from stored data
    pub fn deserialize(data: &[u8]) -> Result<Self> {
        if data.len() < 4 + 32 + 32 + 64 + 8 {
            return Err(anyhow!("Signed pre-key data too short"));
        }
        let id = u32::from_be_bytes(data[0..4].try_into()?);
        let private_key: [u8; 32] = data[4..36].try_into()?;
        let signature: [u8; 64] = data[68..132].try_into()?;
        let timestamp = i64::from_be_bytes(data[132..140].try_into()?);

        Ok(Self {
            id,
            key_pair: DhKeyPair::from_private_key(private_key),
            signature,
            timestamp,
        })
    }
}

/// Pre-key bundle for initiating a session
#[derive(Clone, Debug)]
pub struct PreKeyBundle {
    pub registration_id: u32,
    pub device_id: u32,
    pub pre_key_id: Option<u32>,
    pub pre_key_public: Option<X25519PublicKey>,
    pub signed_pre_key_id: u32,
    pub signed_pre_key_public: X25519PublicKey,
    pub signed_pre_key_signature: [u8; SIGNATURE_SIZE],
    pub identity_key: IdentityPublicKey,
}

impl PreKeyBundle {
    /// Verify the signed pre-key signature
    pub fn verify(&self) -> Result<()> {
        self.identity_key
            .verify(self.signed_pre_key_public.as_bytes(), &self.signed_pre_key_signature)
    }

    /// Serialize for network transmission
    pub fn serialize(&self) -> Vec<u8> {
        let mut data = Vec::with_capacity(256);
        data.extend_from_slice(&self.registration_id.to_be_bytes());
        data.extend_from_slice(&self.device_id.to_be_bytes());

        // Pre-key (optional)
        if let (Some(id), Some(key)) = (self.pre_key_id, &self.pre_key_public) {
            data.push(1); // has pre-key
            data.extend_from_slice(&id.to_be_bytes());
            data.extend_from_slice(key.as_bytes());
        } else {
            data.push(0); // no pre-key
        }

        // Signed pre-key
        data.extend_from_slice(&self.signed_pre_key_id.to_be_bytes());
        data.extend_from_slice(self.signed_pre_key_public.as_bytes());
        data.extend_from_slice(&self.signed_pre_key_signature);

        // Identity key
        data.extend_from_slice(&self.identity_key.as_bytes());

        data
    }

    /// Deserialize from network data
    pub fn deserialize(data: &[u8]) -> Result<Self> {
        if data.len() < 9 {
            return Err(anyhow!("Pre-key bundle data too short"));
        }

        let registration_id = u32::from_be_bytes(data[0..4].try_into()?);
        let device_id = u32::from_be_bytes(data[4..8].try_into()?);

        let has_pre_key = data[8] == 1;
        let mut offset = 9;

        let (pre_key_id, pre_key_public) = if has_pre_key {
            let id = u32::from_be_bytes(data[offset..offset + 4].try_into()?);
            offset += 4;
            let key_bytes: [u8; 32] = data[offset..offset + 32].try_into()?;
            offset += 32;
            (Some(id), Some(X25519PublicKey::from(key_bytes)))
        } else {
            (None, None)
        };

        let signed_pre_key_id = u32::from_be_bytes(data[offset..offset + 4].try_into()?);
        offset += 4;
        let spk_bytes: [u8; 32] = data[offset..offset + 32].try_into()?;
        let signed_pre_key_public = X25519PublicKey::from(spk_bytes);
        offset += 32;
        let signed_pre_key_signature: [u8; 64] = data[offset..offset + 64].try_into()?;
        offset += 64;
        let ik_bytes: [u8; 32] = data[offset..offset + 32].try_into()?;
        let identity_key = IdentityPublicKey::from_bytes(&ik_bytes)?;

        Ok(Self {
            registration_id,
            device_id,
            pre_key_id,
            pre_key_public,
            signed_pre_key_id,
            signed_pre_key_public,
            signed_pre_key_signature,
            identity_key,
        })
    }
}

/// HKDF-based key derivation
pub struct SignalHkdf;

impl SignalHkdf {
    /// Derive keys using HKDF-SHA256
    pub fn derive_secrets(
        input_key_material: &[u8],
        salt: &[u8],
        info: &[u8],
        output_length: usize,
    ) -> Result<Vec<u8>> {
        let hkdf = Hkdf::<Sha256>::new(Some(salt), input_key_material);
        let mut output = vec![0u8; output_length];
        hkdf.expand(info, &mut output)
            .map_err(|_| anyhow!("HKDF expansion failed"))?;
        Ok(output)
    }

    /// Derive root key and chain key from shared secret
    pub fn derive_root_key(root_key: &[u8; 32], dh_output: &[u8; 32]) -> Result<([u8; 32], [u8; 32])> {
        let output = Self::derive_secrets(dh_output, root_key, b"WhisperRatchet", 64)?;
        let mut new_root_key = [0u8; 32];
        let mut chain_key = [0u8; 32];
        new_root_key.copy_from_slice(&output[..32]);
        chain_key.copy_from_slice(&output[32..]);
        Ok((new_root_key, chain_key))
    }

    /// Derive message keys from chain key
    pub fn derive_message_keys(chain_key: &[u8; 32]) -> Result<MessageKeys> {
        // Derive next chain key
        let mut mac = Hmac::<Sha256>::new_from_slice(chain_key)
            .map_err(|_| anyhow!("Invalid chain key length"))?;
        mac.update(&[0x02]);
        let next_chain_key_result = mac.finalize().into_bytes();
        let mut next_chain_key = [0u8; 32];
        next_chain_key.copy_from_slice(&next_chain_key_result);

        // Derive message key material
        let mut mac = Hmac::<Sha256>::new_from_slice(chain_key)
            .map_err(|_| anyhow!("Invalid chain key length"))?;
        mac.update(&[0x01]);
        let message_key_material = mac.finalize().into_bytes();

        // Expand message key material into cipher key, mac key, and IV
        let expanded = Self::derive_secrets(
            &message_key_material,
            b"",
            b"WhisperMessageKeys",
            80,
        )?;

        let mut cipher_key = [0u8; 32];
        let mut mac_key = [0u8; 32];
        let mut iv = [0u8; 16];

        cipher_key.copy_from_slice(&expanded[..32]);
        mac_key.copy_from_slice(&expanded[32..64]);
        iv.copy_from_slice(&expanded[64..80]);

        Ok(MessageKeys {
            cipher_key,
            mac_key,
            iv,
            next_chain_key,
        })
    }
}

/// Message encryption keys derived from chain key
#[derive(Clone, Zeroize, ZeroizeOnDrop)]
pub struct MessageKeys {
    pub cipher_key: [u8; 32],
    pub mac_key: [u8; 32],
    pub iv: [u8; 16],
    pub next_chain_key: [u8; 32],
}

/// AES-256-GCM authenticated encryption
pub struct SignalCipher;

impl SignalCipher {
    /// Encrypt plaintext with AES-256-GCM
    pub fn encrypt(key: &[u8; 32], nonce: &[u8; NONCE_SIZE], plaintext: &[u8]) -> Result<Vec<u8>> {
        let cipher = Aes256Gcm::new_from_slice(key)
            .map_err(|_| anyhow!("Invalid key length"))?;
        let nonce = Nonce::from_slice(nonce);

        cipher
            .encrypt(nonce, plaintext)
            .map_err(|_| anyhow!("Encryption failed"))
    }

    /// Decrypt ciphertext with AES-256-GCM
    pub fn decrypt(key: &[u8; 32], nonce: &[u8; NONCE_SIZE], ciphertext: &[u8]) -> Result<Vec<u8>> {
        let cipher = Aes256Gcm::new_from_slice(key)
            .map_err(|_| anyhow!("Invalid key length"))?;
        let nonce = Nonce::from_slice(nonce);

        cipher
            .decrypt(nonce, ciphertext)
            .map_err(|_| anyhow!("Decryption failed"))
    }

    /// Generate a random nonce
    pub fn generate_nonce() -> [u8; NONCE_SIZE] {
        let mut nonce = [0u8; NONCE_SIZE];
        rand::RngCore::fill_bytes(&mut OsRng, &mut nonce);
        nonce
    }
}

/// Calculate fingerprint/safety number for identity verification
pub fn calculate_fingerprint(
    local_identity: &IdentityPublicKey,
    local_identifier: &str,
    remote_identity: &IdentityPublicKey,
    remote_identifier: &str,
) -> String {
    // Sort identifiers to ensure same result regardless of who initiates
    let (first_id, first_key, second_id, second_key) = if local_identifier < remote_identifier {
        (
            local_identifier,
            local_identity.as_bytes(),
            remote_identifier,
            remote_identity.as_bytes(),
        )
    } else {
        (
            remote_identifier,
            remote_identity.as_bytes(),
            local_identifier,
            local_identity.as_bytes(),
        )
    };

    // Hash the combined data
    let mut hasher = Sha256::new();
    hasher.update(first_id.as_bytes());
    hasher.update(&first_key);
    hasher.update(second_id.as_bytes());
    hasher.update(&second_key);

    // Iterate hash for additional security
    let mut hash = hasher.finalize();
    for _ in 0..5199 {
        let mut hasher = Sha256::new();
        hasher.update(&hash);
        hasher.update(&first_key);
        hasher.update(&second_key);
        hash = hasher.finalize();
    }

    // Convert to numeric fingerprint (12 groups of 5 digits)
    let mut fingerprint = String::with_capacity(70);
    for i in 0..12 {
        let offset = i * 5 % 30;
        let chunk = [
            hash[offset],
            hash[(offset + 1) % 32],
            hash[(offset + 2) % 32],
            hash[(offset + 3) % 32],
            hash[(offset + 4) % 32],
        ];
        let num = u64::from_be_bytes([0, 0, 0, chunk[0], chunk[1], chunk[2], chunk[3], chunk[4]]);
        if i > 0 {
            fingerprint.push(' ');
        }
        fingerprint.push_str(&format!("{:05}", num % 100000));
    }
    fingerprint
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_identity_key_generation() {
        let key_pair = IdentityKeyPair::generate();
        let public_key = key_pair.public_key();

        // Test signing and verification
        let message = b"test message";
        let signature = key_pair.sign(message);
        assert!(public_key.verify(message, &signature).is_ok());
    }

    #[test]
    fn test_dh_key_exchange() {
        let alice = DhKeyPair::generate();
        let bob = DhKeyPair::generate();

        let alice_shared = alice.dh_agreement(bob.public_key());
        let bob_shared = bob.dh_agreement(alice.public_key());

        assert_eq!(alice_shared, bob_shared);
    }

    #[test]
    fn test_encryption_decryption() {
        let key = [0u8; 32];
        let nonce = [0u8; NONCE_SIZE];
        let plaintext = b"Hello, Signal!";

        let ciphertext = SignalCipher::encrypt(&key, &nonce, plaintext).unwrap();
        let decrypted = SignalCipher::decrypt(&key, &nonce, &ciphertext).unwrap();

        assert_eq!(plaintext.as_slice(), decrypted.as_slice());
    }

    #[test]
    fn test_pre_key_serialization() {
        let pre_key = PreKey::generate(42);
        let serialized = pre_key.serialize();
        let deserialized = PreKey::deserialize(&serialized).unwrap();

        assert_eq!(pre_key.id, deserialized.id);
    }
}
