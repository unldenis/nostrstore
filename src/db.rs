use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::future::Future;
use std::io::{self, Read, Write};
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::Duration;
use tokio::io::{AsyncWrite, AsyncWriteExt};
use tracing::info;

use nostr_sdk::prelude::*;
use nostr_sdk::Keys;
use thiserror::Error;

use crate::event_stream::Operation;
use crate::DatabaseBuilder;
use crate::NostrDBError;
use crate::Database;

/// Constants for custom Nostr event kinds
pub const NOSTR_STORE_KIND: u16 = 9215;
pub const NOSTR_STORE_AGGREGATE_KIND: u16 = 39215;

/// Represents an aggregated value in the Nostr database
#[derive(Serialize, Deserialize, Debug, Eq, Clone)]
pub struct AggregateValue {
    pub created_at: u64,
    pub value: String,
    pub event_id: String,
}

impl AggregateValue {
    /// Creates a new `AggregateValue`
    pub fn new(created_at: u64, value: String, event_id: String) -> Self {
        Self {
            created_at,
            value,
            event_id,
        }
    }
}

impl PartialEq for AggregateValue {
    fn eq(&self, other: &Self) -> bool {
        self.event_id == other.event_id
    }
}

impl PartialOrd for AggregateValue {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.created_at.cmp(&other.created_at))
    }
}

impl Ord for AggregateValue {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.created_at.cmp(&other.created_at)
    }
}

impl From<&Event> for AggregateValue {
    fn from(event: &Event) -> Self {
        Self {
            created_at: event.created_at.as_u64(),
            value: event.content.clone(),
            event_id: event.id.to_string(),
        }
    }
}

/// Constructs a Nostr filter for fetching events
fn get_filter(public_key: PublicKey, key: &str, kind: u16) -> Filter {
    Filter::new()
        .kind(Kind::Custom(kind))
        .author(public_key)
        .custom_tag(
            SingleLetterTag {
                character: Alphabet::D,
                uppercase: false,
            },
            key,
        )
}

pub struct QueryOptions {
    decrypt: bool,
}
impl Default for QueryOptions {
    fn default() -> Self {
        Self { decrypt: true }
    }
}
impl QueryOptions {
    pub fn new(decrypt: bool) -> Self {
        Self { decrypt }
    }
}



impl Database {

    pub fn builder(keys: Keys) -> DatabaseBuilder {
        DatabaseBuilder::new(keys)
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

    /// Stores a value in the Nostr database.
    /// The value is encrypted using the NIP-44 encryption scheme and associated with the provided key.
    pub async fn store<T: Into<String>>(&self, key: T, content: &str) -> Result<EventId, NostrDBError> {
        let encrypted_content = self
            .keys
            .nip44_encrypt(&self.keys.public_key, content)
            .await
            .map_err(|e| NostrDBError::NostrError(e.to_string()))?;

        let builder = EventBuilder::new(Kind::Custom(NOSTR_STORE_KIND), encrypted_content)
            .tag(Tag::custom(
                TagKind::SingleLetter(SingleLetterTag {
                    character: Alphabet::D,
                    uppercase: false,
                }),
                vec![key.into()],
            ));

        self.send_event(builder).await
    }

    /// Aggregates values associated with a key in the Nostr database.
    pub async fn aggregate<T: Into<String>>(&self, key: T) -> Result<(), NostrDBError> {
        let key_str: String = key.into();
        let non_aggregates = self.read_non_aggregates(&key_str, false).await?;

        if non_aggregates.is_empty() {
            return Err(NostrDBError::DatabaseError("No events to aggregate".to_string()));
        }

        let mut aggregates = self.read_aggregates(&key_str, false).await?;
        aggregates.extend(non_aggregates.iter().cloned());

        let contents_json = serde_json::to_string(&aggregates)
            .map_err(NostrDBError::SerdeJsonError)?;

        let builder = EventBuilder::new(Kind::Custom(NOSTR_STORE_AGGREGATE_KIND), contents_json)
            .tag(Tag::custom(
                TagKind::SingleLetter(SingleLetterTag {
                    character: Alphabet::D,
                    uppercase: false,
                }),
                vec![key_str.clone()],
            ));
        self.send_event(builder).await?;

        for event in non_aggregates.iter() {
            let delete_event_builder = EventBuilder::delete(
                EventDeletionRequest::new()
                    .id(EventId::parse(&event.event_id).map_err(|e| NostrDBError::NostrError(e.to_string()))?)
                    .reason("delete event"),
            );
            self.send_event(delete_event_builder).await.ok();
        }

        Ok(())
    }

    /// Reads values associated with a key from the Nostr database.
    pub async fn read<T: Into<String>>(&self, key: T, query_options : QueryOptions) -> Result<BTreeSet<AggregateValue>, NostrDBError> {
        let key_str: String = key.into();
        let mut contents = self.read_non_aggregates(&key_str, query_options.decrypt).await?;
        contents.append(&mut self.read_aggregates(&key_str, query_options.decrypt).await?);
        Ok(contents)
    }

    /// Reads non-aggregated values associated with a key.
    async fn read_non_aggregates<T: Into<String>>(&self, key: T, decrypt: bool) -> Result<BTreeSet<AggregateValue>, NostrDBError> {
        let key_str: String = key.into();
        let events = self
            .relay_pool
            .fetch_events(get_filter(self.keys.public_key, &key_str, NOSTR_STORE_KIND), Duration::MAX, ReqExitPolicy::default())
            .await
            .map_err(|e| NostrDBError::NostrError(e.to_string()))?;

        let mut contents = BTreeSet::new();
        for event in events.iter() {
            let value = if decrypt {
                self.keys
                    .nip44_decrypt(&event.pubkey, &event.content)
                    .await
                    .map_err(|e| NostrDBError::NostrError(e.to_string()))?
            } else {
                event.content.clone()
            };
            contents.insert(AggregateValue::new(event.created_at.as_u64(), value, event.id.to_string()));
        }

        Ok(contents)
    }

    /// Reads aggregated values associated with a key.
    async fn read_aggregates(&self, key: &str, decrypt: bool) -> Result<BTreeSet<AggregateValue>, NostrDBError> {
        if let Some(event) = self
            .relay_pool
            .fetch_events(get_filter(self.keys.public_key, key, NOSTR_STORE_AGGREGATE_KIND), Duration::MAX, ReqExitPolicy::default())
            .await
            .map_err(|e| NostrDBError::NostrError(e.to_string()))?
            .first()
        {
            let mut deserialized: Vec<AggregateValue> = serde_json::from_str(&event.content)
                .map_err(NostrDBError::SerdeJsonError)?;

            if decrypt {
                for value in deserialized.iter_mut() {
                    value.value = self
                        .keys
                        .nip44_decrypt(&event.pubkey, &value.value)
                        .await
                        .map_err(|e| NostrDBError::NostrError(e.to_string()))?;
                }
            }

            Ok(deserialized.into_iter().collect())
        } else {
            Ok(BTreeSet::new())
        }
    }

    pub async fn store_event<I : Into<String>, O : Operation>(&self, key : I, operation : O) -> Result<EventId, NostrDBError> {
        let serialized = operation.serialize();
        self.store(key, &serialized).await
    }

    pub async fn read_event<O>(&self, key: impl Into<String>) -> Result<O::Value, NostrDBError>
    where
        O: Operation,
    {
        let values = self.read(key, QueryOptions::new(true)).await?;
    
        let mut acc = O::default();
    
        for ele in values {
            let op = O::deserialize(ele.value)
                .map_err(|e| NostrDBError::EventStreamError(e.to_string()))?;
            acc = op.apply(acc);
        }
    
        Ok(acc)
    }
    
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_aggregate_value_ordering() {
        let value1 = AggregateValue::new(100, "Value1".to_string(), "Event1".to_string());
        let value2 = AggregateValue::new(200, "Value2".to_string(), "Event2".to_string());
        let value3 = AggregateValue::new(150, "Value3".to_string(), "Event3".to_string());

        let mut btree_set = BTreeSet::new();
        btree_set.insert(value1);
        btree_set.insert(value2);
        btree_set.insert(value3);

        let ordered_values: Vec<_> = btree_set.into_iter().collect();

        assert_eq!(ordered_values[0].created_at, 100);
        assert_eq!(ordered_values[1].created_at, 150);
        assert_eq!(ordered_values[2].created_at, 200);
    }

    #[test]
    fn test_aggregate_value_equality() {
        let value1 = AggregateValue::new(100, "Value1".to_string(), "Event1".to_string());
        let value2 = AggregateValue::new(100, "Value2".to_string(), "Event1".to_string());

        assert_eq!(value1, value2);
    }
}
