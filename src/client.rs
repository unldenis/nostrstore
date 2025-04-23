use tracing::{info, error, warn};
use nostr_sdk::Keys;
use nostr_sdk::prelude::*;
use thiserror::Error;
use serde::{Serialize, Deserialize};

pub struct Client {
    pub keys: Keys,
    relays: Vec<String>,
    pub relay_pool: RelayPool, // RelayPool is no longer an Option
}

pub struct ClientBuilder {
    keys: Keys,
    relays: Vec<String>,
}


impl ClientBuilder {
    pub fn new(keys: Keys) -> Self {
        Self {
            keys,
            relays: vec![],
        }
    }

    pub fn with_relays(mut self, relays: Vec<String>) -> Self {
        self.relays = relays;
        self
    }

    pub fn with_default_relays(mut self) -> Self {
        self.relays = vec![
            "wss://relay.damus.io".to_string(),
            "wss://nostr-pub.wellorder.net".to_string(),
            "wss://relay.snort.social".to_string(),
        ];
        self
    }

    pub async fn build(self) -> Result<Client, NostrDBError> {
        if self.relays.is_empty() {
            return Err(NostrDBError::NoRelaysProvided);
        }

        let relay_pool = RelayPool::new();

        for url in self.relays.iter() {
            relay_pool
                .add_relay(url, RelayOptions::default())
                .await
                .map_err(|e| NostrDBError::NostrError(format!("Failed to add relay {}: {}", url, e)))?;
        }

        relay_pool.connect().await;

        Ok(Client {
            keys: self.keys,
            relays: self.relays,
            relay_pool,
        })
    }
}

impl Client {
    pub fn builder(keys: Keys) -> ClientBuilder {
        ClientBuilder::new(keys)
    }

    pub async fn send_event(&self, builder: EventBuilder) -> Result<EventId, NostrDBError> {
        let event: Event = builder
            .sign(&self.keys)
            .await
            .map_err(|e| NostrDBError::NostrError(e.to_string()))?;

        let output = self
            .relay_pool
            .send_event(&event)
            .await
            .map_err(|e| NostrDBError::NostrError(e.to_string()))?;

        Ok(*output.id())
    }
}
#[derive(Debug, Error)]
pub enum NostrDBError {
    #[error("Nostr SDK error: {0}")]
    NostrError(String),

    // Serde JSON error
    #[error("Serde JSON error: {0}")]
    SerdeJsonError(#[from] serde_json::Error),

    #[error("Relay pool error: {0}")]
    RelayPoolError(String),

    // Missing relays error
    #[error("No relays provided")]
    NoRelaysProvided,

    #[error("Unknown error occurred")]
    Unknown,

    // DB error
    #[error("Database error: {0}")]
    DatabaseError(String),
}