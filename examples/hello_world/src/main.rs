use blastview::{component::Component, node::Node};

#[tokio::main]
async fn main() -> std::io::Result<()> {
    blaster::serve(MyComponent).await
}

struct MyComponent;

impl Component for MyComponent {
    fn render(&self) -> impl Into<Node> {
        Node::new("div")
            .attr("class", "container")
            .child("Hello world!")
            .child(Node::new("hr"))
            .child(Node::new("strong").child("BlastView works"))
    }
}
