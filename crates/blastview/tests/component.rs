use std::sync::Arc;

use blastview::{
    context::{Context, ViewContext},
    node::Node,
    renderer::Renderer,
    view::{RenderableView, View},
};

#[test]
fn view_rendering_works() {
    struct MyView;
    impl View for MyView {
        fn render(&self, _: &impl ViewContext) -> impl Into<blastview::node::Node> {
            Node::new("div")
                .attr("class", "container")
                .child("Hello world!")
        }
    }

    let context = Context::new(Arc::new(MyView), Default::default(), Default::default());

    let renderer = Renderer::new(Arc::clone(&context));
    let html =
        renderer.render_node_to_string(&RenderableView::render(&MyView, &context).into(), &context);
    assert_eq!(html, r#"<div class="container">Hello world!</div>"#);
}
