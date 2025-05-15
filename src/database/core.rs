use std::collections::BTreeSet;
use std::time::Duration;

use nostr_sdk::prelude::*;
use nostr_sdk::{Keys, RelayPool};

use super::query::QueryOptions;
use super::{DatabaseBuilder, NostrRecord};
use crate::{NostrDBError, Operation};

const NOSTR_STORE_KIND: u16 = 9215;
const NOSTR_STORE_AGGREGATE_KIND: u16 = 39215;

/// Represents a Nostr database with a relay pool and keys.
/// It provides methods to send, store, remove, and read events.
/// It also allows for aggregation of events and deletion of events.
/// It is built using the builder pattern.
/// The database is designed to work with Nostr events and uses the Nostr SDK for event handling.
pub struct Database {
    pub keys: Keys,
    pub relay_pool: RelayPool,
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

impl Database {
    /// Constructs a new Nostr event and sends it to the relay pool.
    async fn send_event(&self, builder: EventBuilder) -> Result<EventId, NostrDBError> {
        let event = builder
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

    /// Aggregates all non-aggregated events associated with the given key into a single event.
    async fn aggregate<T: Into<String>>(&self, key: T) -> Result<(), NostrDBError> {
        let key_str = key.into();
        let non_aggregated = self.read_non_aggregates(&key_str, false).await?;

        if non_aggregated.is_empty() {
            return Err(NostrDBError::DatabaseError("No events to aggregate".into()));
        }

        let mut combined = self.read_aggregates(&key_str, false).await?;
        combined.extend(non_aggregated.iter().cloned());

        let content = serde_json::to_string(&combined)?;
        let builder =
            EventBuilder::new(Kind::Custom(NOSTR_STORE_AGGREGATE_KIND), content).tag(Tag::custom(
                TagKind::SingleLetter(SingleLetterTag {
                    character: Alphabet::D,
                    uppercase: false,
                }),
                vec![&key_str],
            ));

        self.send_event(builder).await?;
        self.delete_events(&non_aggregated).await?;
        Ok(())
    }

    /// Deletes the specified events from the nostr database.
    async fn delete_events(&self, events: &BTreeSet<NostrRecord>) -> Result<(), NostrDBError> {
        let ids: Vec<EventId> = events
            .iter()
            .filter_map(|rec| EventId::parse(&rec.event_id).ok())
            .collect();

        if !ids.is_empty() {
            let delete_builder =
                EventBuilder::delete(EventDeletionRequest::new().ids(ids).reason("delete events"));
            self.send_event(delete_builder).await?;
        }

        Ok(())
    }

    /// Reads non-aggregated events associated with the given key from the database.
    /// This method fetches all events associated with the key and returns them as a BTreeSet.
    async fn read_non_aggregates<T: Into<String>>(
        &self,
        key: T,
        decrypt: bool,
    ) -> Result<BTreeSet<NostrRecord>, NostrDBError> {
        let key_str = key.into();
        let events = self
            .relay_pool
            .fetch_events(
                get_filter(self.keys.public_key, &key_str, NOSTR_STORE_KIND),
                Duration::MAX,
                ReqExitPolicy::default(),
            )
            .await
            .map_err(|e| NostrDBError::NostrError(e.to_string()))?;

        let mut records = BTreeSet::new();
        for event in events {
            let content = if decrypt {
                self.keys
                    .nip44_decrypt(&event.pubkey, &event.content)
                    .await
                    .map_err(|e| NostrDBError::DecryptionError(e))?
            } else {
                event.content.clone()
            };
            records.insert(NostrRecord::new(
                event.created_at.as_u64(),
                content,
                event.id.to_string(),
            ));
        }

        Ok(records)
    }

    /// Reads aggregated events associated with the given key from the database.
    /// This method fetches the first event associated with the key and returns it as a BTreeSet.
    async fn read_aggregates(
        &self,
        key: &str,
        decrypt: bool,
    ) -> Result<BTreeSet<NostrRecord>, NostrDBError> {
        let events = self
            .relay_pool
            .fetch_events(
                get_filter(self.keys.public_key, key, NOSTR_STORE_AGGREGATE_KIND),
                Duration::MAX,
                ReqExitPolicy::default(),
            )
            .await
            .map_err(|e| NostrDBError::NostrError(e.to_string()))?;

        if let Some(event) = events.first() {
            let mut records: Vec<NostrRecord> = serde_json::from_str(&event.content)?;
            if decrypt {
                for record in &mut records {
                    record.content = self
                        .keys
                        .nip44_decrypt(&event.pubkey, &record.content)
                        .await
                        .map_err(|e| NostrDBError::DecryptionError(e))?;
                }
            }
            Ok(records.into_iter().collect())
        } else {
            Ok(BTreeSet::new())
        }
    }

    /// Creates a new instance of the Database struct.
    pub fn builder(keys: Keys) -> DatabaseBuilder {
        DatabaseBuilder::new(keys)
    }

    /// Stores a new key-value pair in the database.
    /// The content is encrypted using the NIP-44 encryption scheme.
    pub async fn store<T: Into<String>>(
        &self,
        key: T,
        content: &str,
    ) -> Result<EventId, NostrDBError> {
        let encrypted = self
            .keys
            .nip44_encrypt(&self.keys.public_key, content)
            .await
            .map_err(|e| NostrDBError::EncryptionError(e))?;

        let builder =
            EventBuilder::new(Kind::Custom(NOSTR_STORE_KIND), encrypted).tag(Tag::custom(
                TagKind::SingleLetter(SingleLetterTag {
                    character: Alphabet::D,
                    uppercase: false,
                }),
                vec![key],
            ));

        self.send_event(builder).await
    }

    /// Removes all values associated with the given key from the database.
    /// This includes deleting the events and resetting the aggregate event to empty.
    pub async fn remove<T: Into<String>>(&self, key: T) -> Result<(), NostrDBError> {
        let key_str = key.into();
        let records = self.read_non_aggregates(&key_str, false).await?;
        self.delete_events(&records).await?;

        // Reset the aggregate event to empty
        let empty = serde_json::to_string(&BTreeSet::<NostrRecord>::new())?;
        let builder =
            EventBuilder::new(Kind::Custom(NOSTR_STORE_AGGREGATE_KIND), empty).tag(Tag::custom(
                TagKind::SingleLetter(SingleLetterTag {
                    character: Alphabet::D,
                    uppercase: false,
                }),
                vec![&key_str],
            ));

        self.send_event(builder).await?;
        Ok(())
    }

    /// Reads the last value associated with the given key from the database.
    /// This method fetches the history of events associated with the key and returns the last one.
    /// If no events are found, it returns an error.
    pub async fn read<T: Into<String>>(&self, key: T) -> Result<String, NostrDBError> {
        let history = self.read_history(key, QueryOptions::default()).await?;
        let last = history
            .last()
            .ok_or_else(|| NostrDBError::DatabaseError("Variable not found".into()))?;

        Ok(last.content.clone())
    }

    /// Reads the history of values associated with the given key from the database.
    /// This method fetches all events associated with the key and returns them as a BTreeSet.
    /// The events are sorted by their creation time.
    pub async fn read_history<T: Into<String>>(
        &self,
        key: T,
        options: QueryOptions,
    ) -> Result<BTreeSet<NostrRecord>, NostrDBError> {
        let key_str = key.into();
        let mut records = self.read_non_aggregates(&key_str, options.decrypt).await?;

        let should_aggregate = records.len() > options.aggregate_count;

        records.append(&mut self.read_aggregates(&key_str, options.decrypt).await?);

        if should_aggregate {
            self.aggregate(&key_str).await?;
        }

        Ok(records)
    }

    /// Stores an event-operation in the database.
    pub async fn store_event<I: Into<String>, O: Operation>(
        &self,
        key: I,
        operation: O,
    ) -> Result<EventId, NostrDBError> {
        let serialized = operation.serialize().map_err(|e| NostrDBError::EventStreamError(e.to_string()))?;
        self.store(key, &serialized).await
    }

    /// Reads the event-stream processed by the given operation.
    /// This method fetches the history of events associated with the key and applies the operation to each event.
    /// It returns the final value after applying all operations.
    pub async fn read_event<O>(&self, key: impl Into<String>) -> Result<O::Value, NostrDBError>
    where
        O: Operation,
    {
        let records = self.read_history(key, QueryOptions::default()).await?;
        let mut acc = O::default();

        for record in records {
            let op = O::deserialize(record.content)
                .map_err(|e| NostrDBError::EventStreamError(e.to_string()))?;
            acc = op.apply(acc);
        }

        Ok(acc)
    }
}
