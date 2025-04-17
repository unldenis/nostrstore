use tracing::{info, error, warn};
use nostr_sdk::Keys;
use nostr_sdk::prelude::*;
use thiserror::Error;
use serde::{Serialize, Deserialize};

pub const NOSTR_EVENT_TAG : &str = "nostr-dm";
pub const NOSTR_VERSION : &str = "0.1.0";

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ChatMessage {
    pub message: String,              // Can be plaintext or encrypted
    pub version: String,
    pub recipient: Option<PublicKey> // If Some, encrypt `message` field
}

impl ChatMessage {
    pub fn new(message: String, recipient: Option<PublicKey>) -> Self {
        ChatMessage {
            message: message,
            version: NOSTR_VERSION.to_string(),
            recipient,
        }
    }
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

    pub async fn send_chat_message(&self, mut chat_msg: ChatMessage) -> Result<EventId, ClientError> {
        if let Some(recipient) = &chat_msg.recipient {
            let encrypted_msg = self
                .keys
                .nip44_encrypt(recipient, &chat_msg.message)
                .await
                .map_err(|e| ClientError::NostrError(e.to_string()))?;
            
            chat_msg.message = encrypted_msg;
        }
    
        let json = serde_json::to_string(&chat_msg)
            .map_err(|e| ClientError::SerdeError(e.to_string()))?;
    
        let builder = EventBuilder::text_note(json)
            .tag(Tag::custom(
                TagKind::SingleLetter(SingleLetterTag {
                    character: Alphabet::C,
                    uppercase: false,
                }),
                vec![NOSTR_EVENT_TAG.to_string()],
            ));
    
        self.send_event(builder).await
    }
    


    pub async fn send_event(&self, builder: EventBuilder) -> Result<EventId, ClientError> {

        match &self.relay_pool {
            Some(pool) => {
                let event: Event = builder.sign(&self.keys).await.map_err(|e| ClientError::NostrError(e.to_string()))?;

                let output =  pool.send_event(&event).await.map_err(|e| ClientError::NostrError(e.to_string()))?;
        
                Ok(*output.id())
            },
            None => {
                Err(ClientError::NotConnected)
            },
        }
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
    

                        match serde_json::from_str::<ChatMessage>(&event.content) {
                            Ok(mut chat_msg) => {
                                if chat_msg.version != NOSTR_VERSION {
                                    warn!("Version mismatch: expected {}, got {}", NOSTR_VERSION, chat_msg.version);
                                    return;
                                }
                        
                                if let Some(_) = &chat_msg.recipient {
                                    // Try to decrypt only the `message` field
                                    match keys_cloned.nip44_decrypt(&event.pubkey, &chat_msg.message).await {
                                        Ok(decrypted_msg) => {
                                            chat_msg.message = decrypted_msg;
                                            on_event(chat_msg, *event, relay_url);
                                        }
                                        Err(e) => {
                                            warn!(
                                                "Failed to decrypt field `message` from {}: {}",
                                                event.pubkey.to_bech32().unwrap_or_else(|_| "<invalid>".into()),
                                                e
                                            );
                                        }
                                    }
                                } else {
                                    // No decryption needed
                                    on_event(chat_msg, *event, relay_url);
                                }
                            }
                            Err(e) => {
                                // warn!("Failed to parse ChatMessage JSON: {}", e);
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