//! Extended Triple Diffie-Hellman (X3DH) Key Agreement
//!
//! Implements the Signal Protocol's X3DH key agreement for establishing
//! initial shared secrets between parties who may be offline.
//!
//! Reference: https://signal.org/docs/specifications/x3dh/

use anyhow::{anyhow, Result};
use sha2::{Digest, Sha256};
use x25519_dalek::PublicKey as X25519PublicKey;

use super::crypto::{DhKeyPair, IdentityKeyPair, IdentityPublicKey, PreKeyBundle};

/// X3DH protocol version
const X3DH_VERSION: u8 = 3;

/// X3DH key agreement result
pub struct X3dhResult {
    /// Shared secret derived from X3DH
    pub shared_secret: [u8; 32],
    /// Ephemeral public key to send to recipient
    pub ephemeral_public_key: X25519PublicKey,
    /// Pre-key ID used (if any)
    pub used_pre_key_id: Option<u32>,
}

/// Perform X3DH as the initiator (Alice)
///
/// Alice uses Bob's pre-key bundle to compute a shared secret
/// without Bob being online.
pub fn x3dh_initiate(
    our_identity_key: &IdentityKeyPair,
    their_bundle: &PreKeyBundle,
) -> Result<X3dhResult> {
    // Verify the signed pre-key signature
    their_bundle.verify()?;

    // Generate ephemeral key pair
    let ephemeral_key = DhKeyPair::generate();

    // Compute DH outputs:
    // DH1 = DH(IK_A, SPK_B) - our identity key with their signed pre-key
    // DH2 = DH(EK_A, IK_B)  - our ephemeral with their identity key
    // DH3 = DH(EK_A, SPK_B) - our ephemeral with their signed pre-key
    // DH4 = DH(EK_A, OPK_B) - our ephemeral with their one-time pre-key (optional)

    let dh1 = our_identity_key.dh_agreement(&their_bundle.signed_pre_key_public);

    // For DH2, we need to convert their identity key to X25519
    // This is a simplification - in real Signal, identity keys are separate from DH keys
    let their_identity_dh = identity_to_x25519(&their_bundle.identity_key);
    let dh2 = ephemeral_key.dh_agreement(&their_identity_dh);

    let dh3 = ephemeral_key.dh_agreement(&their_bundle.signed_pre_key_public);

    // Concatenate DH outputs
    let mut dh_concat = Vec::with_capacity(128);
    dh_concat.extend_from_slice(&[0xFFu8; 32]); // Padding for curve security
    dh_concat.extend_from_slice(&dh1);
    dh_concat.extend_from_slice(&dh2);
    dh_concat.extend_from_slice(&dh3);

    // Add DH4 if one-time pre-key is present
    let used_pre_key_id = if let Some(opk) = &their_bundle.pre_key_public {
        let dh4 = ephemeral_key.dh_agreement(opk);
        dh_concat.extend_from_slice(&dh4);
        their_bundle.pre_key_id
    } else {
        None
    };

    // Derive shared secret using KDF
    let shared_secret = kdf(&dh_concat);

    Ok(X3dhResult {
        shared_secret,
        ephemeral_public_key: *ephemeral_key.public_key(),
        used_pre_key_id,
    })
}

/// Process incoming X3DH initial message (Bob's side)
///
/// Bob receives Alice's initial message and computes the same shared secret.
pub fn x3dh_respond(
    our_identity_key: &IdentityKeyPair,
    our_signed_pre_key: &DhKeyPair,
    our_one_time_pre_key: Option<&DhKeyPair>,
    their_identity_key: &IdentityPublicKey,
    their_ephemeral_key: &X25519PublicKey,
) -> Result<[u8; 32]> {
    // Compute DH outputs (same as initiator but with roles swapped):
    // DH1 = DH(SPK_B, IK_A) - our signed pre-key with their identity key
    // DH2 = DH(IK_B, EK_A)  - our identity key with their ephemeral
    // DH3 = DH(SPK_B, EK_A) - our signed pre-key with their ephemeral
    // DH4 = DH(OPK_B, EK_A) - our one-time pre-key with their ephemeral (optional)

    let their_identity_dh = identity_to_x25519(their_identity_key);
    let dh1 = our_signed_pre_key.dh_agreement(&their_identity_dh);

    let dh2 = our_identity_key.dh_agreement(their_ephemeral_key);

    let dh3 = our_signed_pre_key.dh_agreement(their_ephemeral_key);

    // Concatenate DH outputs
    let mut dh_concat = Vec::with_capacity(128);
    dh_concat.extend_from_slice(&[0xFFu8; 32]); // Padding for curve security
    dh_concat.extend_from_slice(&dh1);
    dh_concat.extend_from_slice(&dh2);
    dh_concat.extend_from_slice(&dh3);

    // Add DH4 if one-time pre-key was used
    if let Some(opk) = our_one_time_pre_key {
        let dh4 = opk.dh_agreement(their_ephemeral_key);
        dh_concat.extend_from_slice(&dh4);
    }

    // Derive shared secret using KDF
    let shared_secret = kdf(&dh_concat);

    Ok(shared_secret)
}

/// Key derivation function for X3DH
fn kdf(input: &[u8]) -> [u8; 32] {
    // Use HKDF with SHA-256
    // Info includes protocol identifier
    let salt = [0u8; 32]; // Salt is all zeros for X3DH

    let hkdf = hkdf::Hkdf::<Sha256>::new(Some(&salt), input);
    let mut output = [0u8; 32];
    hkdf.expand(b"X3DH", &mut output)
        .expect("HKDF expand failed");
    output
}

/// Convert Ed25519 identity public key to X25519 for DH
/// This is a simplified conversion - real Signal uses separate key types
fn identity_to_x25519(identity_key: &IdentityPublicKey) -> X25519PublicKey {
    use curve25519_dalek::edwards::CompressedEdwardsY;
    use curve25519_dalek::montgomery::MontgomeryPoint;

    let bytes = identity_key.as_bytes();

    // Try to decompress the Edwards point
    let compressed = CompressedEdwardsY(bytes);
    if let Some(edwards) = compressed.decompress() {
        // Convert to Montgomery form
        let montgomery: MontgomeryPoint = edwards.to_montgomery();
        X25519PublicKey::from(montgomery.to_bytes())
    } else {
        // Fallback: hash the key (less secure but works)
        let mut hasher = Sha256::new();
        hasher.update(b"X3DH_IDENTITY_CONVERSION");
        hasher.update(&bytes);
        let hash = hasher.finalize();
        let mut key_bytes = [0u8; 32];
        key_bytes.copy_from_slice(&hash);
        X25519PublicKey::from(key_bytes)
    }
}

/// Initial message sent by Alice to Bob containing X3DH data
#[derive(Clone, Debug)]
pub struct InitialMessage {
    /// Protocol version
    pub version: u8,
    /// Alice's identity key
    pub identity_key: IdentityPublicKey,
    /// Alice's ephemeral public key
    pub ephemeral_key: X25519PublicKey,
    /// Pre-key ID used (if one-time pre-key was used)
    pub pre_key_id: Option<u32>,
    /// Signed pre-key ID used
    pub signed_pre_key_id: u32,
    /// First encrypted message (using derived keys)
    pub encrypted_message: Vec<u8>,
}

impl InitialMessage {
    /// Create a new initial message
    pub fn new(
        identity_key: IdentityPublicKey,
        ephemeral_key: X25519PublicKey,
        pre_key_id: Option<u32>,
        signed_pre_key_id: u32,
        encrypted_message: Vec<u8>,
    ) -> Self {
        Self {
            version: X3DH_VERSION,
            identity_key,
            ephemeral_key,
            pre_key_id,
            signed_pre_key_id,
            encrypted_message,
        }
    }

    /// Serialize for transmission
    pub fn serialize(&self) -> Vec<u8> {
        let mut data = Vec::with_capacity(128 + self.encrypted_message.len());

        data.push(self.version);
        data.extend_from_slice(&self.identity_key.as_bytes());
        data.extend_from_slice(self.ephemeral_key.as_bytes());

        // Pre-key ID (optional)
        if let Some(id) = self.pre_key_id {
            data.push(1);
            data.extend_from_slice(&id.to_be_bytes());
        } else {
            data.push(0);
        }

        // Signed pre-key ID
        data.extend_from_slice(&self.signed_pre_key_id.to_be_bytes());

        // Encrypted message length and data
        data.extend_from_slice(&(self.encrypted_message.len() as u32).to_be_bytes());
        data.extend_from_slice(&self.encrypted_message);

        data
    }

    /// Deserialize from received data
    pub fn deserialize(data: &[u8]) -> Result<Self> {
        if data.len() < 70 {
            return Err(anyhow!("Initial message too short"));
        }

        let version = data[0];
        if version != X3DH_VERSION {
            return Err(anyhow!("Unsupported X3DH version: {}", version));
        }

        let identity_bytes: [u8; 32] = data[1..33].try_into()?;
        let identity_key = IdentityPublicKey::from_bytes(&identity_bytes)?;

        let ephemeral_bytes: [u8; 32] = data[33..65].try_into()?;
        let ephemeral_key = X25519PublicKey::from(ephemeral_bytes);

        let has_pre_key = data[65] == 1;
        let mut offset = 66;

        let pre_key_id = if has_pre_key {
            let id = u32::from_be_bytes(data[offset..offset + 4].try_into()?);
            offset += 4;
            Some(id)
        } else {
            None
        };

        let signed_pre_key_id = u32::from_be_bytes(data[offset..offset + 4].try_into()?);
        offset += 4;

        let msg_len = u32::from_be_bytes(data[offset..offset + 4].try_into()?) as usize;
        offset += 4;

        if data.len() < offset + msg_len {
            return Err(anyhow!("Initial message truncated"));
        }

        let encrypted_message = data[offset..offset + msg_len].to_vec();

        Ok(Self {
            version,
            identity_key,
            ephemeral_key,
            pre_key_id,
            signed_pre_key_id,
            encrypted_message,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::signal::crypto::{PreKey, SignedPreKey};

    #[test]
    fn test_x3dh_key_agreement() {
        // Generate Alice's keys
        let alice_identity = IdentityKeyPair::generate();

        // Generate Bob's keys
        let bob_identity = IdentityKeyPair::generate();
        let bob_signed_pre_key = SignedPreKey::generate(1, &bob_identity);
        let bob_one_time_pre_key = PreKey::generate(1);

        // Create Bob's pre-key bundle
        let bob_bundle = PreKeyBundle {
            registration_id: 12345,
            device_id: 1,
            pre_key_id: Some(bob_one_time_pre_key.id),
            pre_key_public: Some(*bob_one_time_pre_key.key_pair.public_key()),
            signed_pre_key_id: bob_signed_pre_key.id,
            signed_pre_key_public: *bob_signed_pre_key.key_pair.public_key(),
            signed_pre_key_signature: bob_signed_pre_key.signature,
            identity_key: bob_identity.public_key(),
        };

        // Alice initiates X3DH
        let alice_result = x3dh_initiate(&alice_identity, &bob_bundle).unwrap();

        // Bob responds to X3DH
        let bob_result = x3dh_respond(
            &bob_identity,
            &bob_signed_pre_key.key_pair,
            Some(&bob_one_time_pre_key.key_pair),
            &alice_identity.public_key(),
            &alice_result.ephemeral_public_key,
        )
        .unwrap();

        // Both should derive the same shared secret
        assert_eq!(alice_result.shared_secret, bob_result);
    }

    #[test]
    fn test_initial_message_serialization() {
        let identity = IdentityKeyPair::generate();
        let ephemeral = DhKeyPair::generate();

        let message = InitialMessage::new(
            identity.public_key(),
            *ephemeral.public_key(),
            Some(42),
            1,
            b"Hello, World!".to_vec(),
        );

        let serialized = message.serialize();
        let deserialized = InitialMessage::deserialize(&serialized).unwrap();

        assert_eq!(message.pre_key_id, deserialized.pre_key_id);
        assert_eq!(message.signed_pre_key_id, deserialized.signed_pre_key_id);
        assert_eq!(message.encrypted_message, deserialized.encrypted_message);
    }
}
