use blastview::{context::ViewContext, node::Node, view::View};
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
    fn render(&self, _: &impl ViewContext) -> impl Into<Node> {
        Node::new("div")
            .attr("class", "container")
            .child(my_component())
    }
}

fn my_component() -> impl Into<Node> {
    Node::new("div")
        .child(r#"<script>alert("oh hey there")</script>"#)
        .child(
            Node::new("div")
                .attr("data-whatever", r#"bad "" attribute ""#)
                .child("This div has a bad attribute"),
        )
        .child(
            Node::new("div")
                .child("This one's onclick attribute was removed.")
                .attr("onclick", r#"alert("hii")"#),
        )
}
