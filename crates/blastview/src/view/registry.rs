use std::sync::{Arc, Mutex, atomic::AtomicUsize};

use crate::view::context::ViewContext;

#[derive(Default)]
pub struct OrderedViewRegistry {
    views: Mutex<Vec<Arc<ViewContext>>>,
    current_order: AtomicUsize,
}

impl OrderedViewRegistry {
    pub fn insert(&self, cx: Arc<ViewContext>) {
        self.views.lock().unwrap().push(cx);
    }

    pub fn retrieve(&self, order: usize) -> Option<Arc<ViewContext>> {
        self.views.lock().unwrap().get(order).cloned()
    }

    pub fn reset_order(&self) {
        self.current_order
            .store(0, std::sync::atomic::Ordering::Relaxed);
    }

    pub fn get_order(&self) -> usize {
        self.current_order
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ViewRef {
    pub(crate) order: usize,
}
