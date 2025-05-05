use std::collections::BTreeSet;
use tracing::info;
use tracing_subscriber;

use nostr_db::{QueryOptions, DatabaseBuilder};
use nostr_sdk::prelude::*;

#[tokio::main]
async fn main() {
    // install global collector configured based on RUST_LOG env var.
    tracing_subscriber::fmt::init();

    let keys = Keys::parse("nsec1fy50xae8lnd5pd2tx0yqvsflkmu4j0qefwacskhvdklytrf68vcqxunshc").unwrap();
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
        .map(|r| r.content.clone()).collect::<Vec<_>>();

    info!("History of Name: {:?}", history_name);
}