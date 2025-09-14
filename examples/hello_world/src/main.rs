use blastview::{node::Node, view::View};

#[tokio::main]
async fn main() -> std::io::Result<()> {
    blaster::serve(|| MyView).await
}

struct MyView;

impl View for MyView {
    fn render(&self, cx: &blastview::view::context::ViewContext) -> impl Into<Node> {
        let counter = cx.create(|| CounterView);

        Node::new("div")
            .attr("class", "container")
            .child(my_component())
            .child(counter)
    }
}

struct CounterView;

impl View for CounterView {
    fn render(&self, cx: &blastview::view::context::ViewContext) -> impl Into<Node> {
        Node::new("button")
            .on("click", || {
                println!("increment");
            })
            .child(format!("Count: 0"))
    }
}

fn my_component() -> impl Into<Node> {
    Node::new("div")
        .child("Hello world!")
        .child(Node::new("hr"))
        .child(Node::new("strong").child("BlastView works"))
}
