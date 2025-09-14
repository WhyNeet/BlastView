use std::collections::HashMap;

pub enum Node {
    Text(Box<TextNode>),
    Element(Box<ElementNode>),
}

impl Node {
    pub fn new(tag: &str) -> ElementNode {
        ElementNode::new(tag)
    }

    pub fn text(text: &str) -> TextNode {
        TextNode::new(text)
    }
}

pub struct TextNode {
    pub(crate) text: String,
}

pub struct ElementNode {
    pub(crate) tag: String,
    pub(crate) attrs: HashMap<String, String>,
    pub(crate) events: HashMap<String, Box<dyn Fn()>>,
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
        F: Fn() + 'static,
    {
        self.events.insert(event.to_string(), Box::new(handler));
        self
    }

    pub fn child(mut self, node: impl Into<Node>) -> Self {
        self.children.push(node.into());
        self
    }
}

impl TextNode {
    pub fn new(text: &str) -> Self {
        Self {
            text: text.to_string(),
        }
    }
}

impl Into<Node> for &str {
    fn into(self) -> Node {
        Node::Text(Box::new(TextNode::new(self)))
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
