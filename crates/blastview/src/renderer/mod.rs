use std::sync::Arc;

use crate::{
    node::{ElementNode, Node},
    view::{context::ViewContext, registry::ViewRef},
};

pub struct Renderer {
    root_context: Arc<ViewContext>,
    root_view: ViewRef,
}

impl Renderer {
    pub fn new(root_context: Arc<ViewContext>, root_view: ViewRef) -> Self {
        Self {
            root_context,
            root_view,
        }
    }
}

impl Renderer {
    pub fn render_to_string(&self) -> String {
        self.render_view_to_string(self.root_view, &self.root_context)
    }

    fn render_view_to_string(&self, view_ref: ViewRef, cx: &ViewContext) -> String {
        let cx = cx.get_ordered(view_ref.order);
        let node = Arc::clone(&cx).retrieve_last_render();

        format!(
            r#"<bv-view data-view="{}">{}</bv-view>"#,
            cx.id,
            self.render_node_to_string(node, &cx)
        )
    }

    pub fn render_node_to_string(&self, node: Node, cx: &ViewContext) -> String {
        match node {
            Node::Element(node) => self.render_element_node_to_string(*node, cx),
            Node::Text(text) => html_escape::encode_text(&text.0).to_string(),
            Node::ViewRef(view) => self.render_view_to_string(*view, cx),
        }
    }

    fn render_element_node_to_string(&self, node: ElementNode, cx: &ViewContext) -> String {
        let mut buffer = String::new();

        buffer.push('<');
        buffer.push_str(&node.tag);

        for (attr, value) in node.attrs.iter() {
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
                    .into_keys()
                    .reduce(|acc, s| format!("{acc},{s}"))
                    .unwrap(),
            );
            buffer.push('"');
        }

        buffer.push('>');

        for child in node.children {
            buffer.push_str(&self.render_node_to_string(child, cx));
        }

        buffer.push('<');
        buffer.push('/');
        buffer.push_str(&node.tag);
        buffer.push('>');

        buffer
    }
}
