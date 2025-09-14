use crate::node::Node;

pub trait Component {
    fn render() -> impl Into<Node>;
}
