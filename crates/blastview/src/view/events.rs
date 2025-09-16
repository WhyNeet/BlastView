use std::sync::Arc;

use dashmap::DashMap;

use crate::view::context::ViewContext;

#[derive(Default)]
pub struct GlobalEventsRegistry {
    mapping: DashMap<String, Arc<ViewContext>>,
}

impl GlobalEventsRegistry {
    pub fn insert(&self, id: String, cx: Arc<ViewContext>) {
        self.mapping.insert(id, cx);
    }

    pub fn get(&self, id: &str) -> Option<Arc<ViewContext>> {
        self.mapping.get(id).map(|cx| Arc::clone(&cx))
    }

    pub fn remove(&self, id: &str) {
        self.mapping.remove(id);
    }
}
