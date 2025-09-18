use std::sync::{Arc, Mutex};

use crate::context::Context;

#[derive(Default)]
pub struct OrderedViewRegistry {
    views: Mutex<Vec<Arc<Context>>>,
}

impl OrderedViewRegistry {
    pub fn register(&self, cx: Arc<Context>) {
        self.views.lock().unwrap().push(cx);
    }

    pub fn get(&self, order: usize) -> Option<Arc<Context>> {
        self.views.lock().unwrap().get(order).cloned()
    }

    pub fn each<F>(&self, f: F)
    where
        F: Fn(&Context),
    {
        self.views.lock().unwrap().iter().for_each(|cx| f(&cx));
    }
}
