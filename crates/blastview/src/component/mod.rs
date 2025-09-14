use crate::node::Node;

pub trait Component {
    fn render(&self) -> impl Into<Node>;
}
