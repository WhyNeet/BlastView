pub(crate) mod context_registry;
pub mod patch;

use std::sync::{Arc, Mutex};

use tokio::sync::Notify;
use uuid::Uuid;

use crate::{
    context::{Context, events::Event},
    renderer::Renderer,
    session::{context_registry::ContextRegistry, patch::Patch},
    view::View,
};

#[derive(Default)]
pub struct RenderingQueue {
    render_queue: Mutex<Vec<Uuid>>,
}

impl RenderingQueue {
    pub(crate) fn enqueue(&self, id: Uuid) {
        self.render_queue.lock().unwrap().push(id);
    }
}

pub struct LiveSession {
    context: Arc<Context>,
    renderer: Renderer,
    rendering_queue: Arc<RenderingQueue>,
    context_registry: Arc<ContextRegistry>,
    re_render_notifier: Arc<Notify>,
    patch_sender: flume::Sender<Patch>,
}

impl LiveSession {
    pub fn new<V, F>(factory: F) -> (Self, flume::Receiver<Patch>)
    where
        V: View + Send + Sync + 'static,
        F: Fn() -> V + Send + Sync,
    {
        let rendering_queue = Default::default();
        let context_registry = Default::default();
        let context = Context::new(
            Arc::new(factory()),
            Arc::clone(&context_registry),
            Arc::clone(&rendering_queue),
        );
        let renderer = Renderer::new(Arc::clone(&context));

        let (patch_tx, patch_rx) = flume::unbounded();

        (
            Self {
                context,
                renderer,
                rendering_queue,
                context_registry,
                re_render_notifier: Default::default(),
                patch_sender: patch_tx,
            },
            patch_rx,
        )
    }

    pub fn dispatch_event(&self, event: String) {
        let (node_id, event) = event.split_at(36);
        let event = Event {
            event: event[1..].to_string(),
            node_id: node_id.parse().unwrap(),
        };
        self.context.dispatch_event(&event);
    }

    pub fn dynamic_render(&self) -> String {
        self.renderer.render_to_string()
    }

    async fn process_re_render_queue(&self) {
        if self.rendering_queue.render_queue.lock().unwrap().is_empty() {
            return;
        }

        for view_id in self.rendering_queue.render_queue.lock().unwrap().drain(..) {
            if self.patch_sender.is_disconnected() {
                return;
            }
            let cx = self.context_registry.get(&view_id).unwrap();
            let tree = cx.force_render();
            let view_string = self.renderer.render_node_to_string(&tree, &cx);
            self.patch_sender
                .send(Patch::ReplaceInner {
                    selector: format!(r#"bv-view[data-view="{}"]"#, cx.id),
                    html: view_string,
                })
                .unwrap();
        }
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
