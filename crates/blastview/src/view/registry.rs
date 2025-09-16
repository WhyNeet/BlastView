use std::sync::{Arc, Mutex};

use crate::view::{RenderableView, View, context::ViewContext};

#[derive(Default)]
pub struct OrderedViewRegistry {
    views: Mutex<Vec<(Arc<ViewContext>, Arc<dyn RenderableView + Send + Sync>)>>,
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
}

#[derive(Debug, Clone, Copy)]
pub struct ViewRef {
    pub(crate) order: usize,
}
