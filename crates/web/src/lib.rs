pub mod session;

use std::sync::Arc;

use blastview::{
    context::Context,
    node::{ElementNode, Node, RenderableElement, RenderableText},
};

pub struct Renderer {
    root_context: Arc<Context>,
}

impl Renderer {
    pub fn new(root_context: Arc<Context>) -> Self {
        Self { root_context }
    }
}

impl Renderer {
    pub fn render_to_string(&self) -> String {
        self.render_view_to_string(&self.root_context)
    }

    fn render_view_to_string(&self, cx: &Context) -> String {
        cx.render();

        let node = cx.with_last_render(|node| cx.view_node().child(node.unwrap().clone()).into());

        self.render_node_to_string(&node, &cx)
    }

    pub fn render_node_to_string(&self, node: &Node, cx: &Context) -> String {
        match node {
            Node::Element(node) => self.render_element_node_to_string(&node, cx),
            Node::Text(text) => {
                html_escape::encode_text(RenderableText::text(text.as_ref())).to_string()
            }
            Node::ViewRef(view) => self.render_view_to_string(&cx.get_child(view.order).unwrap()),
        }
    }

    fn render_element_node_to_string(&self, node: &ElementNode, cx: &Context) -> String {
        let mut buffer = String::new();

        buffer.push('<');
        buffer.push_str(RenderableElement::tag(node));

        for (attr, value) in RenderableElement::attrs(node).iter() {
            if attr.starts_with("on") {
                continue;
            }
            buffer.push(' ');
            buffer.push_str(attr);
            buffer.push('=');
            buffer.push('"');
            buffer.push_str(&html_escape::encode_quoted_attribute(value).to_string());
            buffer.push('"');
        }

        buffer.push(' ');
        buffer.push_str("data-id");
        buffer.push('=');
        buffer.push('"');
        buffer.push_str(&RenderableElement::id(node).to_string());
        buffer.push('"');
        if !RenderableElement::events(node).is_empty() {
            buffer.push(' ');
            buffer.push_str("data-events");
            buffer.push('=');
            buffer.push('"');
            buffer.push_str(
                &RenderableElement::events(node)
                    .keys()
                    .cloned()
                    .reduce(|acc, s| format!("{acc},{s}"))
                    .unwrap(),
            );
            buffer.push('"');
        }

        buffer.push('>');

        for child in RenderableElement::children(node).iter() {
            buffer.push_str(&self.render_node_to_string(child, cx));
        }

        buffer.push('<');
        buffer.push('/');
        buffer.push_str(RenderableElement::tag(node));
        buffer.push('>');

        buffer
    }
}
