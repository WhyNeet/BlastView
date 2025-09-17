use std::sync::{Arc, Mutex};

use dashmap::DashMap;
use uuid::Uuid;

use crate::{
    node::Node,
    session::{RenderingContext, context_registry::ContextRegistry},
    view::{
        RenderableView, View,
        events::GlobalEventsRegistry,
        registry::{OrderedViewRegistry, ViewRef},
        state::ViewContextState,
    },
};

pub struct ViewContext {
    pub(crate) id: Uuid,
    rendering_context: Arc<RenderingContext>,
    registry: Arc<OrderedViewRegistry>,
    state: Arc<ViewContextState>,
    event_registry: Arc<GlobalEventsRegistry>,
    handlers: DashMap<String, Arc<dyn Fn() + Send + Sync>>,
    last_render: Arc<Mutex<Option<Node>>>,
    context_registry: Arc<ContextRegistry>,
    pub(crate) view: Mutex<Option<Arc<dyn RenderableView + Send + Sync>>>,
}

impl ViewContext {
    pub(crate) fn new(
        event_registry: Arc<GlobalEventsRegistry>,
        rendering_context: Arc<RenderingContext>,
        context_registry: Arc<ContextRegistry>,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            registry: Default::default(),
            event_registry,
            rendering_context,
            handlers: Default::default(),
            state: Default::default(),
            last_render: Default::default(),
            view: Default::default(),
            context_registry,
        }
    }

    pub(crate) fn prepare(&self) {
        self.registry.reset_order();
        self.state.reset_order();
    }

    fn unregister_handlers(&self) {
        for entry in self.handlers.iter() {
            let event = entry.key();
            tracing::debug!("unregister event: {event}");
            self.event_registry.remove(event);
        }
        self.handlers.clear();
    }

    pub fn create<V, F>(&self, factory: F) -> ViewRef
    where
        F: Fn() -> V,
        V: View + Send + Sync + 'static,
    {
        let order = self.registry.get_order();

        if let Some(cx) = self.registry.retrieve(order) {
            cx.trigger_render();
            return ViewRef { order };
        }
        let context = ViewContext::new(
            Arc::clone(&self.event_registry),
            Arc::clone(&self.rendering_context),
            Arc::clone(&self.context_registry),
        );
        let context = Arc::new(context);
        self.context_registry
            .insert(context.id, Arc::clone(&context));
        let view = factory();
        *context.view.lock().unwrap() = Some(Arc::new(view));

        let view_ref = ViewRef { order };

        self.registry.insert(Arc::clone(&context));

        context.trigger_render();

        view_ref
    }

    pub(crate) fn trigger_render(self: Arc<Self>) {
        self.prepare();
        self.unregister_handlers();
        self.state.clean();
        let tree =
            RenderableView::render(self.view.lock().unwrap().as_ref().unwrap().as_ref(), &self);
        self.register_node_events(&tree, Arc::clone(&self));
        *self.last_render.lock().unwrap() = Some(tree);
    }

    pub fn use_state<T>(&self, initial_value: T) -> (T, Arc<dyn Fn(T) + Send + Sync>)
    where
        T: Send + Sync + PartialEq + Clone + 'static,
    {
        let order = self.state.get_order();
        let setter_state = Arc::clone(&self.state);
        let view_id = self.id;
        let rendering_context = Arc::clone(&self.rendering_context);
        let setter = Arc::new(move |value| {
            setter_state.set(order, value);
            if setter_state.is_dirty() {
                tracing::debug!("dirty state");
                rendering_context.enqueue(view_id);
            }
        });
        if let Some(value) = self.state.get(order) {
            (value, setter)
        } else {
            self.state.insert(initial_value);
            (self.state.get::<T>(order).unwrap(), setter)
        }
    }

    pub(crate) fn retrieve_last_render(&self) -> Node {
        let node = self.last_render.lock().unwrap().take().unwrap();
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
                        cx.handlers.insert(event_name.clone(), Arc::clone(handler));
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
        if let Some(handler) = self.handlers.get(&event) {
            handler();
        } else if let Some(cx) = self.event_registry.get(&event) {
            cx.dispatch_event(event);
        }
    }

    pub(crate) fn get_ordered(&self, order: usize) -> Arc<ViewContext> {
        self.registry.retrieve(order).unwrap()
    }
}
