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

    blaster::serve(|| CounterView).await
}

struct CounterView;

impl View for CounterView {
    fn render(&self, cx: &blastview::view::context::ViewContext) -> impl Into<Node> {
        let (count, set_count) = cx.use_state(0);

        Node::new("div").attr("class", "container").child(
            Node::new("button")
                .child(format!("count: {count}"))
                .on("click", move || set_count(count + 1)),
        )
    }
}
