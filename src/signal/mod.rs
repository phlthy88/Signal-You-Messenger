//! Signal Protocol integration
//!
//! This module provides integration with the Signal Protocol for end-to-end
//! encrypted messaging. It implements the core cryptographic protocols:
//!
//! - **X3DH**: Extended Triple Diffie-Hellman for initial key exchange
//! - **Double Ratchet**: For forward secrecy and post-compromise security
//! - **Sealed Sender**: For metadata protection (TODO)
//!
//! ## Architecture
//!
//! - `crypto`: Low-level cryptographic primitives (keys, HKDF, AEAD)
//! - `x3dh`: X3DH key agreement protocol
//! - `ratchet`: Double Ratchet algorithm implementation
//! - `protocol`: High-level protocol interface
//! - `store`: Encrypted database storage using SQLCipher
//! - `client`: Signal service client for messaging
//! - `types`: Data type definitions

mod client;
mod crypto;
mod protocol;
mod ratchet;
mod store;
mod types;
mod x3dh;

// Re-export main types
pub use client::{ConnectionStatus, LinkingSession, SignalClient, SignalEvent};
pub use crypto::{
    calculate_fingerprint, DhKeyPair, IdentityKeyPair, IdentityPublicKey, MessageKeys, PreKey,
    PreKeyBundle, SignalCipher, SignalHkdf, SignedPreKey,
};
pub use protocol::{ProtocolAddress, SignalProtocol};
pub use ratchet::{MessageHeader, RatchetMessage, SessionState};
pub use store::SignalStore;
pub use types::*;
pub use x3dh::{x3dh_initiate, x3dh_respond, InitialMessage, X3dhResult};
