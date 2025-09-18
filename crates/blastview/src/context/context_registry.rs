use std::sync::Arc;

use dashmap::DashMap;
use uuid::Uuid;

use crate::context::Context;

#[derive(Default)]
pub struct ContextRegistry {
    mapping: DashMap<Uuid, Arc<Context>>,
}

impl ContextRegistry {
    pub fn register(&self, id: Uuid, cx: Arc<Context>) {
        self.mapping.insert(id, cx);
    }

    pub fn get(&self, id: &Uuid) -> Option<Arc<Context>> {
        self.mapping.get(id).map(|val| Arc::clone(&val))
    }

    pub fn clear(&self) {
        self.mapping.clear();
    }
}
