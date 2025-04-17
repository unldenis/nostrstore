use nostr_sdk::Keys;
use std::path::PathBuf;
use tokio::fs;
use dirs;
use tracing::{info, warn, error};
use nostr_sdk::prelude::ToBech32;

/// Load existing keys from a local file, or generate and store new keys if none exist.
pub async fn load_or_generate_keys() -> Result<Keys, Box<dyn std::error::Error>> {
    // ~/.nostr-dm/key.txt
    let mut path = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
    path.push(".nostr-dm");

    if let Err(e) = fs::create_dir_all(&path).await {
        error!("Failed to create directory {:?}: {}", path, e);
        return Err(Box::new(e));
    }

    path.push("key.txt");

    if let Ok(data) = fs::read_to_string(&path).await {
        match Keys::parse(data.trim()) {
            Ok(keys) => {
                info!("Loaded existing key from {:?}", path);
                return Ok(keys);
            },
            Err(_) => {
                warn!("⚠️ Invalid key found in {:?}, generating a new one...", path);
            }
        }
    }

    // Generate a new key
    let keys = Keys::generate();

    let nsec = keys.secret_key()
        .to_bech32()
        .map_err(|e| {
            error!("Failed to encode secret key to Bech32: {}", e);
            e
        })?;

    if let Err(e) = fs::write(&path, nsec.clone()).await {
        error!("❌ Failed to write key to {:?}: {}", path, e);
        return Err(Box::new(e));
    }

    info!("New key generated and saved to {:?}", path);
    Ok(keys)
}
