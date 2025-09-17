use crate::{node::Node, view::context::Context};

pub mod context;
pub(crate) mod events;
mod public_api;
pub(crate) mod registry;
pub(crate) mod state;
pub use public_api::*;

pub trait RenderableView {
    fn render(&self, cx: &Context) -> Node;
}

pub trait View: RenderableView {
    fn render(&self, cx: &impl ViewContext) -> impl Into<Node>;
}

impl<V: View> RenderableView for V {
    fn render(&self, cx: &Context) -> Node {
        <V as View>::render(&self, cx).into()
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ViewRef {
    pub(crate) order: usize,
}
