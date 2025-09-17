pub(crate) mod context_registry;
pub mod patch;

use std::{
    sync::{
        Arc, Mutex,
        atomic::{AtomicU64, Ordering},
    },
    time::UNIX_EPOCH,
};

use tokio::sync::Notify;
use uuid::Uuid;

use crate::{
    renderer::Renderer,
    session::{context_registry::ContextRegistry, patch::Patch},
    view::{View, context::ViewContext},
};

#[derive(Default)]
pub(crate) struct RenderingContext {
    render_queue: Mutex<Vec<Uuid>>,
}

impl RenderingContext {
    pub(crate) fn enqueue(&self, id: Uuid) {
        self.render_queue.lock().unwrap().push(id);
    }
}

pub struct LiveSession {
    context: Arc<ViewContext>,
    rendering_context: Arc<RenderingContext>,
    context_registry: Arc<ContextRegistry>,
    renderer: Renderer,
    re_render_notifier: Arc<Notify>,
    last_re_render_time: AtomicU64,
    patch_sender: flume::Sender<Patch>,
}

impl LiveSession {
    pub fn new<V, F>(factory: F) -> (Self, flume::Receiver<Patch>)
    where
        V: View + Send + Sync + 'static,
        F: Fn() -> V + Send + Sync,
    {
        let events_registry = Default::default();
        let rendering_context = Default::default();
        let context_registry = Default::default();
        let context = ViewContext::new(
            Arc::clone(&events_registry),
            Arc::clone(&rendering_context),
            Arc::clone(&context_registry),
        );
        let root_view = context.create(factory);

        let context = Arc::new(context);

        let renderer = Renderer::new(Arc::clone(&context), root_view);

        let (patch_tx, patch_rx) = flume::unbounded();

        (
            Self {
                context,
                renderer,
                re_render_notifier: Default::default(),
                last_re_render_time: Default::default(),
                rendering_context,
                context_registry,
                patch_sender: patch_tx,
            },
            patch_rx,
        )
    }

    pub fn dispatch_event(&self, event: String) {
        self.context.dispatch_event(event);
    }

    pub fn dynamic_render(&self) -> String {
        self.context.prepare();
        self.renderer.render_to_string()
    }

    async fn process_re_render_queue(&self) {
        if self
            .rendering_context
            .render_queue
            .lock()
            .unwrap()
            .is_empty()
        {
            return;
        }

        for view_id in self
            .rendering_context
            .render_queue
            .lock()
            .unwrap()
            .drain(..)
        {
            let cx = self.context_registry.get(&view_id).unwrap();
            Arc::clone(&cx).trigger_render();
            let tree = cx.retrieve_last_render();
            let view_string = self.renderer.render_node_to_string(tree, &cx);
            self.patch_sender
                .send(Patch::ReplaceInner {
                    selector: format!(r#"bv-view[data-view="{}"]"#, cx.id),
                    html: view_string,
                })
                .unwrap();
        }

        self.last_re_render_time.store(
            std::time::SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64,
            Ordering::Relaxed,
        );
    }

    pub fn begin_re_render_task(self: Arc<Self>) {
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_millis(16));

            loop {
                tokio::select! {
                    // Process on notification (immediate updates)
                    _ = self.re_render_notifier.notified() => {
                        self.process_re_render_queue().await;
                    }
                    // Process on interval (fallback for batched updates)
                    _ = interval.tick() => {
                        self.process_re_render_queue().await;
                    }
                }
            }
        });
    }
}
