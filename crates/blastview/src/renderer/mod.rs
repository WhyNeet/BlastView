use crate::{
    node::{ElementNode, Node},
    view::{View, context::ViewContext, registry::ViewRef},
};

pub struct Renderer {}

impl Renderer {
    pub fn render_to_string<V, F>(factory: F) -> String
    where
        V: View + Send + Sync + 'static,
        F: Fn() -> V,
    {
        let root_cx = ViewContext::new(0, Default::default());
        let view_ref = root_cx.create(factory);

        Self::render_view_to_string(view_ref, &root_cx)
    }

    fn render_view_to_string(view_ref: ViewRef, cx: &ViewContext) -> String {
        cx.prepare();
        let (cx, view) = cx.get_ordered(view_ref.id);
        let node = view.render(cx.as_ref());

        Self::render_node_to_string(node, &cx)
    }

    fn render_node_to_string(node: Node, cx: &ViewContext) -> String {
        match node {
            Node::Element(node) => Self::render_element_node_to_string(*node, cx),
            Node::Text(text) => text.0,
            Node::ViewRef(view) => Self::render_view_to_string(*view, cx),
        }
    }

    fn render_element_node_to_string(node: ElementNode, cx: &ViewContext) -> String {
        let mut buffer = String::new();

        buffer.push('<');
        buffer.push_str(&node.tag);

        for (attr, value) in node.attrs.iter() {
            buffer.push(' ');
            buffer.push_str(attr);
            buffer.push('=');
            buffer.push('"');
            buffer.push_str(value);
            buffer.push('"');
        }

        buffer.push('>');

        for child in node.children {
            buffer.push_str(&Self::render_node_to_string(child, cx));
        }

        buffer.push('<');
        buffer.push('/');
        buffer.push_str(&node.tag);
        buffer.push('>');

        buffer
    }
}
