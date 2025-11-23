//! Signal Protocol integration
//!
//! This module provides integration with the Signal Protocol for end-to-end
//! encrypted messaging. It uses the official libsignal library for all
//! cryptographic operations.

mod client;
mod protocol;
mod store;
mod types;

pub use client::SignalClient;
pub use protocol::SignalProtocol;
pub use store::SignalStore;
pub use types::*;
