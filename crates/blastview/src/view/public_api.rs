use std::sync::Arc;

use crate::view::{RenderableView, ViewRef, context::Context};

pub trait ViewContext {
    fn create_view<V, F>(&self, factory: F) -> ViewRef
    where
        V: RenderableView + Send + Sync + 'static,
        F: Fn() -> V;

    fn use_state<T: Send + Sync + PartialEq + Clone + 'static>(
        &self,
        initial_value: T,
    ) -> (T, Arc<dyn Fn(T) + Send + Sync>);
}

impl ViewContext for Context {
    fn create_view<V, F>(&self, factory: F) -> ViewRef
    where
        V: RenderableView + Send + Sync + 'static,
        F: Fn() -> V,
    {
        self.create_view(factory)
    }

    fn use_state<T: Send + Sync + PartialEq + Clone + 'static>(
        &self,
        initial_value: T,
    ) -> (T, Arc<dyn Fn(T) + Send + Sync>) {
        self.use_state(initial_value)
    }
}
