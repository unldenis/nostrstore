use thiserror::Error;

use nostr_sdk::prelude::*;

/// Custom error type for the NostrDB library.
///
/// This enum represents various errors that can occur within the library.
#[derive(Debug, Error)]
pub enum NostrDBError {
    #[error("Nostr SDK error: {0}")]
    NostrError(String),

    #[error("Serde JSON error: {0}")]
    SerdeJsonError(#[from] serde_json::Error),

    #[error("Relay pool error: {0}")]
    RelayPoolError(String),

    #[error("No relays provided")]
    NoRelaysProvided,

    #[error("Database error: {0}")]
    DatabaseError(String),

    #[error("Event stream error: {0}")]
    EventStreamError(String),

    // encryption error
    #[error("Encryption error: {0}")]
    EncryptionError(SignerError),

    // decryption error
    #[error("Decryption error: {0}")]
    DecryptionError(SignerError),

    #[error("Unknown error occurred")]
    Unknown,
}
