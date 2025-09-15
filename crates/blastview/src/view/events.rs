use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use crate::view::context::ViewContext;

#[derive(Default)]
pub struct GlobalEventsRegistry {
    mapping: Mutex<HashMap<String, Arc<ViewContext>>>,
}

impl GlobalEventsRegistry {
    pub fn insert(&self, id: String, cx: Arc<ViewContext>) {
        self.mapping.lock().unwrap().insert(id, cx);
    }

    pub fn get(&self, id: &str) -> Option<Arc<ViewContext>> {
        self.mapping.lock().unwrap().get(id).cloned()
    }

    pub fn remove(&self, id: &str) {
        self.mapping.lock().unwrap().remove(id);
    }

    pub fn all_events<F>(&self, mut f: F)
    where
        F: FnMut(&str),
    {
        for (event, _) in self.mapping.lock().unwrap().iter() {
            f(event);
        }
    }
}
