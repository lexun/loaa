/// Real-time event system for broadcasting data changes
/// This module is used by both the MCP server and web server for SSE updates

use serde::{Deserialize, Serialize};
use tokio::sync::broadcast;

/// Events that can be broadcast to SSE clients
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum DataEvent {
    /// A kid was created
    KidCreated { id: String, name: String },
    /// A kid was deleted
    KidDeleted { id: String },
    /// A task was created
    TaskCreated { id: String, name: String },
    /// A task was updated
    TaskUpdated { id: String, name: String },
    /// A task was deleted
    TaskDeleted { id: String },
    /// A task was completed, creating a ledger entry
    TaskCompleted {
        kid_id: String,
        task_id: String,
        amount: String,
    },
    /// A balance was manually adjusted
    BalanceAdjusted {
        kid_id: String,
        amount: String,
        description: String,
    },
}

/// Sender half of the event channel
pub type EventSender = broadcast::Sender<DataEvent>;

/// Receiver half of the event channel
pub type EventReceiver = broadcast::Receiver<DataEvent>;

/// Create a new event channel with the given capacity
pub fn create_event_channel(capacity: usize) -> EventSender {
    let (tx, _rx) = broadcast::channel(capacity);
    tx
}

/// Helper function to broadcast an event
/// Returns true if at least one receiver got the message
pub fn broadcast_event(tx: &EventSender, event: DataEvent) -> bool {
    match tx.send(event) {
        Ok(count) => count > 0,
        Err(_) => false,
    }
}
