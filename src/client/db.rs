use std::collections::BTreeSet;
use std::io::{self, Read, Write};
use std::time::Duration;
use nostr_sdk::Keys;
use nostr_sdk::prelude::*;
use tracing::info;
use super::Client;
use super::ClientError;
use serde::{Serialize, Deserialize};
use thiserror::Error;
use tokio::io::{AsyncWrite, AsyncWriteExt};
use std::pin::Pin;
use std::task::{Context, Poll};
use std::future::Future;

pub const NOSTR_STORE_KIND: u16 = 9215;
pub const NOSTR_STORE_AGGREGATE_KIND: u16 = 39215;

#[derive(Error, Debug)]
pub enum DbError {

    // no events to aggregate
    #[error("no events to aggregate")]
    NoEventsToAggregate,

    #[error("variable not found: {0}")]
    VariableNotFound(String),
    #[error("failed to decrypt content")]
    DecryptError,
    #[error("client error: {0}")]
    ClientError(ClientError),
    #[error("nostr_sdk error: {0}")]
    NostrError(String),
    #[error("serde json error: {0}")]
    SerdeJsonError(String),
}

#[derive(Serialize, Deserialize, Debug, Eq)]
pub struct AggregateValue {
    pub created_at: u64,
    pub value: String,
    pub event_id: String,
}

impl AggregateValue {
    pub fn new(created_at: u64, value: String, event_id: String) -> Self {
        AggregateValue {
            created_at,
            value,
            event_id,
        }
    }
}

impl PartialEq for AggregateValue {
    fn eq(&self, other: &Self) -> bool {
        self.created_at == other.created_at
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
        AggregateValue {
            created_at: event.created_at.as_u64(),
            value: event.content.clone(),
            event_id: event.id.to_string(),
        }
    }
}

fn get_filter(public_key: PublicKey, key: &str, kind: u16) -> Filter {
    Filter::new()
        .kind(Kind::Custom(kind))
        .author(public_key)
        .custom_tag(SingleLetterTag { character: Alphabet::D, uppercase: false }, key)
}

impl Client {

    /// Store a value in the Nostr database
    /// The value is encrypted using the NIP-44 encryption scheme
    /// and is associated with the provided key.
    /// The key is used as a tag in the event.
    /// The function returns the event ID of the stored event.      
    pub async fn store<T: Into<String>>(&self, key: T, content: &str) -> Result<EventId, DbError> {
        let encrypted_content = self.keys
            .nip44_encrypt(&self.keys.public_key, content)
            .await
            .map_err(|e| DbError::NostrError(e.to_string()))?;

        let builder = EventBuilder::new(Kind::Custom(NOSTR_STORE_KIND), encrypted_content)
            .tag(Tag::custom(
                TagKind::SingleLetter(SingleLetterTag {
                    character: Alphabet::D,
                    uppercase: false,
                }),
                vec![key],
            ));

        self.send_event(builder).await.map_err(DbError::ClientError)
    }


    /// Aggregate the values associated with a key in the Nostr database.
    /// The function fetches the events associated with the key and
    /// deserializes the content into a BTreeSet of AggregateValue.
    /// The function then creates a new event with the aggregated values
    /// and sends it to the Nostr network.
    /// The function also deletes the original events associated with the key.
    pub async fn aggregate<T: Into<String>>(&self, key: T) -> Result<(), DbError> {
        let key_str: String = key.into();
        if let Some(pool) = &self.relay_pool {
            let mut values = self.read_non_aggregates(&key_str, false).await?;

            // no events to aggregate
            if values.is_empty() {
                return Err(DbError::NoEventsToAggregate);
            }

            for event in values.iter() {
                let delete_event_builder = EventBuilder::delete(EventDeletionRequest::new()
                    .id(EventId::parse(&event.event_id).map_err(|e| DbError::NostrError(e.to_string()))?)
                    .reason("delete event"));
                self.send_event(delete_event_builder).await.ok();
            }


            // add already present aggregates
            values.append(&mut self.read_aggregates(pool, &key_str, false).await?);
            

            // push
            let contents_json = serde_json::to_string(&values).map_err(|e| DbError::SerdeJsonError(e.to_string()))?;
            let builder = EventBuilder::new(Kind::Custom(NOSTR_STORE_AGGREGATE_KIND), contents_json)
                .tag(Tag::custom(
                    TagKind::SingleLetter(SingleLetterTag {
                        character: Alphabet::D,
                        uppercase: false,
                    }),
                    vec![key_str],
                ));
            self.send_event(builder).await.map_err(DbError::ClientError)?;

            Ok(())
        } else {
            Err(DbError::ClientError(ClientError::NotConnected))
        }
    }


    /// Read the value associated with a key from the Nostr database.
    /// The function fetches the events associated with the key and
    /// deserializes the content into a BTreeSet of AggregateValue.
    /// If no events are found, an empty BTreeSet is returned.
    pub async fn read<T: Into<String>>(&self, key: T, decrypt : bool) -> Result<BTreeSet<AggregateValue>, DbError> {
        let key_str: String = key.into();
        if let Some(pool) = &self.relay_pool {
            let mut contents = self.read_non_aggregates( &key_str, decrypt).await?;
            // info!("now reading aggregates");
            contents.append(&mut self.read_aggregates(pool, &key_str, decrypt).await?);

            Ok(contents)
        } else {
            Err(DbError::ClientError(ClientError::NotConnected))
        }
    }

    /// Read the non-aggregated values associated with a key from the Nostr database.
    /// The function fetches the events associated with the key and
    /// deserializes the content into a BTreeSet of AggregateValue.
    /// If no events are found, an empty BTreeSet is returned.
    /// This function is used to read the raw values before they are aggregated.
    pub async fn read_non_aggregates<T: Into<String>>(&self, key: T, decrypt : bool) -> Result<BTreeSet<AggregateValue>, DbError> {
        let key_str: String = key.into();
        if let Some(pool) = &self.relay_pool {
            let events = pool
                .fetch_events(get_filter(self.keys.public_key, &key_str, NOSTR_STORE_KIND), Duration::MAX, ReqExitPolicy::default())
                .await
                .map_err(|e| DbError::NostrError(e.to_string()))?;


            // info!("Events: {:?}", events);
            
            let mut contents = BTreeSet::new();
            for event in events.iter() {
                let value = if decrypt {
                    self.keys.nip44_decrypt(&event.pubkey, &event.content)
                        .await
                        .map_err(|e| DbError::NostrError(e.to_string()))?
                } else {
                    event.content.clone()
                };
                contents.insert(AggregateValue::new(event.created_at.as_u64(), value, event.id.to_string()));
            }

            Ok(contents)
        } else {
            Err(DbError::ClientError(ClientError::NotConnected))
        }
    }
    

    /// Read the aggregates for a given key from the Nostr database.
    /// The function fetches the events associated with the key and
    /// deserializes the content into a BTreeSet of AggregateValue.
    /// The function returns the BTreeSet of AggregateValue.
    /// If no events are found, an empty BTreeSet is returned.
    async fn read_aggregates(&self, pool: &RelayPool, key: &str, decrypt : bool) -> Result<BTreeSet<AggregateValue>, DbError> {
        if let Some(event) = pool
            .fetch_events(get_filter(self.keys.public_key, key, NOSTR_STORE_AGGREGATE_KIND), Duration::MAX, ReqExitPolicy::default())
            .await
            .map_err(|e| DbError::NostrError(e.to_string()))?
            .first()
        {


            let mut deserialized : Vec<AggregateValue> = serde_json::from_str(&event.content)
            .map_err(|e| DbError::SerdeJsonError(e.to_string()))?;

            // decrypt the content
            if decrypt {
                for value in deserialized.iter_mut() {

                    value.value = self.keys.nip44_decrypt(&event.pubkey, &value.value)
                        .await
                        .map_err(|e| DbError::NostrError(e.to_string()))?;
                }
            }

    
            let deserialized : BTreeSet<AggregateValue> = deserialized.into_iter().collect();

            
            Ok(deserialized)
        } else {
            Ok(BTreeSet::new())
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_aggregate_value_ordering() {
        let value1 = AggregateValue {
            created_at: 100,
            value: "Value1".to_string(),
            event_id: "Event1".to_string(),
        };

        let value2 = AggregateValue {
            created_at: 200,
            value: "Value2".to_string(),
            event_id: "Event2".to_string(),
        };

        let value3 = AggregateValue {
            created_at: 150,
            value: "Value3".to_string(),
            event_id: "Event3".to_string(),
        };

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
        let value1 = AggregateValue {
            created_at: 100,
            value: "Value1".to_string(),
            event_id: "Event1".to_string(),
        };

        let value2 = AggregateValue {
            created_at: 100,
            value: "Value2".to_string(),
            event_id: "Event2".to_string(),
        };

        assert_eq!(value1, value2);
    }

    #[test]
    fn test_btreeset_with_duplicate_aggregate_values() {
        let value1 = AggregateValue {
            created_at: 100,
            value: "Value1".to_string(),
            event_id: "Event1".to_string(),
        };

        let value2 = AggregateValue {
            created_at: 100,
            value: "Value2".to_string(),
            event_id: "Event2".to_string(),
        };

        let mut btree_set: BTreeSet<AggregateValue> = BTreeSet::new();
        btree_set.insert(value1);
        btree_set.insert(value2);

        // Since both values have the same `created_at`, only one should be in the set
        assert_eq!(btree_set.len(), 1);
    }

    #[tokio::test]
    async fn test_store_and_read_data() {
        let keys = Keys::generate(); // Generate keys for testing
        let mut client = Client::new(keys); // Create a client instance
        client.connect().await.unwrap();

        let key = "test_key";
        let content = "test_content";

        // Store the data
        let store_result = client.store(key, content).await;
        assert!(store_result.is_ok(), "Failed to store data: {:?}", store_result);

        // Read the data
        let read_result = client.read(key, true).await;
        assert!(read_result.is_ok(), "Failed to read data: {:?}", read_result);

        let values = read_result.unwrap();
        assert_eq!(values.len(), 1, "Expected one value in the database");

        let value = values.iter().next().unwrap();
        assert_eq!(value.value, content, "Stored content does not match");
    }

    #[tokio::test]
    async fn test_aggregate_data() {
        let keys = Keys::generate(); // Generate keys for testing
        let mut client = Client::new(keys); // Create a client instance
        client.connect().await.unwrap();

        let key = "aggregate_key";
        let content1 = "content1";
        let content2 = "content2";

        // Store multiple values
        let store_result1 = client.store(key, content1).await;
        assert!(store_result1.is_ok(), "Failed to store first value: {:?}", store_result1);

        let store_result2 = client.store(key, content2).await;
        assert!(store_result2.is_ok(), "Failed to store second value: {:?}", store_result2);

        // Aggregate the values
        let aggregate_result = client.aggregate(key).await;
        assert!(aggregate_result.is_ok(), "Failed to aggregate data: {:?}", aggregate_result);

        // Read the aggregated data
        let read_result = client.read(key, true).await;
        assert!(read_result.is_ok(), "Failed to read aggregated data: {:?}", read_result);

        let values = read_result.unwrap();
        assert_eq!(values.len(), 2, "Expected two aggregated values");

        let mut values_vec: Vec<_> = values.into_iter().collect();
        values_vec.sort_by_key(|v| v.created_at);

        assert_eq!(values_vec[0].value, content1, "First aggregated content does not match");
        assert_eq!(values_vec[1].value, content2, "Second aggregated content does not match");
    }
}
