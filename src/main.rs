use core::error;
use std::env;
use std::io;

use client::ChatMessage;
use client::{ClientError, NOSTR_EVENT_TAG};
use dotenv::dotenv;
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

    dotenv().unwrap(); // Reads the .env file

    let keys = match env::var("SEC_KEY") {
        Ok(val) => Keys::parse(&val).unwrap(),
        Err(_) => load_or_generate_keys().await.unwrap(),
    };

    info!("Your public key: {}", keys.public_key().to_bech32().unwrap());

    let mut client = Client::new(keys);

    client.connect().await.unwrap();


    let event_id = client.store("1".to_string(), Some("age".to_string())).await.unwrap();

    info!("Event ID: {}", event_id.to_bech32().unwrap());

    let value = client.read(Some("age".to_string())).await.unwrap();

    info!("Value: {}", value);

    // info!("Listening for events...");
    // client.subscribe_and_listen(|chat_message, event: Event, relay_url| {

    //     match chat_message.recipient {
    //         Some(recipient) => {
    //             info!("[Private] User {} :\n{}", event.pubkey.to_bech32().unwrap(), chat_message.message);
    //         },
    //         None => {
    //             info!("[Public] User {} :\n{}", event.pubkey.to_bech32().unwrap(), chat_message.message);
           
    //             // info!("[Public] Relay {}, User {}\n{}", relay_url, event.pubkey.to_bech32().unwrap(), chat_message.message);
    //         },
    //     }
    // }).await.unwrap();
    
    // loop {
    //     let mut input = String::new();
    //     // print!(">");
    //     io::stdin().read_line(&mut input).unwrap();
    //     if input.trim() == "" {
    //         break;
    //     }

    //     let broadcast_res = client.send_chat_message(
    //         ChatMessage::new(input.trim().to_owned(), None)).await;
    //     // .tag(Tag::from_standardized(TagStandard::Client{ name: NOSTR_EVENT_TAG.to_string(), address: None, }));

    //     match broadcast_res {
    //         Ok(event_id) => {
    //             info!("Event ID: {}", event_id.to_bech32().unwrap_or("invalid output id".into()));
    //         },
    //         Err(error) => {
    //             match error {
    //                 ClientError::NostrError(e)=>{error!("{}",e);},
    //                 ClientError::NotConnected=>{error!("{}",error);},
    //                 ClientError::SerdeError(e) => {error!("{}",e);},
    //             }
    //         },
    //     }


    // }

}
 