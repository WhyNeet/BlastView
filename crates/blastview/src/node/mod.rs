use std::collections::HashMap;

use crate::view::registry::ViewRef;

pub enum Node {
    Text(Box<TextNode>),
    Element(Box<ElementNode>),
    ViewRef(Box<ViewRef>),
}

impl Node {
    pub fn new(tag: &str) -> ElementNode {
        ElementNode::new(tag)
    }

    pub fn text(text: &str) -> TextNode {
        text.into()
    }
}

pub struct TextNode(pub(crate) String);

pub struct ElementNode {
    pub(crate) tag: String,
    pub(crate) attrs: HashMap<String, String>,
    pub(crate) events: HashMap<String, Box<dyn Fn() + Send + Sync>>,
    pub(crate) children: Vec<Node>,
}

impl ElementNode {
    pub fn new(tag: &str) -> Self {
        Self {
            tag: tag.to_string(),
            attrs: Default::default(),
            events: Default::default(),
            children: Default::default(),
        }
    }

    pub fn attr(mut self, attr: &str, val: &str) -> Self {
        self.attrs.insert(attr.to_string(), val.to_string());
        self
    }

    pub fn on<F>(mut self, event: &str, handler: F) -> Self
    where
        F: Fn() + Send + Sync + 'static,
    {
        self.events.insert(event.to_string(), Box::new(handler));
        self
    }

    pub fn child(mut self, node: impl Into<Node>) -> Self {
        self.children.push(node.into());
        self
    }
}

impl Into<TextNode> for &str {
    fn into(self) -> TextNode {
        TextNode(self.to_string())
    }
}

impl Into<TextNode> for String {
    fn into(self) -> TextNode {
        TextNode(self)
    }
}

impl Into<Node> for &str {
    fn into(self) -> Node {
        TextNode(self.to_string()).into()
    }
}

impl Into<Node> for String {
    fn into(self) -> Node {
        TextNode(self).into()
    }
}

impl Into<Node> for TextNode {
    fn into(self) -> Node {
        Node::Text(Box::new(self))
    }
}

impl Into<Node> for ElementNode {
    fn into(self) -> Node {
        Node::Element(Box::new(self))
    }
}

impl Into<Node> for ViewRef {
    fn into(self) -> Node {
        Node::ViewRef(Box::new(self))
    }
}
