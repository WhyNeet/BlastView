use std::sync::Arc;

use dashmap::DashMap;
use uuid::Uuid;

use crate::view::context::ViewContext;

#[derive(Default)]
pub(crate) struct ContextRegistry {
    mapping: DashMap<Uuid, Arc<ViewContext>>,
}

impl ContextRegistry {
    pub fn insert(&self, id: Uuid, cx: Arc<ViewContext>) {
        self.mapping.insert(id, cx);
    }

    pub fn get(&self, id: &Uuid) -> Option<Arc<ViewContext>> {
        self.mapping.get(id).map(|val| Arc::clone(&val))
    }
}
