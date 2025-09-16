use std::sync::{Arc, Mutex, atomic::AtomicUsize};

use crate::view::{RenderableView, View, context::ViewContext};

#[derive(Default)]
pub struct OrderedViewRegistry {
    views: Mutex<Vec<(Arc<ViewContext>, Arc<dyn RenderableView + Send + Sync>)>>,
    current_order: AtomicUsize,
}

impl OrderedViewRegistry {
    pub fn insert<V>(&self, cx: Arc<ViewContext>, view: V)
    where
        V: View + Send + Sync + 'static,
    {
        self.views.lock().unwrap().push((cx, Arc::new(view)));
    }

    pub fn retrieve(
        &self,
        order: usize,
    ) -> Option<(Arc<ViewContext>, Arc<dyn RenderableView + Send + Sync>)> {
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
