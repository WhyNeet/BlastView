use std::sync::Arc;

use dashmap::DashMap;
use uuid::Uuid;

#[derive(Default)]
pub struct EventRegistry {
    mapping: DashMap<Event, Arc<dyn Fn() + Send + Sync>>,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct Event {
    pub node_id: Uuid,
    pub event: String,
}

impl EventRegistry {
    pub fn register(&self, event: Event, handler: Arc<dyn Fn() + Send + Sync>) {
        self.mapping.insert(event, handler);
    }

    pub fn handle(&self, event: &Event) {
        if let Some(handler) = self.mapping.get(event) {
            handler();
        }
    }

    pub fn unregister(&self, event: &Event) {
        self.mapping.remove(event);
    }

    pub fn unregister_for_node(&self, node_id: &Uuid) {
        for pair in self.mapping.iter() {
            if pair.key().node_id == *node_id {
                self.mapping.remove(pair.key());
            }
        }
    }

    pub fn clear(&self) {
        self.mapping.clear();
    }
}
