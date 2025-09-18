use std::hash::Hash;
use std::sync::Arc;

use crate::context::Context;
use crate::view::{RenderableView, ViewRef};

pub trait ViewContext {
    fn create_view<V, F>(&self, factory: F) -> ViewRef
    where
        V: RenderableView + Send + Sync + 'static,
        F: Fn() -> V;

    fn use_state<T: Send + Sync + PartialEq + Clone + 'static>(
        &self,
        initial_value: T,
    ) -> (T, Arc<dyn Fn(T) + Send + Sync>);

    fn use_effect<F, T, C>(&self, f: F, deps: T)
    where
        F: (Fn() -> C) + Send + Sync + 'static,
        T: Hash,
        C: FnOnce() + Send + Sync + 'static;
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

    fn use_effect<F, T, C>(&self, f: F, deps: T)
    where
        F: (Fn() -> C) + Send + Sync + 'static,
        T: Hash,
        C: FnOnce() + Send + Sync + 'static,
    {
        self.use_effect(f, deps);
    }
}
