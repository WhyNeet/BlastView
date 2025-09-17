use std::{sync::Arc, time::Duration};

use blastview::{
    node::Node,
    view::{View, context::ViewRef},
};
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

    blaster::serve(|| AppView).await
}

struct AppView;

impl View for AppView {
    fn render(&self, cx: &blastview::view::context::ViewContext) -> impl Into<Node> {
        let counter_view = cx.create(|| CounterView);
        let auto_counter_view = cx.create(|| AutoIncrementCounterView);

        Node::new("main")
            .child(view_wrapper("Counter", counter_view))
            .child(Node::new("hr"))
            .child(view_wrapper("Auto-increment counter", auto_counter_view))
    }
}

fn view_wrapper(view_name: &str, view: ViewRef) -> impl Into<Node> {
    Node::new("div")
        .attr("class", "container")
        .child(Node::new("h3").child(view_name.to_string()))
        .child(view)
}

struct CounterView;

impl View for CounterView {
    fn render(&self, cx: &blastview::view::context::ViewContext) -> impl Into<Node> {
        let (count, set_count) = cx.use_state(0);

        let set_count_decrement = Arc::clone(&set_count);
        Node::new("div")
            .child(
                Node::new("button")
                    .child(format!("Sub"))
                    .on("click", move || set_count_decrement(count - 1)),
            )
            .child(count.to_string())
            .child(
                Node::new("button")
                    .child(format!("Add"))
                    .on("click", move || set_count(count + 1)),
            )
    }
}

struct AutoIncrementCounterView;

impl View for AutoIncrementCounterView {
    fn render(&self, cx: &blastview::view::context::ViewContext) -> impl Into<Node> {
        let (count, set_count) = cx.use_state(0);

        tokio::spawn(async move {
            tokio::time::sleep(Duration::from_secs(1)).await;
            set_count(count + 1);
        });

        Node::new("div").child(format!("Count is: {}", count.to_string()))
    }
}
