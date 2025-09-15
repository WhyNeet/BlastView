use std::sync::Arc;

use crate::view::{RenderableView, View, context::ViewContext};

#[derive(Default)]
pub struct OrderedViewRegistry {
    views: Vec<(Arc<ViewContext>, Arc<dyn RenderableView + Send + Sync>)>,
}

impl OrderedViewRegistry {
    pub fn insert<V>(&mut self, cx: Arc<ViewContext>, view: V)
    where
        V: View + Send + Sync + 'static,
    {
        self.views.push((cx, Arc::new(view)));
    }

    pub fn retrieve(
        &self,
        order: usize,
    ) -> Option<(Arc<ViewContext>, Arc<dyn RenderableView + Send + Sync>)> {
        self.views.get(order).cloned()
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ViewRef {
    pub(crate) order: usize,
}
