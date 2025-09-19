use blastview::{context::ViewContext, node::Node, use_async_memo, use_state, view::View};
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
    fn render(&self, cx: &impl ViewContext) -> impl Into<Node> {
        let (count, set_count) = use_state!(cx, 38);
        let expensive_computation = use_async_memo!(
            cx,
            async || {
                // this function may take a lot of time to run
                // thus, use async memo
                fib(count)
            },
            count
        );

        Node::new("div")
            .attr("class", "container")
            .child(Node::new("p").child(format!("count: {count}")))
            .child(Node::new("button").child("Sub").on("click", {
                let set_count = set_count.clone();
                move || set_count(count - 1)
            }))
            .child(
                Node::new("button")
                    .child("Add")
                    .on("click", move || set_count(count + 1)),
            )
            .child(Node::new("p").child(format!(
                "fib({count}): {}",
                expensive_computation.unwrap_or(0)
            )))
    }
}

fn fib(n: u64) -> u64 {
    if n < 2 {
        return n;
    }

    fib(n - 1) + fib(n - 2)
}
