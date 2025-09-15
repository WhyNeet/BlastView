use std::{
    collections::HashMap,
    sync::{
        Arc, Mutex,
        atomic::{AtomicUsize, Ordering},
    },
};

use uuid::Uuid;

use crate::{
    node::Node,
    view::{
        RenderableView, View,
        events::GlobalEventsRegistry,
        registry::{OrderedViewRegistry, ViewRef},
    },
};

pub struct ViewContext {
    id: usize,
    registry: Arc<Mutex<OrderedViewRegistry>>,
    current_order: AtomicUsize,
    event_registry: Arc<GlobalEventsRegistry>,
    handlers: Arc<Mutex<HashMap<String, Arc<dyn Fn() + Send + Sync>>>>,
}

impl ViewContext {
    pub fn new(id: usize, event_registry: Arc<GlobalEventsRegistry>) -> Self {
        Self {
            id,
            registry: Default::default(),
            current_order: Default::default(),
            event_registry,
            handlers: Default::default(),
        }
    }

    pub(crate) fn prepare(&self) {
        self.current_order.store(0, Ordering::Relaxed);
        let mut handlers = self.handlers.lock().unwrap();
        for (event, _) in handlers.iter() {
            self.event_registry.remove(event);
        }
        *handlers = Default::default();
    }

    pub fn create<V, F>(&self, factory: F) -> ViewRef
    where
        F: Fn() -> V,
        V: View + Send + Sync + 'static,
    {
        let id = self.get_order();

        if let Some((cx, _)) = self.registry.lock().unwrap().retrieve(id) {
            cx.prepare();
            let view_ref = ViewRef { id: cx.id };
            self.register_view_events(view_ref, cx);
            return view_ref;
        }
        let context = ViewContext::new(id, Arc::clone(&self.event_registry));
        let context = Arc::new(context);
        let view = factory();

        let view_ref = ViewRef { id };

        self.registry
            .lock()
            .unwrap()
            .insert(Arc::clone(&context), view);

        let (cx, view) = self.get_ordered(view_ref.id);
        let tree = RenderableView::render(view.as_ref(), &self);
        self.register_node_events(tree, cx);

        view_ref
    }

    fn register_view_events(&self, view: ViewRef, context: Arc<ViewContext>) {
        let (cx, view) = context.get_ordered(view.id);
        let tree = RenderableView::render(view.as_ref(), &cx);

        self.register_node_events(tree, cx);
    }

    fn register_node_events(&self, node: Node, context: Arc<ViewContext>) {
        match node {
            Node::Element(node) => {
                let cx = context.clone();
                for (event, handler) in node.events.into_iter() {
                    let cx = Arc::clone(&cx);
                    let event_name = format!("{event}_{}", Uuid::new_v4());
                    tracing::debug!("event registered: {event_name}");
                    let cx_handlers = cx.clone();
                    let mut cx_handlers = cx_handlers.handlers.lock().unwrap();
                    cx_handlers.insert(event_name.clone(), Arc::new(handler));

                    let cx_for_event_reg = cx.clone();
                    cx.event_registry.insert(event_name, cx_for_event_reg);
                }
            }
            Node::Text(_) => {}
            Node::ViewRef(view) => self.register_view_events(*view, context),
        }
    }

    pub(crate) fn dispatch_event(&self, event: String) {
        if let Some(handler) = self.handlers.lock().unwrap().get(&event) {
            handler();
        } else if let Some(cx) = self.event_registry.get(&event) {
            cx.dispatch_event(event);
        }
    }

    pub fn get_ordered(
        &self,
        order: usize,
    ) -> (Arc<ViewContext>, Arc<dyn RenderableView + Send + Sync>) {
        self.registry.lock().unwrap().retrieve(order).unwrap()
    }

    fn get_order(&self) -> usize {
        self.current_order.fetch_add(1, Ordering::Relaxed)
    }
}
