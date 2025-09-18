use crate::{
    context::{Context, ViewContext},
    node::Node,
};

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
