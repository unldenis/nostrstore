use core::error;
use std::env;
use std::io;

use nostr_sdk::prelude::*;
use tracing::info;
use tracing::error;
use tracing_subscriber;
use nostr_db::client::{Client, ClientBuilder};

#[tokio::main]
async fn main() {
    // install global collector configured based on RUST_LOG env var.
    tracing_subscriber::fmt::init();

    let keys = Keys::parse("nsec1fy50xae8lnd5pd2tx0yqvsflkmu4j0qefwacskhvdklytrf68vcqxunshc").unwrap();

    info!("Your public key: {}", keys.public_key().to_bech32().unwrap());



    let mut client = ClientBuilder::new(keys)
        .with_default_relays()
        .build()
        .await
        .unwrap();



    let value = client.read("age", true).await.unwrap();
    info!("Before: {:?}", value.iter().map(|x| x.value.clone()).collect::<Vec<String>>());

    client.store("age", "0").await.unwrap();


    let value = client.read("age", true).await.unwrap();
    info!("After: {:?}", value.iter().map(|x| x.value.clone()).collect::<Vec<String>>());

    client.aggregate("age").await.unwrap();

    let value = client.read("age", true).await.unwrap();
    info!("Aggregation: {:?}", value.iter().map(|x| x.value.clone()).collect::<Vec<String>>());


}
 