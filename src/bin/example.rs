use core::error;
use std::env;
use std::io;

use nostr_db::db::QueryOptions;
use nostr_db::event_stream::CounterEvent;
use nostr_db::event_stream::Operation;
use nostr_db::Database;
use nostr_sdk::prelude::*;
use tracing::info;
use tracing_subscriber;

#[tokio::main]
async fn main() {
    // install global collector configured based on RUST_LOG env var.
    tracing_subscriber::fmt::init();

    let keys = Keys::parse("nsec1fy50xae8lnd5pd2tx0yqvsflkmu4j0qefwacskhvdklytrf68vcqxunshc").unwrap();

    info!("Your public key: {}", keys.public_key().to_bech32().unwrap());

    let db = Database::builder(keys)
        .with_default_relays()
        .build()
        .await
        .unwrap();


    let value = db.read("age", QueryOptions::default()).await.unwrap();
    info!("Before: {:?}", value.iter().map(|x| x.value.clone()).collect::<Vec<String>>());

    db.store("age", "21").await.unwrap();

    let value = db.read("age", QueryOptions::default()).await.unwrap();
    info!("After: {:?}", value.iter().map(|x| x.value.clone()).collect::<Vec<String>>());
    

    db.store_event("my_counter", CounterEvent::Increment).await.unwrap();

    let curr_counter_value = db.read_event::<CounterEvent>("my_counter").await.unwrap();
    info!("Current counter value: {}", curr_counter_value);
    // client.aggregate("age").await.unwrap();

    // let value = client.read("age", QueryOptions::default()).await.unwrap();
    // info!("Aggregation: {:?}", value.iter().map(|x| x.value.clone()).collect::<Vec<String>>());


}
 