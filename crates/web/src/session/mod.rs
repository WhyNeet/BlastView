pub mod patch;

use std::sync::Arc;

use blastview::{
    context::{Context, NodePatch, context_registry::ContextRegistry, events::Event},
    rendering::RenderingQueue,
    view::View,
};
use tokio::sync::Notify;
use tokio_util::sync::CancellationToken;

use crate::{Renderer, session::patch::Patch};

pub struct LiveSession {
    context: Arc<Context>,
    renderer: Renderer,
    rendering_queue: Arc<RenderingQueue>,
    context_registry: Arc<ContextRegistry>,
    re_render_notifier: Arc<Notify>,
    patch_sender: flume::Sender<Patch>,
    stop_re_render_task: CancellationToken,
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
                stop_re_render_task: CancellationToken::new(),
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
        if self.rendering_queue.render_queue.lock().unwrap().is_empty()
            && self
                .rendering_queue
                .deferred_queue
                .lock()
                .unwrap()
                .is_empty()
        {
            return;
        }

        let process_view = |view_id: uuid::Uuid| {
            if self.patch_sender.is_disconnected() {
                return;
            }
            let cx = self.context_registry.get(&view_id).unwrap();
            let patches = cx.force_render();
            let patches = patches
                .into_iter()
                .map(|patch| match patch {
                    NodePatch::ReplaceViewChildren { view_id, children } => Patch::ReplaceInner {
                        selector: format!(r#"bv-view[data-view="{view_id}"]"#),
                        html: children
                            .into_iter()
                            .map(|child| self.renderer.render_node_to_string(&child, &cx))
                            .collect(),
                    },
                    NodePatch::ReplaceChildren { node_id, children } => Patch::ReplaceInner {
                        selector: format!(r#"[data-id="{node_id}"]"#),
                        html: children
                            .into_iter()
                            .map(|child| self.renderer.render_node_to_string(&child, &cx))
                            .collect(),
                    },
                    NodePatch::ReplaceChild {
                        node_id,
                        child_idx,
                        node,
                    } => Patch::ReplaceChild {
                        selector: format!(r#"[data-id="{node_id}"]"#),
                        index: child_idx,
                        html: self.renderer.render_node_to_string(&node, &cx),
                    },
                    NodePatch::Replace { node_id, node } => Patch::ReplaceOuter {
                        selector: format!(r#"[data-id="{node_id}"]"#),
                        html: self.renderer.render_node_to_string(&node, &cx),
                    },
                    NodePatch::SetAttr {
                        node_id,
                        attr,
                        value,
                    } => Patch::SetAttribute {
                        selector: format!(r#"[data-id="{node_id}"]"#),
                        name: attr,
                        value,
                    },
                    NodePatch::RemoveAttr { node_id, attr } => Patch::RemoveAttribute {
                        selector: format!(r#"[data-id="{node_id}"]"#),
                        name: attr,
                    },
                    NodePatch::AttachEvent { node_id, event } => Patch::AttachEvent {
                        selector: format!(r#"[data-id="{node_id}"]"#),
                        event,
                    },
                    NodePatch::DetachEvent { node_id, event } => Patch::DetachEvent {
                        selector: format!(r#"[data-id="{node_id}"]"#),
                        event,
                    },
                })
                .collect();

            if self.patch_sender.is_disconnected() {
                return;
            }
            self.patch_sender.send(Patch::Batch { patches }).unwrap();
        };

        for view_id in self.rendering_queue.render_queue.lock().unwrap().drain() {
            process_view(view_id);
        }

        for view_id in self.rendering_queue.deferred_queue.lock().unwrap().drain() {
            process_view(view_id);
        }
    }

    pub fn stop_re_render_task(&self) {
        self.stop_re_render_task.cancel();
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
                    _ = self.stop_re_render_task.cancelled() => {
                      break;
                    }
                }
            }
        });
    }
}

impl Drop for LiveSession {
    fn drop(&mut self) {
        self.context_registry.clear();
        self.rendering_queue.clear();
    }
}
