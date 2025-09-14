use crate::{
    component::Component,
    node::{ElementNode, Node},
};

pub fn render_component_to_string<C>() -> String
where
    C: Component,
{
    render_node_to_string(C::render().into())
}

fn render_node_to_string(node: Node) -> String {
    match node {
        Node::Element(node) => render_element_node_to_string(*node),
        Node::Text(text) => text.text,
    }
}

fn render_element_node_to_string(node: ElementNode) -> String {
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
        buffer.push_str(&render_node_to_string(child));
    }

    buffer.push('<');
    buffer.push('/');
    buffer.push_str(&node.tag);
    buffer.push('>');

    buffer
}
