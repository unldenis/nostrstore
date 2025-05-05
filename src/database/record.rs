use nostr_sdk::prelude::*;
use serde::{Deserialize, Serialize};

/// Rappresenta un evento che ha un valore aggregabile.
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

// Ordinamento crescente per timestamp
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
