use std::sync::Arc;

use crate::{
    renderer::Renderer,
    view::{View, context::ViewContext, events::GlobalEventsRegistry},
};

pub struct LiveSession {
    context: Arc<ViewContext>,
    renderer: Renderer,
    events_registry: Arc<GlobalEventsRegistry>,
}

impl LiveSession {
    pub fn new<V, F>(factory: F) -> Self
    where
        V: View + Send + Sync + 'static,
        F: Fn() -> V + Send + Sync,
    {
        let events_registry = Default::default();
        let context = ViewContext::new(0, Arc::clone(&events_registry));
        let root_view = context.create(factory);

        let context = Arc::new(context);

        let renderer = Renderer::new(Arc::clone(&context), root_view);

        Self {
            context,
            renderer,
            events_registry,
        }
    }

    pub fn dispatch_event(&self, event: String) {
        self.context.dispatch_event(event);
    }

    pub fn dynamic_render(&self) -> String {
        self.context.prepare();
        self.renderer.render_to_string()
    }

    pub fn hydration_script(&self) -> String {
        let mut listeners = String::new();

        self.events_registry.all_events(|name| {
            let (node_id, event_name) = name.split_once('_').unwrap();
            listeners.push_str(&format!(
                r#"document.querySelector("[data-id='{node_id}']").addEventListener("{event_name}",()=>ws.send("{name}"));"#,
            ));
        });

        format!(r#"<script>console.log("init!");{}</script>"#, listeners)
    }
}
