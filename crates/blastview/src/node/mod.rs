use uuid::Uuid;

use crate::view::ViewRef;
use std::{
    collections::HashMap,
    fmt::{Debug, Display},
    sync::Arc,
};

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
pub struct TextNode(pub(crate) String);

#[derive(Clone)]
pub struct ElementNode {
    pub(crate) id: Uuid,
    pub(crate) tag: String,
    pub(crate) attrs: HashMap<String, String>,
    pub(crate) events: HashMap<String, Arc<dyn Fn() + Send + Sync>>,
    pub(crate) children: Vec<Node>,
}

impl Debug for ElementNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ElementNode")
            .field("id", &self.id)
            .field("tag", &self.tag)
            .field("attrs", &self.attrs)
            .field(
                "events",
                &self.events.iter().map(|(name, _)| name).collect::<Vec<_>>(),
            )
            .field("children", &self.children)
            .finish()
    }
}

impl ElementNode {
    pub fn new(tag: &str) -> Self {
        Self {
            id: Uuid::new_v4(),
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
        self.events.insert(event.to_string(), Arc::new(handler));
        self
    }

    pub fn child(mut self, node: impl Into<Node>) -> Self {
        self.children.push(node.into());
        self
    }
}

impl<T: Display> From<T> for TextNode {
    fn from(value: T) -> Self {
        Self(value.to_string())
    }
}

impl<T: Display> From<T> for Node {
    fn from(value: T) -> Self {
        Self::Text(Box::new(TextNode(value.to_string())))
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

pub trait RenderableElement {
    fn id(&self) -> Uuid;
    fn tag(&self) -> &str;
    fn attrs(&self) -> &HashMap<String, String>;
    fn events(&self) -> &HashMap<String, Arc<dyn Fn() + Send + Sync>>;
    fn children(&self) -> &[Node];
}

impl RenderableElement for ElementNode {
    fn id(&self) -> Uuid {
        self.id
    }

    fn tag(&self) -> &str {
        &self.tag
    }

    fn attrs(&self) -> &HashMap<String, String> {
        &self.attrs
    }

    fn events(&self) -> &HashMap<String, Arc<dyn Fn() + Send + Sync>> {
        &self.events
    }

    fn children(&self) -> &[Node] {
        &self.children
    }
}

pub trait RenderableText {
    fn text(&self) -> &str;
}

impl RenderableText for TextNode {
    fn text(&self) -> &str {
        &self.0
    }
}
