use nostrstore_derive::AppendOnlyStream;
use serde::{Deserialize, Serialize};
use tracing::info;
use tracing_subscriber;

use nostrstore::{
    DatabaseBuilder, QueryOptions,
    operation::counter::CounterEvent,
};
use nostr_sdk::prelude::*;

#[derive(Debug, Serialize, Deserialize, Clone, AppendOnlyStream)]
struct MyPerson {
    pub name: String,
    pub age: u8,
}

impl MyPerson {
    pub fn new(name: String, age: u8) -> Self {
        Self { name, age }
    }
}

#[tokio::main]
async fn main() {
    // install global collector configured based on RUST_LOG env var.
    tracing_subscriber::fmt::init();

    let keys =
        Keys::parse("nsec1fy50xae8lnd5pd2tx0yqvsflkmu4j0qefwacskhvdklytrf68vcqxunshc").unwrap();
    let db = DatabaseBuilder::new(keys.clone())
        .with_default_relays()
        .build()
        .await
        .unwrap();

    // read name from terminal input

    let mut input = String::new();
    println!("Enter your name: ");
    std::io::stdin().read_line(&mut input).unwrap();

    db.store("name", input.trim()).await.unwrap();

    let name = db.read("name").await.unwrap();
    info!("Name: {}", name);

    let history_name = db
        .read_history("name", QueryOptions::default())
        .await
        .unwrap()
        .iter()
        .map(|r| r.content.clone())
        .collect::<Vec<_>>();

    info!("History of Name: {:?}", history_name);

    // event stream
    db.store_event("my-counter", CounterEvent::Increment)
        .await
        .unwrap();

    let counter = db.read_event::<CounterEvent>("my-counter").await.unwrap();
    info!("Counter: {:?}", counter);


    db.store_event("people", MyPerson::new("Maria".to_string(), 18))
        .await
        .unwrap();

    db.store_event("people", MyPerson::new("Giuseppe".to_string(), 22))
        .await
        .unwrap();

    let people : Vec<MyPerson> = db.read_event::<MyPerson>("people").await.unwrap();
    info!("People: {:?}", people);
}
