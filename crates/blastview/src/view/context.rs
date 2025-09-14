use std::{cell::RefCell, rc::Rc};

use crate::view::{
    RenderableView, View,
    registry::{OrderedViewRegistry, ViewRef},
};

pub struct ViewContext {
    id: usize,
    registry: RefCell<OrderedViewRegistry>,
    current_order: RefCell<usize>,
}

impl ViewContext {
    pub fn new(id: usize) -> Self {
        Self {
            id,
            registry: Default::default(),
            current_order: Default::default(),
        }
    }

    pub(crate) fn prepare(&self) {
        self.current_order.replace(0);
    }

    pub fn create<V, F>(&self, factory: F) -> ViewRef
    where
        F: Fn() -> V,
        V: View + 'static,
    {
        let id = self.get_order();

        if let Some((cx, _)) = self.registry.borrow().retrieve(id) {
            cx.prepare();
            return ViewRef { id: cx.id };
        }
        let context = ViewContext::new(id);
        let view = factory();

        self.registry.borrow_mut().insert(context, view);

        ViewRef { id }
    }

    pub fn get_ordered(&self, order: usize) -> (Rc<ViewContext>, Rc<dyn RenderableView>) {
        self.registry.borrow().retrieve(order).unwrap()
    }

    fn get_order(&self) -> usize {
        let mut order = self.current_order.borrow_mut();
        let id = *order;
        *order += 1;
        id
    }
}
