use std::sync::{
    Arc, Mutex,
    atomic::{AtomicUsize, Ordering},
};

pub(crate) mod events;
mod public_api;
pub(crate) mod registry;
pub(crate) mod state;
pub use public_api::*;
pub mod macros;

use uuid::Uuid;

use crate::{
    context::{
        events::{Event, EventRegistry},
        registry::OrderedViewRegistry,
        state::StateRegistry,
    },
    node::Node,
    session::{RenderingQueue, context_registry::ContextRegistry},
    view::{RenderableView, ViewRef},
};

pub struct Context {
    pub(crate) id: Uuid,

    context_registry: Arc<ContextRegistry>,
    rendering_queue: Arc<RenderingQueue>,

    children: OrderedViewRegistry,
    view_registration_order: AtomicUsize,

    event_registry: EventRegistry,

    state_registry: Arc<StateRegistry>,
    state_registration_order: AtomicUsize,

    last_render: Mutex<Option<Arc<Node>>>,
    view: Arc<dyn RenderableView + Send + Sync>,
}

impl Context {
    pub fn new(
        view: Arc<dyn RenderableView + Send + Sync>,
        context_registry: Arc<ContextRegistry>,
        rendering_queue: Arc<RenderingQueue>,
    ) -> Arc<Self> {
        let id = Uuid::new_v4();

        let cx = Arc::new(Self {
            id,

            context_registry: Arc::clone(&context_registry),
            rendering_queue,

            children: OrderedViewRegistry::default(),
            view_registration_order: AtomicUsize::default(),

            event_registry: EventRegistry::default(),

            state_registry: Default::default(),
            state_registration_order: AtomicUsize::default(),

            last_render: Default::default(),
            view,
        });

        context_registry.register(id, Arc::clone(&cx));

        cx
    }

    pub(crate) fn create_view<V, F>(&self, factory: F) -> ViewRef
    where
        V: RenderableView + Send + Sync + 'static,
        F: Fn() -> V,
    {
        let order = self.view_registration_order.fetch_add(1, Ordering::Relaxed);

        if let Some(cx) = self.children.get(order) {
            cx.force_render();
            return ViewRef { order };
        }

        let view = Arc::new(factory());
        let context = Context::new(
            view,
            Arc::clone(&self.context_registry),
            Arc::clone(&self.rendering_queue),
        );

        context.force_render();

        self.children.register(context);

        ViewRef { order }
    }

    fn prepare_render(&self) {
        self.view_registration_order.store(0, Ordering::SeqCst);
        self.state_registration_order.store(0, Ordering::SeqCst);
        self.state_registry.mark_clean();
    }

    fn unregister_events(&self) {
        self.event_registry.clear();
    }

    pub(crate) fn render(&self) -> Arc<Node> {
        if let Some(last_render) = &*self.last_render.lock().unwrap() {
            return Arc::clone(last_render);
        }

        self.force_render()
    }

    pub(crate) fn force_render(&self) -> Arc<Node> {
        let mut last_render = self.last_render.lock().unwrap();

        self.prepare_render();
        // for now, atomic node event operations are not possible - diffing is not yet implemented
        self.unregister_events();

        let tree = self.view.render(self);
        self.register_events(&tree);
        let tree = Arc::new(tree);
        *last_render = Some(Arc::clone(&tree));
        tree
    }

    pub(crate) fn use_state<T: Send + Sync + PartialEq + Clone + 'static>(
        &self,
        initial_value: T,
    ) -> (T, Arc<dyn Fn(T) + Send + Sync>) {
        let order = self
            .state_registration_order
            .fetch_add(1, Ordering::Relaxed);

        let state_registry = Arc::clone(&self.state_registry);
        let rendering_queue = Arc::clone(&self.rendering_queue);
        let id = self.id;
        let update: Arc<dyn Fn(T) + Send + Sync> = Arc::new(move |value| {
            if state_registry.update(order, value) {
                rendering_queue.enqueue(id);
            }
        });

        if let Some(state) = self.state_registry.get::<T>(order) {
            return (state.as_any().downcast_ref::<T>().unwrap().clone(), update);
        }

        self.state_registry.register(initial_value.clone());

        (initial_value, update)
    }

    fn register_events(&self, node: &Node) {
        match node {
            Node::Element(node) => {
                for (event, handler) in node.events.iter() {
                    let event = Event {
                        node_id: node.id,
                        event: event.to_string(),
                    };
                    tracing::debug!("[{}] event registered: {event:?}", node.tag);
                    self.event_registry
                        .register(event.clone(), Arc::clone(handler));
                }

                for child in node.children.iter() {
                    self.register_events(child);
                }
            }
            // ignore text elements
            // child views have already registered their own events
            _ => {}
        }
    }

    pub(crate) fn dispatch_event(&self, event: &Event) {
        if let Some(handler) = self.event_registry.get(event) {
            handler();
        }

        self.children.each(|cx| cx.dispatch_event(&event));
    }

    pub(crate) fn get_child(&self, idx: usize) -> Option<Arc<Context>> {
        self.children.get(idx)
    }
}
