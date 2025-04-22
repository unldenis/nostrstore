
use std::io::{self, Read, Write};
use std::time::Duration;
use nostr_sdk::Keys;
use nostr_sdk::prelude::*;
use tracing::info;
use super::Client;
use super::ClientError;

use thiserror::Error;

pub const NOSTR_STORE_KIND : u16 = 39219;
pub const NOSTR_DB_TAG : &str = "nostr-db";



use tokio::io::{AsyncWrite, AsyncWriteExt};
use std::pin::Pin;
use std::task::{Context, Poll};
use std::future::Future;


#[derive(Error, Debug)]
pub enum DbError {
    // variable not found
    #[error("variable not found: {0}")]
    VariableNotFound(String),

    // must be a singleton value
    #[error("must be a singleton value")]
    SingletonValue,

    // failed to decrypt content
    #[error("failed to decrypt content")]
    DecryptError,

    // client error
    #[error("client error: {0}")]
    ClientError(ClientError),

    // if error is from nostr_sdk 
    #[error("nostr_sdk error: {0}")]
    NostrError(String),
}

impl Client {

    pub async fn store(&self, content : String, tag_filter : Option<String>,) -> Result<EventId, DbError> {


        let content = self
            .keys
            .nip44_encrypt(&self.keys.public_key, &content)
            .await
            .map_err(|e| DbError::NostrError(e.to_string()))?;
    


        let builder : EventBuilder = match tag_filter {
            Some(filter) => {
                EventBuilder::new(Kind::Custom(NOSTR_STORE_KIND), content)
                .allow_self_tagging()
                .tag(Tag::public_key(self.keys.public_key))
                .tag(Tag::custom(
                    TagKind::SingleLetter(SingleLetterTag {
                        character: Alphabet::D,
                        uppercase: false,
                    }),
                    vec![filter],
                ))
            },
            None => {
                EventBuilder::new(Kind::Custom(NOSTR_STORE_KIND), content)
                .allow_self_tagging()
                .tags(vec![Tag::public_key(self.keys.public_key)])            },
        };



        self.send_event(builder).await.map_err(|e| DbError::ClientError(e))

    }


    pub async fn read(&self, tag_filter : Option<String>) -> Result<String, DbError> {
        // read the event from the relay pool
        if let Some(pool) = &self.relay_pool {


            let filter = match  &tag_filter {
                Some(filter) => {
                    Filter::new()
                        .kind(Kind::Custom(NOSTR_STORE_KIND))
                        .pubkey(self.keys.public_key)
                        .custom_tag(SingleLetterTag { character: Alphabet::D, uppercase: false }, filter)
                },
                None => {
                    Filter::new()
                        .kind(Kind::Custom(NOSTR_STORE_KIND))
                        .pubkey(self.keys.public_key)

                },
            };
       

            // TODO: check pool.database()

            let events = pool
            .fetch_events(filter,Duration::MAX,ReqExitPolicy::default()) .await.map_err(|e| DbError::NostrError(e.to_string()))?;


            // events must be a singleton collection
            if events.len() != 1 {
                return Err(DbError::SingletonValue);
            }

            info!("events: {:?}", events);
            let event= events.first().ok_or_else(|| DbError::VariableNotFound(tag_filter.unwrap_or_default()))?;
            match self.keys.nip44_decrypt(&event.pubkey, &event.content).await {
                Ok(decrypted_msg) => {
                    return Ok(decrypted_msg);
                }
                Err(e) => {
                    return Err(DbError::DecryptError);
                }
            }
            
        }
        Err(DbError::ClientError(ClientError::NotConnected))
  
    }
}


