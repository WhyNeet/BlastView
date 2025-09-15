use std::sync::Arc;

use crate::view::{View, context::ViewContext};

pub struct LiveSession {
    context: ViewContext,
}

impl LiveSession {
    pub fn new<V, F>(factory: F) -> Self
    where
        V: View + Send + Sync + 'static,
        F: Fn() -> V + Send + Sync,
    {
        let event_registry = Default::default();
        let context = ViewContext::new(0, Arc::clone(&event_registry));

        context.create(factory);

        Self { context }
    }

    pub fn context(&self) -> &ViewContext {
        &self.context
    }

    pub fn dispatch_event(&self, event: String) {
        self.context.dispatch_event(event);
    }
}
