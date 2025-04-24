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
use nostr_db::db::QueryOptions;
use nostr_db::Database;

#[tokio::main]
async fn main() {
    let keys = Keys::generate();

    let db = Database::builder(keys)
        .with_default_relays()
        .build()
        .await
        .unwrap();


    // Standard database example
    db.store("my_key", "my_val").await.unwrap();
    let value = db.read("my_key").await.unwrap();
    info!("Stored value: {}", value);

    // Historical database example
    db.store("my_key", "my_second_val").await.unwrap();
    let history :BTreeSet<AggregateValue> = db.read_history("my_key", QueryOptions::default()).await.unwrap();
    info!("History: {:?}", history);

    // Event stream example
    db.store_event("my_counter", CounterEvent::Increment).await.unwrap();

    let curr_counter_value = db.read_event::<CounterEvent>("my_counter").await.unwrap();
    info!("Current counter value: {}", curr_counter_value);
}
```

## Roadmap
- Initial release with basic key-value operations.
- Advanced querying and indexing.
- Comprehensive documentation and examples.

## Contributing
Contributions are welcome! Please submit issues or pull requests to help improve Nostr-DB.