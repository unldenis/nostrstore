use std::collections::HashSet;
use std::f32::consts::E;
use std::sync::Arc;

use nostr_sdk::Keys;
use nostr_sdk::prelude::*;
use thiserror::Error;

pub const NOSTR_DM_KIND: u16 = 43872;
pub const NOSTR_EVENT_TAG : &str = "nostr-dm";


#[derive(Error, Debug)]
pub enum ClientError {
    #[error("relay pool is not connected. please connect first.")]
    NotConnected,

    // if error is from nostr_sdk 
    #[error("nostr_sdk error: {0}")]
    NostrError(String),
}

pub struct Client {
    keys : Keys,
    relay_pool : Option<RelayPool>,
}

impl Default for Client {
    fn default() -> Self { 
        // let keys = Keys::parse("nsec1ytvz5cdxfhuj4jg9k47kf9jfecfg8cwgjd5tnygj8cl7l8mc8ljqk7ac7q").unwrap();

        let keys = Keys::generate();
        // let keys = Keys::generate();
        Client { keys, relay_pool : None }  
     }
}

impl Client {

    pub async fn send_event(&self, builder: EventBuilder) -> Result<(), ClientError> {

        match &self.relay_pool {
            Some(pool) => {
                let event: Event = builder.sign(&self.keys).await.map_err(|e| ClientError::NostrError(e.to_string()))?;

                let output =  pool.send_event(&event).await.map_err(|e| ClientError::NostrError(e.to_string()))?;
        
                // println!("Event ID: {}", output.id().to_bech32().unwrap_or("invalid output id".into()));
                // println!("Sent to: {:?}", output.success);
                // println!("Not sent to: {:?}", output.failed);
                // println!("Sending event: {}", event.as_json());
                Ok(())
            },
            None => {
                Err(ClientError::NotConnected)
            },
        }
    }

    
    pub async fn connect(self : &mut Client) -> Result<(), nostr_sdk::client::Error> {

        // Create a relay pool with some test relays
        let relay_pool = RelayPool::new();
        // relay_pool
        //     .add_relay("wss://relay.nostr.net", RelayOptions::default())
        //     .await?;

        relay_pool.add_relay("wss://nos.lol", RelayOptions::default())
            .await?;
        relay_pool.connect().await;  
 

        self.relay_pool = Some(relay_pool);
        Ok(())
    }


    pub async fn subscribe_and_listen(&self) -> Result<(), ClientError> {
        let pool = match &self.relay_pool {
            Some(p) => p,
            None => return Err(ClientError::NotConnected),
        };

        let filter: Filter = Filter::new()
            // content = NOSTR_EVENT_TAG
            // .custom_tag(tag, value)
            .custom_tag(SingleLetterTag { character: Alphabet::C, uppercase: false }, NOSTR_EVENT_TAG);

        pool.subscribe(filter, SubscribeOptions::default()).await.map_err(|e| ClientError::NostrError(e.to_string()))?;

    
        let mut notifications = pool.notifications();

        let keysCloned = self.keys.clone();
        tokio::spawn(async move {
            // let mut seen_events = HashSet::new();

           loop {
                let (event, relay_url) = match notifications.recv().await {
                    Ok(RelayPoolNotification::Event {
                        event, relay_url, ..
                    }) => (event, relay_url),
                    Ok(RelayPoolNotification::Shutdown) => break,
                    _ => continue,
                };

                if event.pubkey == keysCloned.public_key() {
                    // log::trace!("Ignoring event from self");
                    continue;
                }
            
                // if !seen_events.insert(event.id) {
                //     continue;
                // }

                if !event.verify_signature() {
                    println!("Invalid signature for event id: {:?}", event.id);
                    continue;
                }
            
                println!("📥 Received event:\n{}", event.as_json());

            }
            
        });

        Ok(())
    }
}