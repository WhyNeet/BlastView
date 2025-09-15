use blastview::{
    node::Node,
    renderer::StaticRenderer,
    view::{View, context::ViewContext},
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

    let output = StaticRenderer::render_to_string(|| MyView);
    assert_eq!(output, r#"<div class="container">Hello world!</div>"#);
}
