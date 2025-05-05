use nostr_sdk::{RelayPool, Keys, RelayOptions};
use crate::error::NostrDBError;
use super::core::Database;

pub struct DatabaseBuilder {
    keys: Keys,
    relays: Vec<String>,
}

impl DatabaseBuilder {
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

    pub async fn build(self) -> Result<Database, NostrDBError> {
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

        Ok(Database {
            keys: self.keys,
            relay_pool,
        })
    }
}
