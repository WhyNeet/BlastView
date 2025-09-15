use blastview::{node::Node, view::View};
use tracing_subscriber::{
    filter::{EnvFilter, LevelFilter},
    fmt,
    layer::SubscriberExt,
    util::SubscriberInitExt,
};

#[tokio::main]
async fn main() -> std::io::Result<()> {
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(
            EnvFilter::builder()
                .with_default_directive(LevelFilter::INFO.into())
                .from_env()
                .unwrap(),
        )
        .init();

    blaster::serve(|| MyView).await
}

struct MyView;

impl View for MyView {
    fn render(&self, _: &blastview::view::context::ViewContext) -> impl Into<Node> {
        Node::new("div")
            .attr("class", "container")
            .child(my_component())
    }
}

fn my_component() -> impl Into<Node> {
    Node::new("div")
        .child("Hello world!")
        .child(Node::new("hr"))
        .child(Node::new("strong").child("BlastView works"))
}
