use std::sync::Arc;

use blastview::{
    node::Node,
    renderer::Renderer,
    session::LiveSession,
    view::{RenderableView, View, context::ViewContext},
};

#[test]
fn view_rendering_works() {
    struct MyView;
    impl View for MyView {
        fn render(&self, _: &ViewContext) -> impl Into<blastview::node::Node> {
            Node::new("div")
                .attr("class", "container")
                .child("Hello world!")
        }
    }

    let context = ViewContext::default();
    let root_view = context.create(|| MyView);

    let context = Arc::new(context);

    let renderer = Renderer::new(Arc::clone(&context), root_view);
    let html =
        renderer.render_node_to_string(&RenderableView::render(&MyView, &context).into(), &context);
    assert_eq!(html, r#"<div class="container">Hello world!</div>"#);
}
