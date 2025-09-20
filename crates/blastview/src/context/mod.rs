use std::{
    hash::Hash,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, AtomicUsize, Ordering},
    },
};

pub(crate) mod diffing;
pub(crate) mod effects;
pub mod events;
mod public_api;
pub(crate) mod registry;
pub(crate) mod state;
pub use public_api::*;
pub mod context_registry;
pub mod macros;

use uuid::Uuid;

use crate::{
    context::{
        context_registry::ContextRegistry,
        diffing::diff,
        effects::{Effect, EffectRegistry},
        events::{Event, EventRegistry},
        registry::OrderedViewRegistry,
        state::StateRegistry,
    },
    node::{ElementNode, Node},
    rendering::RenderingQueue,
    view::{RenderableView, ViewRef},
};
pub use diffing::NodePatch;

pub struct Context {
    pub id: Uuid,

    context_registry: Arc<ContextRegistry>,
    rendering_queue: Arc<RenderingQueue>,

    children: OrderedViewRegistry,
    view_registration_order: AtomicUsize,

    event_registry: EventRegistry,

    state_registry: Arc<StateRegistry>,
    state_registration_order: AtomicUsize,

    effect_registry: EffectRegistry,
    effect_registration_order: AtomicUsize,

    has_rendered: AtomicBool,
    last_render: Mutex<Option<Node>>,
    diff: Mutex<Option<Vec<NodePatch>>>,
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

            effect_registry: EffectRegistry::default(),
            effect_registration_order: AtomicUsize::default(),

            has_rendered: AtomicBool::new(false),
            last_render: Default::default(),
            diff: Default::default(),
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
        self.effect_registration_order.store(0, Ordering::SeqCst);
        self.state_registry.mark_clean();
    }

    fn unregister_events(&self) {
        self.event_registry.clear();
    }

    pub fn render(&self) -> Vec<NodePatch> {
        if self.last_render.lock().unwrap().is_some() {
            return vec![];
        }

        self.force_render()
    }

    pub fn force_render(&self) -> Vec<NodePatch> {
        let mut last_render = self.last_render.lock().unwrap();

        self.prepare_render();
        // for now, atomic node event operations are not possible - diffing is not yet implemented
        self.unregister_events();

        let mut tree = self.view.render(self);
        let patches = if self.has_rendered.swap(true, Ordering::Relaxed) {
            let diff = diff(last_render.take().unwrap(), &mut tree, self.id, 0);
            *self.diff.lock().unwrap() = Some(diff.clone());
            diff
        } else {
            vec![NodePatch::ReplaceViewChildren {
                view_id: self.id,
                children: vec![tree.clone()],
            }]
        };
        self.register_events(&tree);
        *last_render = Some(tree);

        patches
    }

    pub(crate) fn use_state<T: Send + Sync + PartialEq + Clone + 'static>(
        &self,
        initial_value: T,
    ) -> (T, Arc<dyn Fn(T) + Send + Sync>) {
        self.use_state_factory(|| initial_value)
    }

    pub(crate) fn use_state_factory<
        T: Send + Sync + PartialEq + Clone + 'static,
        F: FnOnce() -> T,
    >(
        &self,
        factory: F,
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

        let initial_value = factory();

        self.state_registry.register(initial_value.clone());

        (initial_value, update)
    }

    pub(crate) fn use_effect<F, T, C>(&self, f: F, deps: T)
    where
        F: (FnOnce() -> C) + Send + Sync,
        T: Hash,
        C: FnOnce() + Send + Sync + 'static,
    {
        let order = self
            .effect_registration_order
            .fetch_add(1, Ordering::Relaxed);

        if let Some(effect) = self.effect_registry.get(order) {
            if self.effect_registry.update_deps(order, &deps) {
                effect.run(move || Box::new(f()));
            }
            return;
        }

        let effect = Effect::new(move || Box::new(f()), deps);
        self.effect_registry.register(effect);
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

    pub fn dispatch_event(&self, event: &Event) {
        self.event_registry.handle(event);

        self.children.each(|cx| cx.dispatch_event(&event));
    }

    pub fn get_child(&self, idx: usize) -> Option<Arc<Context>> {
        self.children.get(idx)
    }

    pub fn view_node(&self) -> ElementNode {
        Node::new("bv-view").attr("data-view", &self.id.to_string())
    }

    pub fn with_last_render<R>(&self, f: impl FnOnce(Option<&Node>) -> R) -> R {
        f(self.last_render.lock().unwrap().as_ref())
    }
}

impl Drop for Context {
    fn drop(&mut self) {
        self.effect_registry.clear();
        self.event_registry.clear();
        self.state_registry.clear();
    }
}
