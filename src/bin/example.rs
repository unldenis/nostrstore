use core::error;
use std::env;
use std::io;

use nostr_db::client::ChatMessage;
use nostr_db::client::{ClientError, NOSTR_EVENT_TAG};
use nostr_sdk::prelude::*;
use tracing::info;
use tracing::error;
use tracing_subscriber;

use nostr_db::client::Client;

#[tokio::main]
async fn main() {
    // install global collector configured based on RUST_LOG env var.
    tracing_subscriber::fmt::init();

    let keys = Keys::parse("nsec1fy50xae8lnd5pd2tx0yqvsflkmu4j0qefwacskhvdklytrf68vcqxunshc").unwrap();

    info!("Your public key: {}", keys.public_key().to_bech32().unwrap());

    let mut client = Client::new(keys);

    client.connect().await.unwrap();



    let value = client.read("age", true).await.unwrap();
    info!("Before: {:?}", value.iter().map(|x| x.value.clone()).collect::<Vec<String>>());

    client.store("age", "0").await.unwrap();


    let value = client.read("age", true).await.unwrap();
    info!("After: {:?}", value.iter().map(|x| x.value.clone()).collect::<Vec<String>>());

    client.aggregate("age").await.unwrap();

    let value = client.read("age", true).await.unwrap();
    info!("Aggregation: {:?}", value.iter().map(|x| x.value.clone()).collect::<Vec<String>>());

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
 