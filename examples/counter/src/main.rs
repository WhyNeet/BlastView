use std::{sync::Arc, time::Duration};

use blastview::{
    context::ViewContext,
    node::Node,
    use_state,
    view::{View, ViewRef},
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
    fn render(&self, cx: &impl ViewContext) -> impl Into<Node> {
        let counter_view = cx.create_view(|| CounterView);
        let auto_counter_view = cx.create_view(|| AutoIncrementCounterView);

        Node::new("main")
            .child(view_wrapper("Counter", counter_view))
            .child(view_wrapper("Counter 2", counter_view))
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
    fn render(&self, cx: &impl ViewContext) -> impl Into<Node> {
        let (count, set_count) = use_state!(cx, 0);

        Node::new("div")
            .child(Node::new("button").child(format!("Sub")).on("click", {
                let set_count = Arc::clone(&set_count);
                move || set_count(count - 1)
            }))
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
    fn render(&self, cx: &impl ViewContext) -> impl Into<Node> {
        let (count, set_count) = cx.use_state(0);

        cx.use_effect(
            move || {
                let set_count = set_count.clone();
                let task = tokio::spawn(async move {
                    let mut interval = tokio::time::interval(Duration::from_secs(1));
                    interval.tick().await;
                    let mut current_count = 0;
                    loop {
                        interval.tick().await;
                        set_count(current_count + 1);
                        current_count += 1;
                    }
                });

                Some(Box::new(move || task.abort()))
            },
            [] as [u8; 0],
        );

        Node::new("div").child(format!("Count is: {}", count.to_string()))
    }
}
