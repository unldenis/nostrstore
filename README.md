# Nostr-DB

Nostr-DB is a lightweight, decentralized key-value store designed to work seamlessly with the Nostr protocol. It provides tools for storing, querying, and managing data in a distributed environment.

## Features
- **Decentralized Storage**: Built to integrate with the Nostr protocol for peer-to-peer data sharing.
- **Lightweight**: Minimal dependencies and optimized for performance.
- **Key-Value Store**: Simple and efficient key-value data storage.
- **Encrypted Data**: All data is encrypted using NIP-44 for enhanced security.

## Getting Started
1. Install the library (instructions coming soon).
2. Import it into your project.
3. Start building decentralized applications with Nostr-DB.

### Example Usage

Below is an example of how to use the library in Rust:

```rust
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
```

## Roadmap
- Initial release with basic key-value operations.
- Advanced querying and indexing.
- Comprehensive documentation and examples.

## Contributing
Contributions are welcome! Please submit issues or pull requests to help improve Nostr-DB.