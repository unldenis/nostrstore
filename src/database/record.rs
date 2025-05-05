use nostr_sdk::prelude::*;
use serde::{Deserialize, Serialize};

/// A struct representing a Database record in Nostr.
/// It's used primarily when aggregating events in one single event.
/// The content is encrypted using the NIP-44 encryption scheme.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NostrRecord {
    pub created_at: u64,
    pub content: String,
    pub event_id: String,
}

impl NostrRecord {
    pub fn new(created_at: u64, content: String, event_id: String) -> Self {
        Self {
            created_at,
            content,
            event_id,
        }
    }
}

impl PartialEq for NostrRecord {
    fn eq(&self, other: &Self) -> bool {
        self.event_id == other.event_id
    }
}
impl Eq for NostrRecord {}

impl PartialOrd for NostrRecord {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.created_at.cmp(&other.created_at))
    }
}
impl Ord for NostrRecord {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.created_at.cmp(&other.created_at)
    }
}

impl From<&Event> for NostrRecord {
    fn from(event: &Event) -> Self {
        Self {
            created_at: event.created_at.as_u64(),
            content: event.content.clone(),
            event_id: event.id.to_string(),
        }
    }
}
