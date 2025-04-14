use std::io;

use client::{ClientError, NOSTR_EVENT_TAG};
use nostr_sdk::prelude::*;

mod client;
use crate::client::Client;

#[tokio::main]
async fn main() {
    println!("Hello, world!");

    let mut client = Client::default();

    client.connect().await.unwrap();


    println!("Listening for events...");
    client.subscribe_and_listen().await.unwrap();

    loop {
        let mut input = String::new();
        print!(">");
        io::stdin().read_line(&mut input).unwrap();
        if input.trim() == "" {
            break;
        }

        let builder = EventBuilder::text_note(input)
        .tag(Tag::custom(TagKind::SingleLetter(SingleLetterTag { character: Alphabet::C, uppercase: false }),
        vec![NOSTR_EVENT_TAG.to_string()])) ;
        // .tag(Tag::from_standardized(TagStandard::Client{ name: NOSTR_EVENT_TAG.to_string(), address: None, }));

        match client.send_event(builder).await {
            Ok(_) => {

            },
            Err(error) => {
                match error {
                    ClientError::NostrError(e) => {
                        println!("{}", e);
                    },
                    ClientError::NotConnected => {
                        println!("{}", error);
                    },
                }
            },
        }


    }

}
 