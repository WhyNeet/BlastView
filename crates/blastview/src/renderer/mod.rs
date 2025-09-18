use std::sync::Arc;

use crate::{
    context::Context,
    node::{ElementNode, Node},
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
        let node = cx.render();

        format!(
            r#"<bv-view data-view="{}">{}</bv-view>"#,
            cx.id,
            self.render_node_to_string(&node, &cx)
        )
    }

    pub fn render_node_to_string(&self, node: &Node, cx: &Context) -> String {
        match node {
            Node::Element(node) => self.render_element_node_to_string(&node, cx),
            Node::Text(text) => html_escape::encode_text(&text.0).to_string(),
            Node::ViewRef(view) => self.render_view_to_string(&cx.get_child(view.order).unwrap()),
        }
    }

    fn render_element_node_to_string(&self, node: &ElementNode, cx: &Context) -> String {
        let mut buffer = String::new();

        buffer.push('<');
        buffer.push_str(&node.tag);

        for (attr, value) in node.attrs.iter() {
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

        if !node.events.is_empty() {
            buffer.push(' ');
            buffer.push_str("data-id");
            buffer.push('=');
            buffer.push('"');
            buffer.push_str(&node.id.to_string());
            buffer.push('"');

            buffer.push(' ');
            buffer.push_str("data-events");
            buffer.push('=');
            buffer.push('"');
            buffer.push_str(
                &node
                    .events
                    .keys()
                    .cloned()
                    .reduce(|acc, s| format!("{acc},{s}"))
                    .unwrap(),
            );
            buffer.push('"');
        }

        buffer.push('>');

        for child in node.children.iter() {
            buffer.push_str(&self.render_node_to_string(child, cx));
        }

        buffer.push('<');
        buffer.push('/');
        buffer.push_str(&node.tag);
        buffer.push('>');

        buffer
    }
}
