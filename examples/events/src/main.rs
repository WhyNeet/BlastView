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

    blaster::serve(|| EventView).await
}

struct EventView;

impl View for EventView {
    fn render(&self, _: &impl ViewContext) -> impl Into<Node> {
        Node::new("div")
            .attr("class", "container")
            .child(click_event_button())
            .child(double_click_event_button())
    }
}

fn click_event_button() -> impl Into<Node> {
    Node::new("button")
        .child("I send click events")
        .on("click", || {
            println!("a button was clicked");
        })
}

fn double_click_event_button() -> impl Into<Node> {
    Node::new("button")
        .child("I send double click events")
        .on("dblclick", || {
            println!("a button was double-clicked");
        })
}
