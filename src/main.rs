use core::error;
use std::io;

use client::{ClientError, NOSTR_EVENT_TAG};
use nostr_sdk::prelude::*;
use tracing::info;
use tracing::error;
use tracing_subscriber;
use utils::load_or_generate_keys;

mod client;
mod utils;
use crate::client::Client;

#[tokio::main]
async fn main() {
    // install global collector configured based on RUST_LOG env var.
    tracing_subscriber::fmt::init();

    // let keys = Keys::parse("nsec1ytvz5cdxfhuj4jg9k47kf9jfecfg8cwgjd5tnygj8cl7l8mc8ljqk7ac7q").unwrap();
    let keys = load_or_generate_keys().await.unwrap();

    info!("Your public key: {}", keys.public_key().to_bech32().unwrap());

    let mut client = Client::new(keys);

    client.connect().await.unwrap();


    info!("Listening for events...");
    client.subscribe_and_listen(|chat_message, event, relay_url| {
        info!("Relay {}, User {}\n{}", relay_url, event.pubkey.to_bech32().unwrap(), chat_message.message);
    }).await.unwrap();
    
    loop {
        let mut input = String::new();
        // print!(">");
        io::stdin().read_line(&mut input).unwrap();
        if input.trim() == "" {
            break;
        }

        let broadcast_res = client.broadcast(input.trim()).await;
        // .tag(Tag::from_standardized(TagStandard::Client{ name: NOSTR_EVENT_TAG.to_string(), address: None, }));

        match broadcast_res {
            Ok(event_id) => {
                info!("Event ID: {}", event_id.to_bech32().unwrap_or("invalid output id".into()));
            },
            Err(error) => {
                match error {
                    ClientError::NostrError(e)=>{error!("{}",e);},
                    ClientError::NotConnected=>{error!("{}",error);},
                    ClientError::SerdeError(e) => {error!("{}",e);},
                }
            },
        }


    }

}
 