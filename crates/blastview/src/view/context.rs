use std::{
    collections::HashMap,
    sync::{
        Arc, Mutex,
        atomic::{AtomicUsize, Ordering},
    },
};

use crate::{
    node::Node,
    view::{
        RenderableView, View,
        events::GlobalEventsRegistry,
        registry::{OrderedViewRegistry, ViewRef},
    },
};

pub struct ViewContext {
    order: usize,
    registry: Arc<Mutex<OrderedViewRegistry>>,
    current_order: AtomicUsize,
    event_registry: Arc<GlobalEventsRegistry>,
    handlers: Arc<Mutex<HashMap<String, Arc<dyn Fn() + Send + Sync>>>>,
    last_render: Arc<Mutex<Option<Node>>>,
}

impl ViewContext {
    pub fn new(order: usize, event_registry: Arc<GlobalEventsRegistry>) -> Self {
        Self {
            order,
            registry: Default::default(),
            current_order: Default::default(),
            event_registry,
            handlers: Default::default(),
            last_render: Default::default(),
        }
    }

    pub(crate) fn prepare(&self) {
        self.current_order.store(0, Ordering::Relaxed);
    }

    fn unregister_handlers(&self) {
        let mut handlers = self.handlers.lock().unwrap();
        for (event, _) in handlers.iter() {
            tracing::debug!("unregister event: {event}");
            self.event_registry.remove(event);
        }
        *handlers = Default::default();
    }

    pub fn create<V, F>(&self, factory: F) -> ViewRef
    where
        F: Fn() -> V,
        V: View + Send + Sync + 'static,
    {
        let order = self.get_order();

        if let Some((cx, view)) = self.registry.lock().unwrap().retrieve(order) {
            cx.prepare();
            cx.unregister_handlers();
            let view_ref = ViewRef { order: cx.order };
            let tree = RenderableView::render(view.as_ref(), &cx);
            self.register_node_events(&tree, Arc::clone(&cx));
            *cx.last_render.lock().unwrap() = Some(tree);
            return view_ref;
        }
        let context = ViewContext::new(order, Arc::clone(&self.event_registry));
        let context = Arc::new(context);
        let view = factory();

        let view_ref = ViewRef { order };

        self.registry
            .lock()
            .unwrap()
            .insert(Arc::clone(&context), view);

        let (cx, view) = self.get_ordered(view_ref.order);
        *cx.last_render.lock().unwrap() = Some(RenderableView::render(view.as_ref(), &cx));

        view_ref
    }

    pub(crate) fn retrieve_last_render(self: Arc<Self>) -> Node {
        let node = self.last_render.lock().unwrap().take().unwrap();
        Arc::clone(&self).register_node_events(&node, self);
        node
    }

    fn register_node_events(&self, node: &Node, context: Arc<ViewContext>) {
        match node {
            Node::Element(node) => {
                let cx = context.clone();
                for (event, handler) in node.events.iter() {
                    let cx = Arc::clone(&cx);
                    let event_name = format!("{}_{event}", node.id);
                    tracing::debug!("[{}] event registered: {event_name}", node.tag);
                    {
                        let cx = cx.clone();
                        let mut cx_handlers = cx.handlers.lock().unwrap();
                        cx_handlers.insert(event_name.clone(), Arc::clone(handler));
                    }

                    {
                        let cx_for_event_reg = cx.clone();
                        cx.event_registry.insert(event_name, cx_for_event_reg);
                    }
                }

                for child in node.children.iter() {
                    self.register_node_events(child, Arc::clone(&context));
                }
            }
            _ => {}
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
