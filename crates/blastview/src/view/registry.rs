use std::rc::Rc;

use crate::view::{RenderableView, View, context::ViewContext};

#[derive(Default)]
pub struct OrderedViewRegistry {
    views: Vec<(Rc<ViewContext>, Rc<dyn RenderableView>)>,
}

impl OrderedViewRegistry {
    pub fn insert<V>(&mut self, cx: ViewContext, view: V)
    where
        V: View + 'static,
    {
        self.views.push((Rc::new(cx), Rc::new(view)));
    }

    pub fn retrieve(&self, order: usize) -> Option<(Rc<ViewContext>, Rc<dyn RenderableView>)> {
        self.views.get(order).cloned()
    }
}

pub struct ViewRef {
    pub(crate) id: usize,
}
