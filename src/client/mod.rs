use tracing::{info, error, warn};
use nostr_sdk::Keys;
use nostr_sdk::prelude::*;
use thiserror::Error;
use serde::{Serialize, Deserialize};

pub const NOSTR_EVENT_TAG : &str = "nostr-dm";
pub const NOSTR_VERSION : &str = "0.1.0";

#[derive(Serialize, Deserialize)]
pub struct ChatMessage {
    pub message: String,
    pub version: String,
}


#[derive(Error, Debug)]
pub enum ClientError {
    #[error("relay pool is not connected. please connect first.")]
    NotConnected,

    // if error is from nostr_sdk 
    #[error("nostr_sdk error: {0}")]
    NostrError(String),

    // if error is from serde 
    #[error("serde error: {0}")]
    SerdeError(String)
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

    pub fn new(keys: Keys) -> Self {
        Client { keys, relay_pool: None }
    }

    pub async fn send_event(&self, builder: EventBuilder) -> Result<EventId, ClientError> {

        match &self.relay_pool {
            Some(pool) => {
                let event: Event = builder.sign(&self.keys).await.map_err(|e| ClientError::NostrError(e.to_string()))?;

                let output =  pool.send_event(&event).await.map_err(|e| ClientError::NostrError(e.to_string()))?;
        
                // println!("Event ID: {}", output.id().to_bech32().unwrap_or("invalid output id".into()));
                // println!("Sent to: {:?}", output.success);
                // println!("Not sent to: {:?}", output.failed);
                // println!("Sending event: {}", event.as_json());
                Ok(*output.id())
            },
            None => {
                Err(ClientError::NotConnected)
            },
        }
    }

    pub async fn broadcast(&self, message : &str) -> Result<EventId, ClientError> {
        let chat_msg = ChatMessage {
            message: message.to_string(),
            version: NOSTR_VERSION.to_string(),
        };

        let json = serde_json::to_string(&chat_msg).map_err(|e| ClientError::SerdeError(e.to_string()))?;

        let builder = EventBuilder::text_note(json)
        .tag(Tag::custom(TagKind::SingleLetter(SingleLetterTag { character: Alphabet::C, uppercase: false }),
        vec![NOSTR_EVENT_TAG.to_string()])) ;

        self.send_event(builder).await
    }

    pub async fn send_encrypted_message(
        &self,
        recipient_pubkey: &PublicKey,
        message: &str,
    ) -> Result<EventId, ClientError> {
        let chat_msg = ChatMessage {
            message: message.to_string(),
            version: NOSTR_VERSION.to_string(),
        };
    
        let json = serde_json::to_string(&chat_msg)
            .map_err(|e| ClientError::SerdeError(e.to_string()))?;
    
        let encrypted_content = self
            .keys
            .nip44_encrypt(recipient_pubkey, &json)
            .await
            .map_err(|e| ClientError::NostrError(e.to_string()))?;

            // .map_err(|e| ClientError::NostrError(e.to_string()))?;
    
        let builder = EventBuilder::text_note(encrypted_content)
            .tag(Tag::custom(TagKind::SingleLetter(SingleLetterTag { character: Alphabet::C, uppercase: false }),
            vec![NOSTR_EVENT_TAG.to_string()])) ;
        self.send_event(builder).await
    }
    

    
    pub async fn connect(self : &mut Client) -> Result<(), nostr_sdk::client::Error> {
        let relay_pool = RelayPool::new();
 
        let relays = vec![
            "wss://nos.lol",
            "wss://relay.damus.io",
            "wss://nostr-pub.wellorder.net",
            "wss://relay.nostr.band",
        ];
        
        for url in relays {
            relay_pool.add_relay(url, RelayOptions::default()).await?;
        }
    

        relay_pool.connect().await;  
 

        self.relay_pool = Some(relay_pool);
        Ok(())
    }

 
    pub async fn subscribe_and_listen<F>(&self, mut on_event: F) -> Result<(), ClientError>
    where
        F: FnMut(ChatMessage, Event, RelayUrl) + Send + 'static,
    {
        let pool = match &self.relay_pool {
            Some(p) => p,
            None => return Err(ClientError::NotConnected),
        };
    
        let filter = Filter::new()
            .kind(Kind::TextNote)
            .custom_tag(SingleLetterTag { character: Alphabet::C, uppercase: false }, NOSTR_EVENT_TAG);
    
        pool.subscribe(filter, SubscribeOptions::default())
            .await
            .map_err(|e| ClientError::NostrError(e.to_string()))?;
    
        let mut notifications = pool.notifications();
        let keys_cloned = self.keys.clone();
    
        tokio::spawn(async move {
            loop {
                match notifications.recv().await {
                    Ok(RelayPoolNotification::Event { event, relay_url, .. }) => {
                        if event.pubkey == keys_cloned.public_key() {
                            info!("Ignoring event from self");
                            continue;
                        }
    
                        if !event.verify_signature() {
                            warn!("Invalid signature for event id: {:?}", event.id);
                            continue;
                        }
    

                        match keys_cloned.nip44_decrypt(&event.pubkey, &event.content).await {
                            Ok(decrypted_json) => {
                                match serde_json::from_str::<ChatMessage>(&decrypted_json) {
                                    Ok(chat_message) => {
                                        if chat_message.version != NOSTR_VERSION {
                                            warn!("Version mismatch: expected {}, got {}", NOSTR_VERSION, chat_message.version);
                                            return;
                                        }
                                        on_event(chat_message, *event, relay_url);
                                    }
                                    Err(e) => {
                                        warn!("Failed to parse decrypted JSON: {}", e);
                                    }
                                }
                            }
                            Err(e) => {
                                warn!("Failed to decrypt message from {}: {}", event.pubkey.to_bech32().unwrap(), e);
                            }
                        }
                                       

                    },
                    Ok(RelayPoolNotification::Shutdown) => break,
                    _ => continue,
                }
            }
        });
    
        Ok(())
    }
    
}