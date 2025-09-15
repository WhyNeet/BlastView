use crate::{node::Node, view::context::ViewContext};

pub mod context;
pub(crate) mod events;
pub(crate) mod registry;

pub trait RenderableView {
    fn render(&self, cx: &ViewContext) -> Node;
}

pub trait View: RenderableView {
    fn render(&self, cx: &ViewContext) -> impl Into<Node>;
}

impl<V: View> RenderableView for V {
    fn render(&self, cx: &ViewContext) -> Node {
        <V as View>::render(&self, cx).into()
    }
}
